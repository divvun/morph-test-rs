use anyhow::Result;
use morph_test2::backend::Backend;
use morph_test2::engine::run_suites;
use morph_test2::spec::{BackendChoice, load_specs};
use std::fs;
use tempfile::tempdir;

struct MockGen;

impl Backend for MockGen {
    fn analyze_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let results = inputs.iter().map(|input| {
            // Return expected analysis results for the trimmed test inputs
            match input.as_str() {
                "gæljoejidie" => vec!["gæljodh+V+TV+Ind+Prs+Pl2".into()],
                "bar" => vec!["foo+V".into()],
                "baz" => vec!["foo+V".into()],
                _ => vec![],
            }
        }).collect();
        Ok(results)
    }
    fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let results = inputs.iter().map(|input| {
            // Returnerer eksakt, utan ekstra blank
            match input.as_str() {
                "gæljodh+V+TV+Ind+Prs+Pl2" => vec!["gæljoejidie".into()],
                "foo+V" => vec!["bar".into(), "baz".into()],
                _ => vec![],
            }
        }).collect();
        Ok(results)
    }
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn trims_spaces_in_yaml_keys_and_values() -> Result<()> {
    morph_test2::i18n::init();
    let dir = tempdir()?;
    let file = dir.path().join("suite.yaml");
    let yaml = r#"
Config:
  hfst:
    Gen: /dev/null
Tests:
  Verb - sample:
    "gæljodh+V+TV+Ind+Prs+Pl2   ": "   gæljoejidie "
    "foo+V": [ "bar  ", "  baz" ]
"#;
    fs::write(&file, yaml)?;
    let swc = load_specs(&[file.clone()], BackendChoice::Hfst)?;
    assert_eq!(swc.len(), 1);
    let suite = &swc[0].suite;
    // Sjekk at trimming skjedde ved parsing
    let c1 = suite
        .cases
        .iter()
        .find(|c| c.input.starts_with("gæljodh+"))
        .unwrap();
    assert_eq!(c1.input, "gæljodh+V+TV+Ind+Prs+Pl2");
    assert_eq!(c1.expect, vec!["gæljoejidie"]);
    let c2 = suite.cases.iter().find(|c| c.input == "foo+V").unwrap();
    assert_eq!(c2.expect, vec!["bar", "baz"]);
    // Kjør testen med mock-backend
    let backend = MockGen;
    let summary = run_suites(&backend, &[suite.clone()], true);
    assert_eq!(summary.failed, 0);
    Ok(())
}
