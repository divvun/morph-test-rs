// tests/order_insensitive.rs
use anyhow::Result;
use morph_test2::backend::Backend;
use morph_test2::engine::run_suites;
use morph_test2::types::*;

struct MockBackend;

impl Backend for MockBackend {
    fn analyze_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        Ok(inputs.iter().map(|_| vec![]).collect())
    }
    fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        Ok(inputs.iter().map(|_| vec!["a".into(), "b".into(), "c".into()]).collect())
    }
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn order_does_not_matter_for_lists() {
    morph_test2::i18n::init();
    let suite = TestSuite {
        name: "order".into(),
        cases: vec![TestCase {
            name: "same_set_different_order".into(),
            direction: Direction::Generate,
            input: "X+V".into(),
            expect: vec!["c".into(), "a".into(), "b".into()],
            expect_not: vec![],
        }],
    };
    let backend = MockBackend;
    let summary = run_suites(&backend, &[suite], true);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.failed, 0);
}
