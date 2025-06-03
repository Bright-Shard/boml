//! Runs TOML's official test suite: https://github.com/toml-lang/toml-test
//!
//! To run: `cargo t toml_test -- --nocapture`
//!
//! Assumes git is installed and works.

use {
	boml::{
		prelude::*,
		types::{TomlDate, TomlOffset, TomlTime},
	},
	json::JsonValue,
	std::{env, fs, process::Command},
};

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
		println!("[WARN] Git failed to pull the test suite - tests may be out of date");
	}

	let files = fs::read_to_string("./files-toml-1.0.0").unwrap();
	let mut lines = files.lines().peekable();

	// Statistics
	let mut invalid_tests_passed = 0;
	let mut invalid_tests_failed = 0;
	let mut tests_failed_to_read = 0;
	let mut valid_tests_passed = 0;
	let mut valid_tests_failed = 0;

	// Invalid TOML tests
	while let Some(file) = lines.next() {
		println!("Testing `{file}`");

		let Ok(input) = fs::read_to_string(file) else {
			println!("WARNING: Failed to read test, skipping");
			tests_failed_to_read += 1;
			continue;
		};
		let toml = boml::parse(&input);

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

		let expected_response = fs::read_to_string(expectation_file).unwrap();
		let input = fs::read_to_string(file).unwrap();

		let toml = match boml::parse(&input) {
			Ok(toml) => toml,
			Err(err) => {
				println!("WARNING: Test failed: {err:?}");
				valid_tests_failed += 1;
				continue;
			}
		};

		let expected_response = json::parse(&expected_response).unwrap();

		let val = TomlValue::Table(toml.into());
		if json_equals_toml(&expected_response, &val) {
			valid_tests_passed += 1;
		} else {
			println!("WARNING: JSON != TOML:\n{expected_response}\n//\n{val:#?}");
			valid_tests_failed += 1;
		}
	}

	println!(
		"\
		\n\n\nTOML test suite results:\n\
		Invalid tests passed: {invalid_tests_passed} (these should have failed!)\n\
		Invalid tests failed: {invalid_tests_failed}\n\
		\n\
		Valid tests passed: {valid_tests_passed}\n\
		Valid tests failed: {valid_tests_failed} (these should have passed!)\n\
		\n\
		Tests that failed to read (probably due to invalid encoding): {tests_failed_to_read}
		"
	);
}

fn json_equals_toml(json: &JsonValue, toml: &TomlValue) -> bool {
	if json.is_object() {
		if json.has_key("type") && json.has_key("value") {
			// value
			match json["type"].as_str().unwrap() {
				"integer" => {
					let int: i64 = json["value"].as_str().unwrap().parse().unwrap();
					let toml_int = toml.as_integer().unwrap();
					toml_int == int
				}
				"float" => {
					let float: f64 = json["value"].as_str().unwrap().parse().unwrap();
					let toml_float = toml.as_float().unwrap();

					if float.is_nan() {
						toml_float.is_nan()
					} else {
						toml_float == float
					}
				}
				"string" => {
					let string = json["value"].as_str().unwrap();
					let toml_string = toml.as_string().unwrap();
					toml_string == string
				}
				"bool" => {
					let bool: bool = json["value"].as_str().unwrap().parse().unwrap();
					let toml_bool = toml.as_bool().unwrap();

					toml_bool == bool
				}
				"date-local" => {
					let date = json["value"].as_str().unwrap();

					let toml_date = toml.as_date().unwrap();
					let TomlDate {
						year,
						month,
						month_day,
					} = toml_date;

					let formatted = format!("{year:04}-{month:02}-{month_day:02}");
					formatted.as_str() == date
				}
				"time-local" => {
					let time = json["value"].as_str().unwrap();

					let toml_time = toml.as_time().unwrap();
					let TomlTime {
						hour,
						minute,
						second,
						nanosecond,
					} = toml_time;

					let mut formatted = format!("{hour:02}:{minute:02}:{second:02}");
					if nanosecond > 0 {
						formatted +=
							&format!(".{:0<3}", nanosecond.to_string().trim_end_matches('0'));
					}

					formatted.as_str() == time
				}
				"datetime-local" => {
					let datetime = json["value"].as_str().unwrap();

					let toml_datetime = toml.as_datetime().unwrap();
					let TomlDate {
						year,
						month,
						month_day,
					} = toml_datetime.date;
					let TomlTime {
						hour,
						minute,
						second,
						nanosecond,
					} = toml_datetime.time;

					let mut formatted = format!(
						"{year:04}-{month:02}-{month_day:02}T{hour:02}:{minute:02}:{second:02}"
					);
					if nanosecond > 0 {
						formatted +=
							&format!(".{:0<3}", nanosecond.to_string().trim_end_matches('0'));
					}

					formatted.as_str() == datetime
				}
				"datetime" => {
					let datetime = json["value"].as_str().unwrap();

					let toml_datetime = toml.as_offset_datetime().unwrap();
					let TomlOffset {
						hour: offset_hour,
						minute: offset_minute,
					} = toml_datetime.offset;
					let TomlDate {
						year,
						month,
						month_day,
					} = toml_datetime.date;
					let TomlTime {
						hour,
						minute,
						second,
						nanosecond,
					} = toml_datetime.time;

					let mut formatted = format!(
						"{year:04}-{month:02}-{month_day:02}T{hour:02}:{minute:02}:{second:02}"
					);
					if nanosecond > 0 {
						formatted +=
							&format!(".{:0<3}", nanosecond.to_string().trim_end_matches('0'));
					}
					if offset_hour == 0 && offset_minute == 0 {
						formatted.push('Z');
					} else {
						if offset_hour >= 0 {
							formatted.push('+');
						} else {
							formatted.push('-');
						}
						formatted +=
							&format!("{:02}:{offset_minute:02}", offset_hour.unsigned_abs());
					}

					formatted.as_str() == datetime
				}
				other => unreachable!("{other}"),
			}
		} else {
			// table
			let toml = toml.as_table().unwrap();
			for (key, json) in json.entries() {
				let Some(toml) = toml.get(key) else {
					return false;
				};
				if !json_equals_toml(json, toml) {
					return false;
				}
			}

			true
		}
	} else if json.is_array() {
		// array
		let mut toml = toml.as_array().unwrap().iter();
		for json in json.members() {
			let Some(toml) = toml.next() else {
				return false;
			};
			if !json_equals_toml(json, toml) {
				return false;
			}
		}

		true
	} else {
		unreachable!()
	}
}
