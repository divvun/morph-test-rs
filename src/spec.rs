use crate::types::{Direction, TestCase, TestSuite};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
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
    pub gen: Option<String>,
    pub morph: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FomaCfg {
    pub gen: Option<String>,
    pub morph: Option<String>,
    pub app: Option<String>, // default: flookup
}
// Godtek alias for bakoverkompatibilitet
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
    pub tests: BTreeMap<String, BTreeMap<String, OneOrMany>>,
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
            .with_context(|| format!("Klarte ikkje å lesa: {}", f.display()))?;
        let raw: RawSpec = serde_yaml::from_str(&content)
            .with_context(|| format!("YAML-feil i: {}", f.display()))?;
        let (backend, lookup_cmd, gen_fst, morph_fst) = resolve_backend(&raw, &prefer)
            .with_context(|| format!("Mangelfull eller utydeleg Config i {}", f.display()))?;
        let mut cases: Vec<TestCase> = Vec::new();
        // For kvar gruppe: bygg både generate-cases og inverter til analyze-cases
        for (group, map) in &raw.tests {
            let group_name = group.trim();
            // Akkumulator for analyze: surface -> set av analysar (lexical-nøkkel)
            let mut surface_to_analyses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
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
                // 2) Inverter til analyze: for kvar surface legg til lexical som analyse
                for surf in expect_vec {
                    let entry = surface_to_analyses
                        .entry(surf)
                        .or_insert_with(BTreeSet::new);
                    entry.insert(lexical_trim.clone());
                }
            }
            // Lag Analyze-cases frå akkumulatoren
            for (surface, analyses_set) in surface_to_analyses {
                let mut analyses: Vec<String> = analyses_set.into_iter().collect();
                // Stabil, deterministisk rekkjefølgje
                analyses.sort();
                let name = format!("{}: {}", group_name, surface);
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
fn resolve_backend(
    raw: &RawSpec,
    prefer: &BackendChoice,
) -> Result<(BackendChoice, String, String, Option<String>)> {
    let cfg = raw
        .config
        .as_ref()
        .ok_or_else(|| anyhow!("Config manglar"))?;
    let chosen = match prefer {
        BackendChoice::Hfst => BackendChoice::Hfst,
        BackendChoice::Foma => BackendChoice::Foma,
        BackendChoice::Auto => {
            if cfg.hfst.as_ref().and_then(|h| h.gen.clone()).is_some() {
                BackendChoice::Hfst
            } else if cfg.foma.as_ref().and_then(|x| x.gen.clone()).is_some() {
                BackendChoice::Foma
            } else {
                return Err(anyhow!("Fann verken HFST.Gen eller Foma.Gen i Config"));
            }
        }
    };
    match chosen {
        BackendChoice::Hfst => {
            let h = cfg
                .hfst
                .as_ref()
                .ok_or_else(|| anyhow!("Config.hfst manglar"))?;
            let gen = h
                .gen
                .clone()
                .ok_or_else(|| anyhow!("Config.hfst.Gen manglar"))?;
            let gen = gen.trim().to_string();
            let morph = h.morph.clone().map(|m| m.trim().to_string());
            let cmd = "hfst-optimised-lookup".to_string();
            Ok((BackendChoice::Hfst, cmd, gen, morph))
        }
        BackendChoice::Foma => {
            let x = cfg
                .foma
                .as_ref()
                .ok_or_else(|| anyhow!("Config.foma manglar"))?;
            let gen = x
                .gen
                .clone()
                .ok_or_else(|| anyhow!("Config.foma.Gen manglar"))?;
            let gen = gen.trim().to_string();
            let morph = x.morph.clone().map(|m| m.trim().to_string());
            let cmd = x
                .app
                .clone()
                .unwrap_or_else(|| "flookup".to_string())
                .trim()
                .to_string();
            Ok((BackendChoice::Foma, cmd, gen, morph))
        }
        BackendChoice::Auto => unreachable!(),
    }
}
