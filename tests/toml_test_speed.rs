use std::{env, fs};

/// Run `cargo t toml_test_speed -- -Zunstable-options --report-time` to see
/// how long it takes BOML to parse the entire toml test suite.
/// This time will be somewhat bloated by the time it takes to load all the
/// files in the test suite.
#[ignore]
#[test]
fn toml_test_speed() {
	let cwd = env::current_dir()
		.unwrap()
		.join("target")
		.join("toml-test")
		.join("tests");
	env::set_current_dir(&cwd).unwrap();

	let files = fs::read_to_string("./files-toml-1.0.0").unwrap();
	let mut lines = files.lines().peekable();

	// Invalid TOML tests
	while let Some(file) = lines.next() {
		let Ok(input) = fs::read_to_string(file) else {
			continue;
		};
		let _toml = boml::parse(&input);

		if !lines.peek().unwrap().contains("invalid") {
			break;
		}
	}

	// Valid TOML tests
	while let Some(_expectation_file) = lines.next() {
		let file = lines.next().unwrap();
		let input = fs::read_to_string(file).unwrap();
		let _toml = boml::parse(&input);
	}
}
