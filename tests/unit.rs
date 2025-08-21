use anyhow::Result;
use morph_test2::backend::Backend;
use morph_test2::engine::run_suites;
use morph_test2::types::*;

struct MockBackend;

impl Backend for MockBackend {
    fn analyze_batch(&self, _inputs: &[String]) -> Result<Vec<Vec<String>>> {
        Ok(_inputs.iter().map(|_| vec![]).collect())
    }

    fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let results = inputs.iter().map(|input| {
            match input.as_str() {
                "gæljodh+V+TV+Ind+Prs+Sg1" => vec!["gæljoem".into()],
                "multi" => vec!["a".into(), "b".into()],
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
fn exact_match_and_order() {
    morph_test2::i18n::init();
    let suite = TestSuite {
        name: "sample".into(),
        cases: vec![
            TestCase {
                name: "ok".into(),
                direction: Direction::Generate,
                input: "gæljodh+V+TV+Ind+Prs+Sg1".into(),
                expect: vec!["gæljoem".into()],
                expect_not: vec![],
            },
            TestCase {
                name: "order_sensitive_fail".into(),
                direction: Direction::Generate,
                input: "multi".into(),
                expect: vec!["b".into(), "a".into()],
                expect_not: vec![],
            },
        ],
    };
    let backend = MockBackend;
    let summary = run_suites(&backend, &[suite], true);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 0);
}
