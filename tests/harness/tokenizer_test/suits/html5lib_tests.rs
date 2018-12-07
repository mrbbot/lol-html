use crate::harness::tokenizer_test::TestCase;
use glob;
use serde_json::from_reader;

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

pub fn get_test_cases() -> Vec<TestCase> {
    let mut tests = Vec::new();

    for file in read_test_data!("html5lib-tests/tokenizer/*.test") {
        tests.extend(from_reader::<_, Suite>(file).unwrap().tests);
    }

    tests
}
