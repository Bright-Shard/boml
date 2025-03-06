use std::collections::HashMap;

use boml::prelude::*;

#[test]
fn test_derive_struct_named() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test {
		foo: i64,
		bar: String,
	}

	let toml = r#"
        foo = 42
        bar = "hello"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test {
			foo: 42,
			bar: "hello".to_string()
		},
		actual.unwrap()
	);
}

#[test]
fn test_derive_struct_unnamed() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test(i64, String);

	let toml = r#"
        0 = 42
        1 = "hello"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test(42, "hello".to_string()), actual.unwrap());
}

#[test]
fn test_derive_struct_unit() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test;

	let toml = r#""#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test, actual.unwrap());
}

#[test]
fn test_derive_lifetimes() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test<'a, 'b: 'a, 'c> {
		foo: &'a str,
		bar: &'b str,
		baz: &'c str,
	}

	let toml = r#"
        foo = "hello"
        bar = "world"
        baz = "!"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test {
			foo: "hello",
			bar: "world",
			baz: "!"
		},
		actual.unwrap()
	);
}

#[test]
fn test_derive_generics() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test<T, U> {
		foo: T,
		bar: U,
	}

	let toml = r#"
        foo = 42
        bar = "hello world"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test {
			foo: 42,
			bar: "hello world"
		},
		actual.unwrap()
	);
}

#[test]
fn test_derive_generics_lifetimes() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test<'a, T> {
		foo: T,
		bar: &'a str,
	}

	let toml = r#"
        foo = 42
        bar = "hello world"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test {
			foo: 42,
			bar: "hello world"
		},
		actual.unwrap()
	);
}

#[test]
fn test_derive_nesting() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Inner {
		foo: i64,
		bar: String,
	}

	#[derive(FromToml, Debug, PartialEq)]
	struct Outer {
		foo: i64,
		inner: Inner,
	}

	let toml = r#"
        foo = 69
        [inner]
        foo = 42
        bar = "hello"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Outer::from_toml(Some(&v));
	let expected = Outer {
		foo: 69,
		inner: Inner {
			foo: 42,
			bar: "hello".to_string(),
		},
	};
	assert!(actual.is_ok());
	assert_eq!(expected, actual.unwrap());
}

#[test]
fn test_derive_option() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test<'a> {
		a: i64,
		b: Option<&'a str>,
	}

	let toml = r#"
        a = 42
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test { a: 42, b: None }, actual.unwrap());
}

#[test]
fn test_derive_vec() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test {
		a: Vec<i64>,
		b: Vec<String>,
	}

	let toml = r#"
        a = [1, 2, 3]
        b = ["hello", "world"]
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	let expected = Test {
		a: vec![1, 2, 3],
		b: vec!["hello".to_string(), "world".to_string()],
	};
	assert!(actual.is_ok());
	assert_eq!(expected, actual.unwrap());
}

#[test]
fn test_derive_map() {
	#[derive(FromToml, Debug, PartialEq)]
	struct Test<'a> {
		a: HashMap<&'a str, i64>,
		b: HashMap<&'a str, String>,
	}

	let toml = r#"
        a = { one = 1, two = 2, three = 3 }
        b = { hello = "world", foo = "bar" }
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	let mut a = HashMap::new();
	a.insert("one", 1);
	a.insert("two", 2);
	a.insert("three", 3);

	let mut b = HashMap::new();
	b.insert("hello", "world".to_string());
	b.insert("foo", "bar".to_string());

	let expected = Test { a, b };

	assert!(actual.is_ok());
	assert_eq!(expected, actual.unwrap());
}

#[test]
fn test_derive_enum() {
	#[derive(FromToml, Debug, PartialEq)]
	enum Test<'a> {
		A(i64),
		B { foo: i64, bar: &'a str },
		C,
	}

	let toml = r#"
        [A]
        0 = 42
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::A(42), actual.unwrap());

	let toml = r#"
        [B]
        foo = 69
        bar = "hello world"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test::B {
			foo: 69,
			bar: "hello world"
		},
		actual.unwrap()
	);

	let toml = r#"
        [C]
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::C, actual.unwrap());
}

#[test]
fn test_derive_enum_tag_internal() {
	#[derive(FromToml, Debug, PartialEq)]
	#[boml(tag = "type")]
	enum Test<'a> {
		A(i64),
		B { foo: i64, bar: &'a str },
		C,
	}

	let toml = r#"
        type = "A"
        0 = 42
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::A(42), actual.unwrap());

	let toml = r#"
        type = "B"
        foo = 69
        bar = "hello world"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test::B {
			foo: 69,
			bar: "hello world"
		},
		actual.unwrap()
	);

	let toml = r#"
        type = "C"
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::C, actual.unwrap());
}

#[test]
fn test_derive_enum_tag_adjacent() {
	#[derive(FromToml, Debug, PartialEq)]
	#[boml(tag = "type", content = "content")]
	enum Test<'a> {
		A(i64),
		B { foo: i64, bar: &'a str },
		C,
	}

	let toml = r#"
        type = "A"
        content = { "0" = 42 }
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::A(42), actual.unwrap());

	let toml = r#"
        type = "B"
        content = { foo = 69, bar = "hello world" }
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(
		Test::B {
			foo: 69,
			bar: "hello world"
		},
		actual.unwrap()
	);

	let toml = r#"
        type = "C"
        content = {}
    "#;
	let toml = boml::parse(toml).unwrap();
	let v = TomlValue::Table(toml.into());
	let actual = Test::from_toml(Some(&v));

	assert!(actual.is_ok());
	assert_eq!(Test::C, actual.unwrap());
}
