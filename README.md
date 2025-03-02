# BOML

A dead-simple, efficient, dependency-free TOML parser for Rust.

*Special thanks to [cubic](https://github.com/ucubic) and [Speykious](https://github.com/speykious) for their help while developing BOML!*



# Usage

## Parsing

BOML requires no imports - just call `boml::parse` with the TOML source code, then use the returned `Toml` value similarly to a hashmap:

```rust
fn parse_cargo_toml() {
	let source = include_str!("../Cargo.toml");
	let toml = boml::parse(source).unwrap();

	// Get the package table from the `Cargo.toml` file
	let package = toml.get_table("package").unwrap();
}
```

## Data Types

In the above snippet, we used `get_table` to read a value, because we knew the value was a table. BOML provides equivalent methods for every TOML type (`get_string`, `get_integer`, etc). However, BOML also provides a `get` method, which allows you to resolve the type yourself. For example:

```rust
use boml::prelude::TomlValue;

let source = include_str!("../Cargo.toml");
let toml = boml::parse(source).unwrap();

// Specific types via `.get_<type>`
let package = toml.get_table("package").unwrap();
let name = package.get_string("name").unwrap();
assert_eq!(name, "boml");

// Dynamic types via `.get`
let package_untyped = toml.get("package").unwrap();
// You can then convert the value to a specific type with helper methods or
// pattern matching:
let package = toml.get("package").unwrap().as_table().unwrap();
let name = package.get("name").unwrap().as_string().unwrap();
assert_eq!(name, "boml");

let Some(TomlValue::Table(package)) = toml.get("package") else {
    panic!("I love boilerplate, why would anyone use helper methods");
};
match package.get("name").unwrap() {
	TomlValue::String(name) => assert_eq!(name.as_str(), "boml"),
	TomlValue::Integer(int) => println!("{int} is a pretty weird name, bro"),
	_ => panic!("Expected string or int for package name")
}
```

You can also determine a value's type without touching its data, via the `.ty()` method and `TomlValueType` enum:

```rust
use boml::prelude::TomlValueType;

let source = include_str!("../Cargo.toml");
let toml = boml::parse(source).unwrap();

let package = toml.get("package").unwrap();
assert_eq!(package.ty(), TomlValueType::Table);
```

## Error Handling

There are 2 sources of errors in BOML: A parsing error, or an error from one of
the `get_<type>` methods. These use the `TomlError` and `TomlGetError` types,
respectively.

`TomlError`, the parsing error type, stores the span of text where the parsing
error occurred, and a `TomlErrorKind` which describes the type of error at that
span. Printing the error will show the error kind and the region of TOML that had the error.

`TomlGetError` is an error from one of the `get_<type>` methods in tables. It
occurs when there's no value for the provided key (`InvalidKey`) or when the
types aren't the same (`TypeMismatch` - could happen if, for example, you try
to get a `String` value with `get_table`). A `TypeMismatch` error stores the
actual TOML value and its type, so you can still attempt to use it if possible.



# Date and Time Types

TOML supports 4 data types related to dates and times. BOML will parse these types, but performs no validation on those date and time types, because [time is hard](https://gist.github.com/timvisee/fcda9bbdff88d45cc9061606b4b923ca). BOML does not even check if an hour is between 0 and 23 or if a minute is between 0 and 60.

The *only* guarantee BOML makes about date/time values is that they are formatted according to [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). This does not mean the date/time values are actually valid. It just means that the year was a four-digit number, the month was a two-digit number, etc.

You should pass date/time values parsed with BOML to another crate - such as [chrono](https://docs.rs/chrono/latest/chrono/) or [jiff](https://docs.rs/jiff/latest/jiff/) - before actually using them.

If you enable the crate feature `chrono`, BOML will provide `From` and `Into` implementations to convert TOML date/time types into Chrono date/time types.



# TOML Compliance

BOML passes all valid tests cases of the [official TOML test suite](https://github.com/toml-lang/toml-test) for TOML 1.0.

BOML does parse some invalid test cases without erroring, meaning it may parse something that's technically invalid TOML as valid TOML.

To run BOML against the TOML test suite yourself, see [tests/toml_test.rs](tests/toml_test.rs).

TOML 1.1 is not currently supported, but support for it will be added if it's released.



# Efficiency

BOML aims to be very fast. On a Framework 16, BOML parses the entire TOML test suite - ~1.8k lines of TOML - in ~.003 seconds. You can run this benchmark yourself with `cargo +nightly t toml_test_speed --release -- -Zunstable-options --report-time --ignored`.

Here's some more in-depth details on BOML's efficiency:

- BOML only copies data from the original TOML source string in two places:
	1. Strings with escape sequences. Parts of the string that aren't escaped have to be copied to a new string, so the escape can be added.
	2. Floats. Floats are really hard to parse correctly (and efficiently), so BOML uses the standard library's float parser. Unfortunately TOML allows underscores in floats, while the standard library does not, so BOML first has to copy the float to a new buffer and remove any underscores. This process does not allocate.
- BOML only allocates memory in three places:
	1. Creating hashmaps for TOML tables
	2. Creating vecs for TOML arrays
	3. Copying strings with escape sequences to process the escape sequences


# To-Do

- Support for serializing TOML
- `no_std` support? Currently only allocated types from the standard library are used, so it should be possible
- Improve error messages to be more like rustc or <https://github.com/brendanzab/codespan>



# So what does "BOML" stand for

Yes.
