use crate::types::{Direction, TestCase, TestSuite};
use crate::{t, t_args};
use anyhow::{Context, Result, anyhow};
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendChoice {
    #[default]
    Auto,
    Hfst,
    Foma,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct HfstCfg {
    pub r#gen: Option<String>,
    pub morph: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FomaCfg {
    pub r#gen: Option<String>,
    pub morph: Option<String>,
    pub app: Option<String>, // default: flookup
}

// Accept alias for backward compatibility
#[derive(Debug, Deserialize, Clone)]
pub struct RawConfig {
    #[serde(alias = "Hfst")]
    pub hfst: Option<HfstCfg>,
    #[serde(alias = "Foma", alias = "xerox", alias = "Xerox")]
    pub foma: Option<FomaCfg>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RawSpec {
    pub config: Option<RawConfig>,
    pub tests: IndexMap<String, IndexMap<String, OneOrMany>>,
}

#[derive(Debug, Clone)]
pub struct SuiteWithConfig {
    pub suite: TestSuite,
    pub backend: BackendChoice,
    pub lookup_cmd: String,
    pub gen_fst: String,
    pub morph_fst: Option<String>,
}

fn trim_owned(s: &str) -> String {
    s.trim().to_string()
}

pub fn load_specs(paths: &[PathBuf], prefer: BackendChoice) -> Result<Vec<SuiteWithConfig>> {
    let mut files = Vec::new();
    for p in paths {
        if p.is_dir() {
            for entry in WalkDir::new(p) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" || ext == "yml" {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        } else {
            files.push(p.clone());
        }
    }
    let mut out = Vec::new();
    for f in files {
        let content = fs::read_to_string(&f)
            .with_context(|| t_args!("spec-failed-to-read", "file" => f.display()))?;
        let raw: RawSpec = serde_yaml::from_str(&content)
            .with_context(|| t_args!("spec-yaml-error", "file" => f.display()))?;
        let (backend, lookup_cmd, gen_fst, morph_fst) = resolve_backend(&raw, &prefer, &f)
            .with_context(|| t_args!("spec-incomplete-config", "file" => f.display()))?;
        let mut cases: Vec<TestCase> = Vec::new();
        // For each group: build both generate-cases and invert to analyze-cases
        for (group, map) in &raw.tests {
            let group_name = group.trim();
            // Accumulator for analyze: surface -> set of analyses (lexical-key)
            let mut surface_to_analyses: IndexMap<String, BTreeSet<String>> = IndexMap::new();
            for (lexical, expected) in map {
                let lexical_trim = lexical.trim().to_string();
                let expect_vec: Vec<String> = match expected {
                    OneOrMany::One(s) => vec![trim_owned(s)],
                    OneOrMany::Many(v) => v.iter().map(|s| s.trim().to_string()).collect(),
                };
                // 1) Generate-case: input=lexical, expect=surface-former
                let name = format!("{}: {}", group_name, &lexical_trim);
                cases.push(TestCase {
                    name,
                    direction: Direction::Generate,
                    input: lexical_trim.clone(),
                    expect: expect_vec.clone(),
                });
                // 2) Invert to analyze: for each surface add lexical as analysis
                for surf in expect_vec {
                    let entry = surface_to_analyses.entry(surf).or_default();
                    entry.insert(lexical_trim.clone());
                }
            }
            // Create Analyze-cases from the accumulator
            for (surface, analyses_set) in surface_to_analyses {
                let mut analyses: Vec<String> = analyses_set.into_iter().collect();
                // Stable, deterministic order
                analyses.sort();
                let name = format!("{group_name}: {surface}");
                cases.push(TestCase {
                    name,
                    direction: Direction::Analyze,
                    input: surface,
                    expect: analyses,
                });
            }
        }
        let suite = TestSuite {
            name: f
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "suite".to_string()),
            cases,
        };
        out.push(SuiteWithConfig {
            suite,
            backend,
            lookup_cmd,
            gen_fst,
            morph_fst,
        });
    }
    Ok(out)
}

fn resolve_path_relative_to_yaml(path: &str, yaml_file_path: &PathBuf) -> String {
    let path_buf = std::path::Path::new(path);
    if path_buf.is_absolute() {
        path.to_string()
    } else {
        // Resolve relative to the YAML file's directory
        if let Some(yaml_dir) = yaml_file_path.parent() {
            yaml_dir.join(path).to_string_lossy().into_owned()
        } else {
            path.to_string()
        }
    }
}

pub fn determine_hfst_lookup_tool(gen_path: &str, morph_path: Option<&str>) -> String {
    // Check the generator FST extension first
    let gen_path_obj = std::path::Path::new(gen_path);
    if let Some(ext) = gen_path_obj.extension() {
        if ext == "hfst" {
            return "hfst-lookup".to_string();
        } else if ext == "hfstol" {
            return "hfst-optimised-lookup".to_string();
        }
    }
    
    // If generator extension is unclear, check morph FST extension
    if let Some(morph) = morph_path {
        let morph_path_obj = std::path::Path::new(morph);
        if let Some(ext) = morph_path_obj.extension() {
            if ext == "hfst" {
                return "hfst-lookup".to_string();
            } else if ext == "hfstol" {
                return "hfst-optimised-lookup".to_string();
            }
        }
    }
    
    // Default to optimised-lookup for backward compatibility
    "hfst-optimised-lookup".to_string()
}

fn resolve_backend(
    raw: &RawSpec,
    prefer: &BackendChoice,
    yaml_file_path: &PathBuf,
) -> Result<(BackendChoice, String, String, Option<String>)> {
    let cfg = raw
        .config
        .as_ref()
        .ok_or_else(|| anyhow!(t!("spec-missing-config")))?;
    let chosen = match prefer {
        BackendChoice::Hfst => BackendChoice::Hfst,
        BackendChoice::Foma => BackendChoice::Foma,
        BackendChoice::Auto => {
            if cfg.hfst.as_ref().and_then(|h| h.r#gen.clone()).is_some() {
                BackendChoice::Hfst
            } else if cfg.foma.as_ref().and_then(|x| x.r#gen.clone()).is_some() {
                BackendChoice::Foma
            } else {
                return Err(anyhow!(t!("spec-missing-gen")));
            }
        }
    };
    match chosen {
        BackendChoice::Hfst => {
            let h = cfg
                .hfst
                .as_ref()
                .ok_or_else(|| anyhow!(t!("spec-missing-hfst")))?;
            let gen_ = h
                .r#gen
                .clone()
                .ok_or_else(|| anyhow!(t!("spec-missing-hfst-gen")))?;
            let gen_ = resolve_path_relative_to_yaml(&gen_.trim(), yaml_file_path);
            let morph = h.morph.clone().map(|m| resolve_path_relative_to_yaml(&m.trim(), yaml_file_path));
            let cmd = determine_hfst_lookup_tool(&gen_, morph.as_deref());
            Ok((BackendChoice::Hfst, cmd, gen_, morph))
        }
        BackendChoice::Foma => {
            let x = cfg
                .foma
                .as_ref()
                .ok_or_else(|| anyhow!(t!("spec-missing-foma")))?;
            let gen_ = x
                .r#gen
                .clone()
                .ok_or_else(|| anyhow!(t!("spec-missing-foma-gen")))?;
            let gen_ = resolve_path_relative_to_yaml(&gen_.trim(), yaml_file_path);
            let morph = x.morph.clone().map(|m| resolve_path_relative_to_yaml(&m.trim(), yaml_file_path));
            let cmd = x
                .app
                .clone()
                .unwrap_or_else(|| "flookup".to_string())
                .trim()
                .to_string();
            Ok((BackendChoice::Foma, cmd, gen_, morph))
        }
        BackendChoice::Auto => unreachable!(),
    }
}
