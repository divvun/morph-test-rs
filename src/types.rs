use serde::Deserialize;
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Analyze,
    Generate,
}
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub direction: Direction,
    pub input: String,
    pub expect: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub cases: Vec<TestCase>,
}
#[derive(Debug, Clone)]
pub struct CaseResult {
    pub name: String,
    pub direction: Direction,
    pub input: String,
    pub expected: Vec<String>,
    pub actual: Vec<String>,
    pub error: Option<String>,
    pub passed: bool,
}
#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub cases: Vec<CaseResult>,
}
