// tests/order_insensitive.rs
use morph_test::types::*;
use morph_test::engine::run_suites;
use morph_test::backend::Backend;
use anyhow::Result;
struct MockBackend;
impl Backend for MockBackend {
    fn analyze(&self, _input: &str) -> Result<Vec<String>> { Ok(vec![]) }
    fn generate(&self, _input: &str) -> Result<Vec<String>> {
        Ok(vec!["a".into(), "b".into(), "c".into()])
    }
}
#[test]
fn order_does_not_matter_for_lists() {
    let suite = TestSuite {
        name: "order".into(),
        cases: vec![
            TestCase {
                name: "same_set_different_order".into(),
                direction: Direction::Generate,
                input: "X+V".into(),
                expect: vec!["c".into(), "a".into(), "b".into()],
            },
        ],
    };
    let backend = MockBackend;
    let summary = run_suites(&backend, &[suite]);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.failed, 0);
}
