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
                        if ext == "yaml" || ext == "yml" || ext == "lexc" {
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
        
        // Check if this is a lexc file
        if f.extension().map_or(false, |ext| ext == "lexc") {
            // Parse as lexc test data
            let lexc_test_sets = parse_lexc_test_data(&content)
                .with_context(|| format!("Failed to parse lexc test data from {}", f.display()))?;
            
            if !lexc_test_sets.is_empty() {
                let suites = convert_lexc_to_suites(lexc_test_sets, &f, prefer.clone())
                    .with_context(|| format!("Failed to convert lexc test data from {}", f.display()))?;
                out.extend(suites);
            }
            continue;
        }
        
        // Parse as YAML
        let raw: RawSpec = serde_yaml::from_str(&content)
            .with_context(|| t_args!("spec-yaml-error", "file" => f.display()))?;
        let (backend, lookup_cmd, gen_fst, morph_fst) = resolve_backend(&raw, &prefer, &f)
            .with_context(|| t_args!("spec-incomplete-config", "file" => f.display()))?;
        let mut cases: Vec<TestCase> = Vec::new();
        // Global accumulator for analyze: (surface, group) -> set of analyses (lexical-key)
        let mut surface_to_analyses: IndexMap<(String, String), BTreeSet<String>> = IndexMap::new();
        
        // For each group: build generate-cases and collect surface forms
        for (group, map) in &raw.tests {
            let group_name = group.trim();
            for (lexical, expected) in map {
                let lexical_trim = lexical.trim().to_string();
                let expect_vec: Vec<String> = match expected {
                    OneOrMany::One(s) => vec![trim_owned(s)],
                    OneOrMany::Many(v) => v.iter().map(|s| s.trim().to_string()).collect(),
                };
                // Separate positive and negative expectations
                let mut positive_forms = Vec::new();
                let mut negative_forms = Vec::new();
                
                for surf in &expect_vec {
                    if surf.starts_with('~') {
                        // Negative test: remove ~ prefix for the actual form
                        let actual_form = surf[1..].to_string();
                        negative_forms.push(actual_form);
                    } else {
                        // Positive test
                        positive_forms.push(surf.clone());
                    }
                }
                
                // 1) Generate-case: input=lexical, expect=positive surface forms, expect_not=negative forms
                let name = format!("{}: {}", group_name, &lexical_trim);
                cases.push(TestCase {
                    name,
                    direction: Direction::Generate,
                    input: lexical_trim.clone(),
                    expect: positive_forms.clone(),
                    expect_not: negative_forms.clone(),
                });
                
                // 2) Invert to analyze: only positive surface forms should analyze to lexical
                for surf in positive_forms {
                    let entry = surface_to_analyses.entry((surf, group_name.to_string())).or_default();
                    entry.insert(lexical_trim.clone());
                }
                
                // 2b) Negative analyze-cases: negative forms should not analyze to anything
                for neg_form in negative_forms {
                    // Create a separate negative analysis test
                    let name = format!("Analysis (negative): {}", neg_form);
                    cases.push(TestCase {
                        name,
                        direction: Direction::Analyze,
                        input: neg_form,
                        expect: vec![], // Expect no result
                        expect_not: vec![], // No negative expectations needed for these
                    });
                }
            }
        }
        
        // Create Analyze-cases from the global accumulator
        for ((surface, group_name), analyses_set) in surface_to_analyses {
            let mut analyses: Vec<String> = analyses_set.into_iter().collect();
            // Stable, deterministic order
            analyses.sort();
            let name = format!("{}: {}", group_name, surface);
            cases.push(TestCase {
                name,
                direction: Direction::Analyze,
                input: surface,
                expect: analyses,
                expect_not: vec![], // No negative expectations for regular analysis tests
            });
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

#[derive(Debug, Clone)]
pub struct LexcTestSet {
    pub fst_type: String,    // e.g., "gt-norm"
    pub test_name: String,   // e.g., "gierehtse (*\"pulk\"*)"
    pub tests: IndexMap<String, String>,  // surface_form -> analysis
}

pub fn parse_lexc_test_data(content: &str) -> Result<Vec<LexcTestSet>> {
    let mut test_sets = Vec::new();
    let mut current_set: Option<LexcTestSet> = None;
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.starts_with("!!€") {
            let test_line = &line[5..]; // Remove !!€ prefix (€ is 3 bytes in UTF-8)
            
            if test_line.starts_with(' ') {
                // This is a test data line: " surface_form: analysis"
                if let Some(ref mut test_set) = current_set {
                    let test_line = test_line.trim();
                    if let Some(colon_pos) = test_line.find(':') {
                        let surface_form = test_line[..colon_pos].trim().to_string();
                        let analysis_part = &test_line[colon_pos + 1..];
                        // Filter out comments starting with '!'
                        let analysis = if let Some(comment_pos) = analysis_part.find('!') {
                            analysis_part[..comment_pos].trim().to_string()
                        } else {
                            analysis_part.trim().to_string()
                        };
                        test_set.tests.insert(surface_form, analysis);
                    }
                }
            } else {
                // This is a header line: "fst_type: test_name # comment"
                // Save previous test set if exists
                if let Some(test_set) = current_set.take() {
                    if !test_set.tests.is_empty() {
                        test_sets.push(test_set);
                    }
                }
                
                // Parse header line
                let header = if let Some(hash_pos) = test_line.find('#') {
                    &test_line[..hash_pos]
                } else {
                    test_line
                };
                
                if let Some(colon_pos) = header.find(':') {
                    let fst_type = header[..colon_pos].trim().to_string();
                    let test_name = header[colon_pos + 1..].trim().to_string();
                    
                    current_set = Some(LexcTestSet {
                        fst_type,
                        test_name,
                        tests: IndexMap::new(),
                    });
                }
            }
        }
    }
    
    // Don't forget the last test set
    if let Some(test_set) = current_set {
        if !test_set.tests.is_empty() {
            test_sets.push(test_set);
        }
    }
    
    Ok(test_sets)
}

fn search_in_directory(dir: &std::path::Path, patterns: &[String]) -> Vec<PathBuf> {
    let mut found_files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    for pattern in patterns {
                        if filename_str == *pattern {
                            found_files.push(path);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    found_files
}

fn search_upward_from_directory(start_dir: &std::path::Path, patterns: &[String]) -> Vec<PathBuf> {
    let mut current_dir = start_dir;
    
    loop {
        // Search in current directory
        let files = search_in_directory(current_dir, patterns);
        if !files.is_empty() {
            return files;
        }
        
        // Move up to parent directory
        if let Some(parent) = current_dir.parent() {
            current_dir = parent;
        } else {
            break; // Reached root
        }
    }
    
    Vec::new()
}

pub fn find_fst_files(lexc_file_path: &PathBuf, fst_type: &str) -> Result<(String, Option<String>)> {
    let analyzer_patterns = [
        format!("analyser-{}.hfstol", fst_type),
        format!("analyser-{}.hfst", fst_type),
        format!("analyzer-{}.hfstol", fst_type),
        format!("analyzer-{}.hfst", fst_type),
    ];
    
    let generator_patterns = [
        format!("generator-{}.hfstol", fst_type),
        format!("generator-{}.hfst", fst_type),
    ];
    
    // Search starting points:
    // 1. Current working directory (where the test script is run from)
    // 2. Directory containing the lexc file
    let mut search_starts = Vec::new();
    
    // Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        search_starts.push(cwd);
    }
    
    // Directory containing the lexc file
    if let Some(parent) = lexc_file_path.parent() {
        search_starts.push(parent.to_path_buf());
    }
    
    for start_dir in &search_starts {
        // Look for generator FSTs first, searching upward
        let generator_files = search_upward_from_directory(start_dir, &generator_patterns);
        
        if let Some(generator_path) = generator_files.first() {
            // Found a generator, now look for corresponding analyzer in the same directory
            if let Some(generator_dir) = generator_path.parent() {
                let analyzer_files = search_in_directory(generator_dir, &analyzer_patterns);
                let analyzer_path = analyzer_files.first().map(|p| p.to_string_lossy().to_string());
                
                return Ok((generator_path.to_string_lossy().to_string(), analyzer_path));
            }
        }
    }
    
    Err(anyhow!("Could not find FST files for type: {} by searching upward from current directory or lexc file location", fst_type))
}

pub fn convert_lexc_to_suites(lexc_test_sets: Vec<LexcTestSet>, lexc_file_path: &PathBuf, _prefer: BackendChoice) -> Result<Vec<SuiteWithConfig>> {
    // Group test sets by FST type
    let mut fst_groups: IndexMap<String, Vec<LexcTestSet>> = IndexMap::new();
    
    for test_set in lexc_test_sets {
        fst_groups.entry(test_set.fst_type.clone()).or_default().push(test_set);
    }
    
    let mut suites = Vec::new();
    
    for (fst_type, test_sets) in fst_groups {
        // Find FST files for this type
        let (gen_fst, morph_fst) = find_fst_files(lexc_file_path, &fst_type)?;
        
        // Determine lookup command
        let lookup_cmd = determine_hfst_lookup_tool(&gen_fst, morph_fst.as_deref());
        
        // Combine all test sets for this FST type into a single suite
        let mut all_cases = Vec::new();
        
        for test_set in test_sets {
            let group_name = format!("{} ({})", test_set.test_name, fst_type);
            
            // Group surface forms by analysis for generation tests
            let mut analysis_to_surfaces: IndexMap<String, Vec<String>> = IndexMap::new();
            for (surface_form, analysis) in &test_set.tests {
                analysis_to_surfaces.entry(analysis.clone()).or_default().push(surface_form.clone());
            }
            
            // Generate test cases: one test per analysis, expecting all surface forms
            for (analysis, surface_forms) in &analysis_to_surfaces {
                let name = format!("{}: {}", group_name, analysis);
                all_cases.push(TestCase {
                    name,
                    direction: Direction::Generate,
                    input: analysis.clone(),
                    expect: surface_forms.clone(),
                    expect_not: vec![],
                });
            }
            
            // Analysis test cases: one per surface form
            for (surface_form, analysis) in &test_set.tests {
                let name = format!("{}: {}", group_name, &surface_form);
                all_cases.push(TestCase {
                    name,
                    direction: Direction::Analyze,
                    input: surface_form.clone(),
                    expect: vec![analysis.clone()],
                    expect_not: vec![],
                });
            }
        }
        
        let suite_name = format!("{}-{}.lexc", 
            lexc_file_path.file_stem().unwrap_or_default().to_string_lossy(),
            fst_type
        );
        
        let suite = TestSuite {
            name: suite_name,
            cases: all_cases,
        };
        
        suites.push(SuiteWithConfig {
            suite,
            backend: BackendChoice::Hfst, // lexc files always use HFST
            lookup_cmd,
            gen_fst,
            morph_fst,
        });
    }
    
    Ok(suites)
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
