#[test]
pub fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/input.rs");
}