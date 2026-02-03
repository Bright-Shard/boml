use {
	boml::{
		prelude::*,
		types::{TomlDate, TomlOffset, TomlTime},
	},
	json::JsonValue,
	std::{env, fs},
};

/// Run BOML against the official TOML test suite. This test fails if any valid
/// TOML tests fail. It ignores any invalid TOML tests that pass, since these
/// are currently quite pedantic and I'm not too worried about passing them.
///
/// To see what TOML tests passed/failed, run this test without capturing its
/// `stdout`:
/// ```
/// cargo t toml_test -- --nocapture
/// ```
#[test]
fn toml_test() {
	enter_toml_test_folder();

	let files = fs::read_to_string("./files-toml-1.1.0").unwrap();
	let mut lines = files.lines().peekable();

	// Statistics
	let mut invalid_tests_passed = 0;
	let mut invalid_tests_failed = 0;
	let mut tests_failed_to_read = 0;
	let mut valid_tests_passed = 0;
	let mut valid_tests_failed = 0;

	// Invalid TOML tests
	while let Some(file) = lines.next() {
		let Ok(input) = fs::read_to_string(file) else {
			println!("WARNING: Failed to read test `{file}`, skipping");
			tests_failed_to_read += 1;
			continue;
		};
		let toml = boml::parse(&input);

		if toml.is_ok() {
			println!("WARNING: Invalid test `{file}` succeeded");
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

		let expected_response = fs::read_to_string(expectation_file).unwrap();
		let input = fs::read_to_string(file).unwrap();

		let toml = match boml::parse(&input) {
			Ok(toml) => toml,
			Err(err) => {
				println!("ERROR: Valid test `{file}` failed: {err:?}");
				valid_tests_failed += 1;
				continue;
			}
		};

		let expected_response = json::parse(&expected_response).unwrap();

		let val = TomlValue::Table(toml.into());
		if json_equals_toml(&expected_response, &val, file) {
			valid_tests_passed += 1;
		} else {
			println!("ERROR: JSON != TOML:\n{expected_response}\n//\n{val:#?}");
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

	if valid_tests_failed > 0 {
		panic!();
	}
}

/// Get a rough estimate of boml's performance by parsing the entire TOML test
/// suite. Note that this time gets affected by lots of other things (e.g.
/// reading files from disk), so it's just a ballpark performance measure.
///
/// Run this test with:
/// ```
/// cargo +nightly t toml_test_speed -- -Zunstable-options --report-time --include-ignored
/// ```
///
/// You can find out how many lines of code are in the test suite with tokei,
/// e.g. in the `toml-test` folder:
/// ```
/// cat files-toml-1.1.0 | grep '.toml' | xargs tokei
/// ```
#[ignore]
#[test]
fn toml_test_speed() {
	let cwd = env::current_dir()
		.unwrap()
		.join("target")
		.join("toml-test")
		.join("tests");
	env::set_current_dir(&cwd).unwrap();

	let files = fs::read_to_string("./files-toml-1.1.0").unwrap();
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

/// Ensure the `toml-test` test suite is downloaded, then set our current
/// directory to that folder so we can run its tests.
fn enter_toml_test_folder() {
	let toml_test_folder = env::current_dir().unwrap().join("toml-test");
	if !toml_test_folder.exists() {
		panic!(
			"You need to update the `toml-test` git submodule so boml's tests can access it. You can do this with:\n\tgit submodule update --init\n\nAfter running that command, you'll see a new `toml-test` directory, which has TOML's official test suite. You can then rerun boml's tests and boml will run the official test suite."
		)
	}

	env::set_current_dir(toml_test_folder.join("tests")).unwrap();
}

/// The TOML test suite works by having a TOML file and then the same data
/// encoded in JSON. To pass the valid tests, you compare what you parsed from
/// the TOML file to what an official JSON parser parsed. This function handles
/// that by comparing `boml` to the `json` crate.
fn json_equals_toml(json: &JsonValue, toml: &TomlValue, test_file: &str) -> bool {
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
							format!(".{:.3}", nanosecond.to_string()).trim_end_matches('0');
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
							format!(".{:.3}", nanosecond.to_string()).trim_end_matches('0');
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
						let ns = format!(".{:.3}", nanosecond.to_string());
						// yes, this one test is quirky and formats the ns differently
						if test_file == "valid/datetime/milliseconds.toml" {
							formatted += ns.as_str();
						} else {
							formatted += ns.trim_end_matches('0')
						}
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
				if !json_equals_toml(json, toml, test_file) {
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
			if !json_equals_toml(json, toml, test_file) {
				return false;
			}
		}

		true
	} else {
		unreachable!()
	}
}
