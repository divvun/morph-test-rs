use crate::types::{Direction, TestCase, TestSuite};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendChoice {
    #[default]
    Auto,
    Hfst,
    Xerox,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct HfstCfg {
    pub Gen: Option<String>,
    pub Morph: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct XeroxCfg {
    pub Gen: Option<String>,
    pub Morph: Option<String>,
    pub App: Option<String>, // t.d. "lookup"
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RawConfig {
    pub hfst: Option<HfstCfg>,
    pub xerox: Option<XeroxCfg>,
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
    pub Config: Option<RawConfig>,
    pub Tests: BTreeMap<String, BTreeMap<String, OneOrMany>>,
}
#[derive(Debug, Clone)]
pub struct SuiteWithConfig {
    pub suite: TestSuite,
    pub backend: BackendChoice,
    pub lookup_cmd: String,
    pub gen_fst: String,
    pub morph_fst: Option<String>,
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
        let mut cases = Vec::new();
        for (group, map) in &raw.Tests {
            for (input, expected) in map {
                let expect_vec = match expected {
                    OneOrMany::One(s) => vec![s.clone()],
                    OneOrMany::Many(v) => v.clone(),
                };
                let name = format!("{}: {}", group, input);
                cases.push(TestCase {
                    name,
                    direction: Direction::Generate,
                    input: input.clone(),
                    expect: expect_vec,
                });
            }
        }
        let suite = TestSuite {
            name: f.file_name()
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
fn resolve_backend(raw: &RawSpec, prefer: &BackendChoice)
    -> Result<(BackendChoice, String, String, Option<String>)>
{
    let cfg = raw.Config.as_ref().ok_or_else(|| anyhow::anyhow!("Config manglar"))?;
    // Vel backend: preferert, elles HFST om tilgjengeleg, elles Xerox.
    let chosen = match prefer {
        BackendChoice::Hfst => BackendChoice::Hfst,
        BackendChoice::Xerox => BackendChoice::Xerox,
        BackendChoice::Auto => {
            if cfg.hfst.as_ref().and_then(|h| h.Gen.clone()).is_some() {
                BackendChoice::Hfst
            } else if cfg.xerox.as_ref().and_then(|x| x.Gen.clone()).is_some() {
                BackendChoice::Xerox
            } else {
                return Err(anyhow::anyhow!("Fann verken HFST.Gen eller Xerox.Gen i Config"));
            }
        }
    };
    match chosen {
        BackendChoice::Hfst => {
            let h = cfg.hfst.as_ref().ok_or_else(|| anyhow::anyhow!("Config.hfst manglar"))?;
            let gen = h.Gen.clone().ok_or_else(|| anyhow::anyhow!("Config.hfst.Gen manglar"))?;
            // Default kommando for HFST-lookup
            let cmd = "hfst-lookup".to_string();
            Ok((BackendChoice::Hfst, cmd, gen, h.Morph.clone()))
        }
        BackendChoice::Xerox => {
            let x = cfg.xerox.as_ref().ok_or_else(|| anyhow::anyhow!("Config.xerox manglar"))?;
            let gen = x.Gen.clone().ok_or_else(|| anyhow::anyhow!("Config.xerox.Gen manglar"))?;
            let cmd = x.App.clone().unwrap_or_else(|| "lookup".to_string());
            Ok((BackendChoice::Xerox, cmd, gen, x.Morph.clone()))
        }
        BackendChoice::Auto => unreachable!(),
    }
}
