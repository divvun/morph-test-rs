use crate::backend::Backend;
use crate::types::{CaseResult, Direction, Summary, TestSuite};
use rayon::prelude::*;
fn vec_eq_exact(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return false;
        }
    }
    true
}
pub fn run_suites<B: Backend>(backend: &B, suites: &[TestSuite]) -> Summary {
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
                    let passed = vec_eq_exact(&actual, &case.expect);
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
