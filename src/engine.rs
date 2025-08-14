use crate::backend::Backend;
use crate::types::{CaseResult, Direction, Summary, TestSuite};
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
            all_cases.push(c.clone());
        }
    }

    // Group cases by direction for batch processing
    let mut analyze_cases = Vec::new();
    let mut generate_cases = Vec::new();

    for (idx, case) in all_cases.iter().enumerate() {
        match case.direction {
            Direction::Analyze => analyze_cases.push((idx, case)),
            Direction::Generate => generate_cases.push((idx, case)),
        }
    }

    let mut results = vec![
        CaseResult {
            name: String::new(),
            direction: Direction::Analyze,
            input: String::new(),
            expected: vec![],
            actual: vec![],
            error: Some("Not processed".to_string()),
            passed: false,
        };
        all_cases.len()
    ];

    // Process analyze tests in batch
    if !analyze_cases.is_empty() {
        let inputs: Vec<String> = analyze_cases
            .iter()
            .map(|(_, case)| case.input.clone())
            .collect();
        match backend.analyze_batch(&inputs) {
            Ok(batch_results) => {
                for ((idx, case), actual) in analyze_cases.iter().zip(batch_results.iter()) {
                    let passed = if ignore_extra_analyses {
                        expected_subset_of_actual(actual, &case.expect)
                    } else {
                        set_eq(actual, &case.expect)
                    };

                    results[*idx] = CaseResult {
                        name: case.name.clone(),
                        direction: case.direction.clone(),
                        input: case.input.clone(),
                        expected: case.expect.clone(),
                        actual: actual.clone(),
                        error: None,
                        passed,
                    };
                }
            }
            Err(e) => {
                // Mark all analyze tests as failed due to batch error
                for (idx, case) in &analyze_cases {
                    results[*idx] = CaseResult {
                        name: case.name.clone(),
                        direction: case.direction.clone(),
                        input: case.input.clone(),
                        expected: case.expect.clone(),
                        actual: vec![],
                        error: Some(format!("Batch analyze error: {e}")),
                        passed: false,
                    };
                }
            }
        }
    }

    // Process generate tests in batch
    if !generate_cases.is_empty() {
        let inputs: Vec<String> = generate_cases
            .iter()
            .map(|(_, case)| case.input.clone())
            .collect();
        match backend.generate_batch(&inputs) {
            Ok(batch_results) => {
                for ((idx, case), actual) in generate_cases.iter().zip(batch_results.iter()) {
                    let passed = set_eq(actual, &case.expect);

                    results[*idx] = CaseResult {
                        name: case.name.clone(),
                        direction: case.direction.clone(),
                        input: case.input.clone(),
                        expected: case.expect.clone(),
                        actual: actual.clone(),
                        error: None,
                        passed,
                    };
                }
            }
            Err(e) => {
                // Mark all generate tests as failed due to batch error
                for (idx, case) in &generate_cases {
                    results[*idx] = CaseResult {
                        name: case.name.clone(),
                        direction: case.direction.clone(),
                        input: case.input.clone(),
                        expected: case.expect.clone(),
                        actual: vec![],
                        error: Some(format!("Batch generate error: {e}")),
                        passed: false,
                    };
                }
            }
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    Summary {
        total: results.len(),
        passed,
        failed,
        cases: results,
    }
}

// With batch processing, parallelization is less critical since we only spawn
// 1-2 subprocesses per suite (analyze + generate). We can add chunked parallel
// batching later if needed, but the batch processing alone should provide
// the main performance improvement.
