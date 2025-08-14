use crate::backend::Backend;
use crate::types::{CaseResult, Direction, Summary, TestSuite};
use rayon::prelude::*;
use std::collections::BTreeSet;

fn set_eq(a: &[String], b: &[String]) -> bool {
    let sa: BTreeSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let sb: BTreeSet<&str> = b.iter().map(|s| s.as_str()).collect();
    sa == sb
}

fn expected_subset_of_actual(actual: &[String], expected: &[String]) -> bool {
    let sa: BTreeSet<&str> = actual.iter().map(|s| s.as_str()).collect();
    let sb: BTreeSet<&str> = expected.iter().map(|s| s.as_str()).collect();
    sb.is_subset(&sa)
}

pub fn run_suites<B: Backend>(
    backend: &B,
    suites: &[TestSuite],
    ignore_extra_analyses: bool,
) -> Summary {
    let mut all_cases = Vec::new();
    for s in suites {
        for c in &s.cases {
            all_cases.push((s.name.clone(), c.clone()));
        }
    }

    let results: Vec<CaseResult> = all_cases
        .par_iter()
        .map(|(_suite_name, case)| {
            let res = match case.direction {
                Direction::Analyze => backend.analyze(&case.input),
                Direction::Generate => backend.generate(&case.input),
            };
            match res {
                Ok(actual) => {
                    let passed = match case.direction {
                        Direction::Analyze if ignore_extra_analyses => {
                            expected_subset_of_actual(&actual, &case.expect)
                        }
                        _ => set_eq(&actual, &case.expect),
                    };
                    CaseResult {
                        name: case.name.clone(),
                        direction: case.direction.clone(),
                        input: case.input.clone(),
                        expected: case.expect.clone(),
                        actual,
                        error: None,
                        passed,
                    }
                }
                Err(e) => CaseResult {
                    name: case.name.clone(),
                    direction: case.direction.clone(),
                    input: case.input.clone(),
                    expected: case.expect.clone(),
                    actual: vec![],
                    error: Some(e.to_string()),
                    passed: false,
                },
            }
        })
        .collect();

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    Summary {
        total: results.len(),
        passed,
        failed,
        cases: results,
    }
}
