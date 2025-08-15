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
    pub total: usize,  // Total number of test cases
    pub passed: usize, // Number of passed test cases
    pub failed: usize, // Number of failed test cases
    pub cases: Vec<CaseResult>,

    // Expectation-level counts for more granular reporting
    pub total_expectations: usize,
    pub passed_expectations: usize,
    pub failed_expectations: usize,
}
