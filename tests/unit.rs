use anyhow::Result;
use morph_test::backend::Backend;
use morph_test::engine::run_suites;
use morph_test::report::render_human;
use morph_test::types::*;
struct MockBackend;
impl Backend for MockBackend {
    fn analyze(&self, _input: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn generate(&self, input: &str) -> Result<Vec<String>> {
        Ok(match input {
            "gæljodh+V+TV+Ind+Prs+Sg1" => vec!["gæljoem".into()],
            "multi" => vec!["a".into(), "b".into()],
            _ => vec![],
        })
    }
}
#[test]
fn exact_match_and_order() {
    let suite = TestSuite {
        name: "sample".into(),
        cases: vec![
            TestCase {
                name: "ok".into(),
                direction: Direction::Generate,
                input: "gæljodh+V+TV+Ind+Prs+Sg1".into(),
                expect: vec!["gæljoem".into()],
            },
            TestCase {
                name: "order_sensitive_fail".into(),
                direction: Direction::Generate,
                input: "multi".into(),
                expect: vec!["b".into(), "a".into()],
            },
        ],
    };
    let backend = MockBackend;
    let summary = run_suites(&backend, &[suite]);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    let text = render_human(&summary);
    assert!(text.contains("[OK]"));
    assert!(text.contains("[FAIL]"));
}
