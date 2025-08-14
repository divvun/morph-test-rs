use crate::pool::PooledBackend;
use crate::types::{CaseResult, Direction, Summary, TestSuite};
use anyhow::Result;
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

pub async fn run_suites_async(
    backend: &PooledBackend,
    suites: &[TestSuite],
    ignore_extra_analyses: bool,
) -> Result<Summary> {
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

    // Process analyze and generate batches in parallel
    let (analyze_result, generate_result) = futures::future::join(
        async {
            if analyze_cases.is_empty() {
                return Ok(Vec::new());
            }
            let inputs: Vec<String> = analyze_cases
                .iter()
                .map(|(_, case)| case.input.clone())
                .collect();
            backend.analyze_batch(&inputs).await
        },
        async {
            if generate_cases.is_empty() {
                return Ok(Vec::new());
            }
            let inputs: Vec<String> = generate_cases
                .iter()
                .map(|(_, case)| case.input.clone())
                .collect();
            backend.generate_batch(&inputs).await
        },
    )
    .await;

    // Process analyze results
    match analyze_result {
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

    // Process generate results
    match generate_result {
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

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    Ok(Summary {
        total: results.len(),
        passed,
        failed,
        cases: results,
    })
}
