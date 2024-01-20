//! Runs TOML's official test suite: https://github.com/toml-lang/toml-test
//!
//! Assumes git is installed and works.

use {
	boml::prelude::*,
	json::JsonValue,
	std::{env, fs, process::Command},
};

/// Temporarily blacklisted tests that include dates/times.
const BLACKLIST: &[&str] = &[
	"valid/array/array.toml",
	"valid/comment/everywhere.toml",
	"valid/example.toml",
	"valid/spec-example-1.toml",
	"valid/spec-example-1-compact.toml",
	"valid/spec/table-7.toml",
];

#[test]
fn toml_test() {
	// Gets us into boml/target/toml-test/tests
	// the toml-test directory is the cloned toml-test repo
	let mut cwd = env::current_dir().unwrap().join("target");
	if !cwd.exists() {
		fs::create_dir(&cwd).unwrap();
	}
	cwd.push("toml-test");
	if !cwd.exists() {
		let git = Command::new("git")
			.arg("clone")
			.arg("https://github.com/toml-lang/toml-test")
			.current_dir(cwd.parent().unwrap())
			.status();

		if git.is_err() || !git.unwrap().success() {
			panic!("Git failed to clone the test suite");
		}
	}
	cwd.push("tests");

	env::set_current_dir(&cwd).unwrap();
	let git = Command::new("git").arg("pull").status();
	if git.is_err() || !git.unwrap().success() {
		panic!("Git failed to pull the test suite");
	}

	let files = fs::read_to_string("./files-toml-1.0.0").unwrap();
	let mut lines = files.lines().peekable();

	// Statistics
	let mut invalid_tests_passed = 0;
	let mut invalid_tests_failed = 0;
	let mut invalid_tests_skipped = 0;
	let mut tests_failed_to_read = 0;
	let mut valid_tests_passed = 0;
	let mut valid_tests_skipped = 0;

	// Invalid TOML tests
	while let Some(file) = lines.next() {
		if file.contains("date") || file.contains("time") {
			// Time types are TODO
			println!("WARNING: Test skipped due to having time values");
			invalid_tests_skipped += 1;
			continue;
		}

		println!("Testing `{file}`");

		let Ok(input) = fs::read_to_string(file) else {
			println!("WARNING: Failed to read test, skipping");
			tests_failed_to_read += 1;
			continue;
		};
		let toml = Toml::parse(&input);
		// TODO: Enforce invalid toml, ie:
		// assert!(toml.is_err());
		if toml.is_ok() {
			println!("WARNING: Invalid test succeeded");
			invalid_tests_passed += 1;
		} else {
			invalid_tests_failed += 1;
		}

		if !lines.peek().unwrap().contains("invalid") {
			break;
		}
	}

	print!(
		"\
		\n\n\n\
		====== END OF INVALID TESTS, START OF VALID TESTS ======\
		\n\n\n\
		"
	);

	// Valid TOML tests
	while let Some(expectation_file) = lines.next() {
		let file = lines.next().unwrap();
		println!("Testing `{file}`");

		if file.contains("date") || file.contains("time") || BLACKLIST.contains(&file) {
			// Time types are TODO
			println!("WARNING: Test skipped due to having time values");
			valid_tests_skipped += 1;
			continue;
		}

		let expected_response = fs::read_to_string(expectation_file).unwrap();
		let input = fs::read_to_string(file).unwrap();

		let toml = Toml::parse(&input).unwrap();

		let expected_response = json::parse(&expected_response).unwrap();

		assert_json_equals_toml(&expected_response, &TomlValue::Table(toml.into_table()));
		valid_tests_passed += 1;
	}

	println!(
		"\
		\n\n\nTest results:\n\
		Invalid tests passed: {invalid_tests_passed} (these should have failed!)\n\
		Invalid tests failed: {invalid_tests_failed}\n\
		Invalid tests skipped: {invalid_tests_skipped} (these probably had time values)\n\
		\n\
		Valid tests passed: {valid_tests_passed}\n\
		Valid tests failed: 0 (these should have passed!)\n\
		Valid tests skipped: {valid_tests_skipped} (these probably had time values)\n\
		\n\
		Tests that failed to read (probably due to invalid encoding): {tests_failed_to_read}
		"
	);
}

fn assert_json_equals_toml(json: &JsonValue, toml: &TomlValue) {
	if json.is_object() {
		if json.has_key("type") && json.has_key("value") {
			// value
			match json["type"].as_str().unwrap() {
				"integer" => {
					let int: i64 = json["value"].as_str().unwrap().parse().unwrap();
					let toml_int = toml.integer().unwrap();
					assert_eq!(toml_int, int);
				}
				"float" => {
					let float: f64 = json["value"].as_str().unwrap().parse().unwrap();
					let toml_float = toml.float().unwrap();

					if float.is_nan() {
						assert!(toml_float.is_nan());
					} else {
						assert_eq!(toml_float, float);
					}
				}
				"string" => {
					let string = json["value"].as_str().unwrap();
					let toml_string = toml.string().unwrap();
					assert_eq!(toml_string, string);
				}
				"bool" => {
					let bool: bool = json["value"].as_str().unwrap().parse().unwrap();
					let toml_bool = toml.boolean().unwrap();

					assert_eq!(toml_bool, bool);
				}
				_ => unreachable!(),
			}
			return;
		}

		// table
		let toml = toml.table().unwrap();
		for (key, json) in json.entries() {
			let toml = toml.get(key).unwrap();
			assert_json_equals_toml(json, toml);
		}
	} else if json.is_array() {
		// array
		let mut toml = toml.array().unwrap().iter();
		for json in json.members() {
			let toml = toml.next().unwrap();
			assert_json_equals_toml(json, toml);
		}
	}
}
