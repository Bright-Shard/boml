# BOML

A dependency-free, (almost) zero-copy TOML parser for Rust.

# Quick Demo

## Parsing

BOML uses the `Toml` struct to parse a TOML file and load its root table. This will either
return the successfully-parsed TOML, or detailed information about a syntax error. If parsing
was successful, the `Toml` struct can essentially be used like a `HashMap` to get values from
the root table.

```rs
use boml::prelude::*;

fn parse_cargo_toml() {
	let source = include_str!("../Cargo.toml");
  let toml = Toml::parse(source).unwrap();
  // If you prefer, `Toml::new()` will do the same thing:
  // let toml = Toml::new(source).unwrap();

  // Get the package table from the `Cargo.toml` file
  let package = toml.get("package").unwrap();
}
```

## Types

BOML stores TOML data in a `TomlValue` enum, with variants for each type. In the example above,
the `package` variable is just storing a `TomlValue`, which could be any TOML type and isn't very
useful to us. We want `package` to be a table. BOML provides several ergonomic ways of doing this:

```rs
// Tables provide `get_<type>` methods, which will return the value as that type if it matches.
// If this method fails, it provides detailed error information, described below.
let package = toml.get_table("package").unwrap();
// All tables provide the `get_<type>` methods, so we can use them to get values from package, too
let name = package.get_string("name").unwrap();
assert_eq!(name, "boml");

// `TomlValue`s can be converted to one of their enum variants - this works similarly to the `.ok()` and
// `.err()` methods on `Result`s.
let package = toml.get("package").unwrap().table().unwrap();
let name = package.get("name").unwrap().string().unwrap();
assert_eq!(name, "boml");

// If you're really dedicated to boilerplate, you can also manually unwrap the enum variant.
let Some(TomlValue::Table(package)) = toml.get("package") else {
  panic!("I love boilerplate");
};
```

BOML also provides a `TomlValueType` enum, allowing you to determine a value's type at runtime. The
`.ty()` method on `TomlValue`s gives you this information:

```rs
let package = toml.get("package").unwrap();
assert_eq!(package.ty(), TomlValueType::Table);
```

## Error Handling

There are 2 sources of errors in BOML: A parsing error, or an error from one of the `get_<type>` methods.
These use the `TomlError` and `TomlGetError` types, respectively.

`TomlError`, the parsing error type, stores the span of text where the parsing error occurred,
and a `TomlErrorKind` which describes the type of error at that span.

`TomlGetError` is an error from one of the `get_<type>` methods in tables. It occurs when there's no value for
the provided key (`InvalidKey`) or when the types aren't the same (`TypeMismatch` - could happen if,
for example, you try to get a `String` value with `get_table`). A `TypeMismatch` error stores
the actual TOML value and its type, so you can attempt to still use it if possible.

# Status/To-Do

The time types (date, time, date-time) aren't of importance to BOML since the
current goal is just to parse `Cargo.toml` files. They will be supported at
some point in the future, but are not right now, hence why it's marked `(future)`.

- [x] Keys
  - [x] Bare keys
  - [x] Quoted keys
  - [x] Dotted keys
- [ ] Values
  - [x] String
    - [x] Basic string
    - [x] Basic multiline string
    - [x] Literal string
    - [x] Literal multiline string
  - [x] Integer
  - [x] Float
  - [x] Boolean
  - [ ] Time (future)
    - [ ] Local Date-Time
    - [ ] Local Date
    - [ ] Local Time
  - [x] Array
- [x] Tables
  - [x] Table
  - [x] Inline Table
  - [x] Array of Tables
  - [x] Array of Inline Tables

# Why "(almost) zero-copy"?

TOML has 2 kinds of strings: basic strings, and literal strings. Literal strings are
just strings BOML can read from the file, but basic strings can have escapes (`\n`,
for example, gets replaced with the newline character). Processing these escapes requires
copying the string, and then replacing the escapes with their actual characters.

BOML will only copy and format a string if the string is a basic string (surrounded by `"`)
*and* actually contains escapes. Literal strings (surrounded by `'`) and basic strings without
escapes are not copied.

# Whatsitstandfor

Yes.