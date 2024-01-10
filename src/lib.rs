pub mod parser;
pub mod text;
pub mod value;

use {
    std::{collections::HashMap, ops::Deref},
    text::Text,
    value::Value,
};

pub struct TOML<'a> {
    pub text: &'a str,
    values: HashMap<&'a str, Value<'a>>,
}
impl<'a> TOML<'a> {
    #[inline(always)]
    pub fn new(text: &'a str) -> Result<Self, Error> {
        Self::parse(text)
    }

    pub fn parse(text: &'a str) -> Result<Self, Error> {
        let mut text = Text { text, idx: 0 };
        let mut values = HashMap::new();

        while text.idx < text.len() - 1 {
            println!(
                "Main loop is at: |{}|",
                text.byte(text.idx).unwrap().to_owned() as char
            );
            match text.byte(text.idx).unwrap() {
                // Whitespace
                b' ' | b'\n' | b'\r' => text.idx += 1,
                // Comment
                b'#' => {
                    todo!()
                }
                // Table definition
                b'[' => {
                    todo!()
                }
                // Quoted key definition
                b'\'' | b'\"' => {
                    let key = parser::parse_quoted_key(&mut text)?;
                    let value = parser::parse_value(&mut text)?;
                    values.insert(key.to_str(), value);
                }
                // Bare key definition
                _ => {
                    let key = parser::parse_bare_key(&mut text)?;
                    let value = parser::parse_value(&mut text)?;
                    values.insert(key.to_str(), value);
                }
            }
        }

        let values = values.into_iter().map(|(k, v)| (k, v.value)).collect();

        Ok(Self {
            text: text.text,
            values,
        })
    }
}
impl<'a> Deref for TOML<'a> {
    type Target = HashMap<&'a str, Value<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

#[derive(Debug)]
pub struct Error {
    pub start: usize,
    pub end: usize,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// A bare key (key without quotes) contains an invalid character.
    InvalidBareKey,
    /// There was a space in the middle of a bare key.
    BareKeyHasSpace,
    /// A quoted key didn't have a closing quote.
    UnclosedQuotedKey,
    /// There was no `=` sign in a key/value assignment.
    NoEqualsInAssignment,
    /// There was no key in a key/value assignment.
    NoKeyInAssignment,
    /// There was no value in a key/value assignment.
    NoValueInAssignment,
    /// A string literal didn't have closing quotes.
    UnclosedStringLiteral,
    /// The value in a key/value assignment wasn't recognised.
    UnrecognisedValue,
    /// The same key was used twice.
    ReusedKey,
}

mod crate_prelude {
    pub use super::{
        text::{Span, Text},
        value::{TomlData, Value},
        Error, ErrorKind,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> TOML<'a> {
        #[inline]
        pub fn assert_value(&self, key: &str, expected_value: Value<'_>) {
            assert_eq!(*self.get(key).unwrap(), expected_value);
        }
        #[inline]
        pub fn assert_values(&self, expected_values: Vec<(&str, Value<'_>)>) {
            for (key, expected_value) in expected_values {
                self.assert_value(key, expected_value);
            }
        }
    }

    /// Test that boml can parse booleans and bare keys.
    #[test]
    fn bools_and_bare_keys() {
        let toml_source = concat!("val1 = true\n", "val2 = false\n", "5678 = true");
        let toml = TOML::parse(toml_source).unwrap();
        toml.assert_values(vec![
            ("val1", Value::Boolean(true)),
            ("val2", Value::Boolean(false)),
            ("5678", Value::Boolean(true)),
        ]);
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
        let toml = TOML::parse(toml_source).unwrap();
        toml.assert_values(vec![
            ("val0.1.1", Value::Boolean(true)),
            ("ʎǝʞ", Value::Boolean(true)),
            ("quoted 'key'", Value::Boolean(true)),
            ("quoted \"key\" 2", Value::Boolean(true)),
        ]);
    }

    /// Test that boml works with CRLF files.
    #[test]
    fn crlf() {
        let toml_source = concat!("val1 = true\r\n", "val2 = false");
        let toml = TOML::new(toml_source).unwrap();
        assert_eq!(*toml.get("val1").unwrap(), Value::Boolean(true));
        assert_eq!(*toml.get("val2").unwrap(), Value::Boolean(false));
    }
}
