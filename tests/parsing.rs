use boml::prelude::*;

/// Test that boml can parse booleans and bare keys.
#[test]
fn bools_and_bare_keys() {
	let toml_source = concat!(
		"val1 = true\n",
		"val2 = false\n",
		"5678 = true\n",
		"dash-ed = true\n",
		"under_score = true\n"
	);
	let toml = Toml::parse(toml_source).unwrap();
	toml.assert_values(
		vec![
			("val1", true),
			("val2", false),
			("5678", true),
			("dash-ed", true),
			("under_score", true),
		]
		.into_iter()
		.map(|(k, v)| (k, TomlValue::Boolean(v)))
		.collect(),
	);
}

/// Test that boml can parse quoted keys.
#[test]
fn quoted_keys() {
	let toml_source = concat!(
		"'val0.1.1' = true\n",
		"'ʎǝʞ' = true\n",
		"\"quoted 'key'\" = true\n",
		"'quoted \"key\" 2' = true\n",
	);
	let toml = Toml::parse(toml_source).unwrap();
	toml.assert_values(
		vec![
			("val0.1.1", true),
			("ʎǝʞ", true),
			("quoted 'key'", true),
			("quoted \"key\" 2", true),
		]
		.into_iter()
		.map(|(k, v)| (k, TomlValue::Boolean(v)))
		.collect(),
	);
}

/// Test that boml can handle dotted keys.
#[test]
fn dotted_keys() {
	let toml_source = concat!(
		"table.bool = true\n",
		"table.string = 'hi'\n",
		"table. spaced = 69\n",
		"table  .infinity = -inf\n",
	);
	let toml = Toml::parse(toml_source).unwrap();

	let table = toml.get_table("table").unwrap();
	assert!(table.get_boolean("bool").unwrap());
	assert_eq!(table.get_string("string").unwrap(), "hi");
	assert_eq!(table.get_integer("spaced").unwrap(), 69);
	assert_eq!(table.get_float("infinity").unwrap(), -f64::INFINITY);
}

/// Test that boml can parse literal strings and multiline literal strings.
#[test]
fn literal_strings() {
	let single = "Me when I have to write a demo sentence to test my incredible TOML parser but dunno what to say";
	let multi = "Bruhhhh I gotta write\n*another*\ndemo sentence???\n:(";
	let toml_source = format!("single = '{single}'\n") + &format!("multi = '''{multi}'''");
	let toml = Toml::parse(&toml_source).unwrap();
	toml.assert_strings(vec![("single", single), ("multi", multi)]);
}

/// Test that boml can parse basic strings and multiline basic strings.
#[test]
fn basic_strings() {
	let toml_source = concat!(
		"normal = \"normality 100\"\n",
		r#"quotes = "Bro I got \"quotes\"" "#,
		"\n",
		r#"escapes = "\t\n\r\\" "#,
		"\n",
		"multi = \"\"\"me when\\n",
		"i do multiline\\r pretty neat",
		"\"\"\"\n",
		"whitespace = \"\"\"white\\    \n\n\n\r\n    space\"\"\""
	);
	let toml = Toml::parse(toml_source).unwrap();
	toml.assert_strings(vec![
		("normal", "normality 100"),
		("quotes", "Bro I got \"quotes\""),
		("escapes", "\t\n\r\\"),
		("multi", "me when\ni do multiline\r pretty neat"),
		("whitespace", "whitespace"),
	]);
}

/// Test that boml can parse integers.
#[test]
fn integers() {
	let toml_source = concat!(
		"hex = 0x10\n",
		"decimal = 10\n",
		"octal = 0o10\n",
		"binary = 0b10\n",
		"neghex = -0x10\n",
		"posoctal = +0o10\n",
		"lmao = -0\n",
		"underscore = 10_00\n",
		"single = 2\n",
	);
	let toml = Toml::parse(toml_source).unwrap();
	toml.assert_values(
		vec![
			("hex", 16),
			("decimal", 10),
			("octal", 8),
			("binary", 2),
			("neghex", -16),
			("posoctal", 8),
			("lmao", 0),
			("underscore", 1000),
			("single", 2),
		]
		.into_iter()
		.map(|(k, v)| (k, TomlValue::Integer(v)))
		.collect(),
	);
}

/// Test that boml can parse floats.
#[test]
fn floats() {
	let toml_source = concat!(
		"fractional = 0.345\n",
		"exponential = 4e2\n",
		"exponential_neg = 4e-2\n",
		"exponential_pos = 4e+2\n",
		"pos_fractional = +0.567\n",
		"neg_fractional = -0.123\n",
		"capital_exponential = 2E2\n",
		"combined = 7.27e2\n",
		"nan = +nan\n",
		"infinity = -inf\n",
		"underscore = 10_00.0\n",
	);

	let toml = Toml::parse(toml_source).unwrap();
	toml.assert_values(
		vec![
			("fractional", 0.345),
			("exponential", 4e2),
			("exponential_neg", 4e-2),
			("exponential_pos", 4e2),
			("pos_fractional", 0.567),
			("neg_fractional", -0.123),
			("capital_exponential", 2e2),
			("combined", 727.0),
			("infinity", -f64::INFINITY),
			("underscore", 1000.0),
		]
		.into_iter()
		.map(|(key, val)| (key, TomlValue::Float(val)))
		.collect(),
	);

	// NaN != NaN, so we have to check with the `is_nan()` method.
	let nan = toml.get_float("nan");
	assert!(nan.is_ok());
	assert!(nan.unwrap().is_nan())
}

/// Test that boml can parse tables.
#[test]
fn tables() {
	let toml_source = concat!(
		"empty = {}\n",
		"inline = { name = 'inline', num = inf }\n",
		"\n",
		"[table1]\n",
		"name = 'table1'\n",
		"\n",
		"[table2]\n",
		"name = 'table2'\n",
		"num = 420\n",
		"\n",
		"[table3]\n",
		"array = ['hi', 'bye']\n",
		"array2 = [1]\n",
	);
	let toml = Toml::parse(toml_source).unwrap();

	let _empty = toml.get_table("empty").unwrap();

	let inline = toml.get_table("inline").unwrap();
	assert_eq!(inline.get_string("name"), Ok("inline"));
	assert_eq!(inline.get_float("num"), Ok(f64::INFINITY));

	let table1 = toml.get_table("table1").unwrap();
	assert_eq!(table1.get_string("name"), Ok("table1"));

	let table2 = toml.get_table("table2").unwrap();
	assert_eq!(table2.get_string("name"), Ok("table2"));
	assert_eq!(table2.get_integer("num"), Ok(420));

	let table3 = toml.get_table("table3").unwrap();
	let array = table3.get_array("array").unwrap();
	let array2 = table3.get_array("array2").unwrap();
	assert_eq!(array.len(), 2);
	assert_eq!(array2.len(), 1);
	assert_eq!(array.first().unwrap().string().unwrap(), "hi");
	assert_eq!(array.get(1).unwrap().string().unwrap(), "bye");
	assert_eq!(array2.first().unwrap().integer().unwrap(), 1);
}

/// Test that boml can parse arrays.
#[test]
fn arrays() {
	let toml_source = concat!(
		"strings = ['hi', 'hello', 'how are you']\n",
		"nested = ['me', ['when i', 'nest'], 'arrays']\n",
		"tables = [{name = 'bruh'}, {name = 'bruh 2 electric boogaloo'}]\n",
		"single = [2]\n"
	);
	// panic!("`{}`", &toml_source[160..=163]);
	let toml = Toml::parse(toml_source).unwrap();

	let strings = toml.get_array("strings").unwrap();
	let strings: Vec<&str> = strings.iter().map(|val| val.string().unwrap()).collect();
	assert_eq!(strings, vec!["hi", "hello", "how are you"]);

	let mut nested = toml.get_array("nested").unwrap().iter();
	assert_eq!(nested.next().unwrap().string().unwrap(), "me");
	let mut subtable = nested.next().unwrap().array().unwrap().iter();
	assert_eq!(subtable.next().unwrap().string().unwrap(), "when i");
	assert_eq!(subtable.next().unwrap().string().unwrap(), "nest");
	assert_eq!(nested.next().unwrap().string().unwrap(), "arrays");

	let mut tables = toml.get_array("tables").unwrap().iter();
	let table1 = tables.next().unwrap().table().unwrap();
	assert_eq!(table1.get_string("name").unwrap(), "bruh");
	let table2 = tables.next().unwrap().table().unwrap();
	assert_eq!(
		table2.get_string("name").unwrap(),
		"bruh 2 electric boogaloo"
	);

	let single = toml.get_array("single").unwrap();
	assert_eq!(single.len(), 1);
	assert_eq!(single.first().unwrap().integer().unwrap(), 2);
}

/// Test that boml can parse array tables.
#[test]
fn array_tables() {
	let toml_source = concat!(
		"[[entry]]\n",
		"idx = 0\n",
		"value = 'HALLO'\n",
		"\n",
		"[[entry]]\n",
		"idx = 1\n",
		"value = 727\n",
		"\n",
		"[[entry]]\n",
		"idx = 2\n",
		"value = true\n",
	);
	let toml = Toml::parse(toml_source).unwrap();

	let entries = toml.get_array("entry").unwrap();

	let first = entries[0].table().unwrap();
	assert_eq!(first.get_integer("idx").unwrap(), 0);
	assert_eq!(first.get_string("value").unwrap(), "HALLO");

	let second = entries[1].table().unwrap();
	assert_eq!(second.get_integer("idx").unwrap(), 1);
	assert_eq!(second.get_integer("value").unwrap(), 727);

	let third = entries[2].table().unwrap();
	assert_eq!(third.get_integer("idx").unwrap(), 2);
	assert!(third.get_boolean("value").unwrap());
}

/// Test that boml works with weird formats - CRLF, weird spacings, etc.
#[test]
fn weird_formats() {
	let toml_source = concat!(
		"   val1 = true\r\n",
		"val2=      false",
		"\n\r\n\r\n\n",
		"val3  =true\n",
		"val4=false\n",
		"val5 = true      \n",
		"[parent .  \"child.dotted\"]\n",
		"yippee = true"
	);
	let toml = Toml::new(toml_source).unwrap();
	toml.assert_values(
		vec![
			("val1", true),
			("val2", false),
			("val3", true),
			("val4", false),
			("val5", true),
		]
		.into_iter()
		.map(|(k, v)| (k, TomlValue::Boolean(v)))
		.collect(),
	);

	let parent = toml.get_table("parent").unwrap();
	let child = parent.get_table("child.dotted").unwrap();
	assert!(child.get_boolean("yippee").unwrap());
}

trait TomlTestUtils {
	fn assert_value(&self, key: &str, expected_value: TomlValue<'_>);
	fn assert_values(&self, expected_values: Vec<(&str, TomlValue<'_>)>);
	fn assert_strings(&self, strings: Vec<(&str, &str)>);
}

impl TomlTestUtils for Toml<'_> {
	#[inline]
	fn assert_value(&self, key: &str, expected_value: TomlValue<'_>) {
		assert_eq!(*self.get(key).unwrap(), expected_value);
	}
	#[inline]
	fn assert_values(&self, expected_values: Vec<(&str, TomlValue<'_>)>) {
		for (key, expected_value) in expected_values {
			self.assert_value(key, expected_value);
		}
	}
	fn assert_strings(&self, strings: Vec<(&str, &str)>) {
		for (key, expected_string) in strings {
			let value = self.get_string(key);
			assert!(value.is_ok());
			assert_eq!(value.unwrap(), expected_string);
		}
	}
}
