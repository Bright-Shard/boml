pub mod parser;
pub mod table;
pub mod text;
pub mod types;

use {
    crate_prelude::*,
    std::{collections::HashMap, ops::Deref},
};

pub struct Toml<'a> {
    pub text: &'a str,
    table: Table<'a>,
}
impl<'a> Toml<'a> {
    #[inline(always)]
    pub fn new(text: &'a str) -> Result<Self, Error> {
        Self::parse(text)
    }

    pub fn parse(text: &'a str) -> Result<Self, Error> {
        let mut text = Text { text, idx: 0 };
        text.skip_whitespace_and_newlines();
        let mut values = HashMap::new();

        while text.idx < text.len() - 1 {
            match text.current_byte().unwrap() {
                // Comment
                b'#' => {
                    todo!()
                }
                // Table definition
                b'[' => {
                    todo!()
                }
                // Key definition
                _ => {
                    let key = parser::parse_key(&mut text)?;

                    text.idx += 1;
                    text.skip_whitespace();
                    if text.current_byte() != Some(b'=') {
                        return Err(Error {
                            start: key.span().start,
                            end: text.idx,
                            kind: ErrorKind::NoEqualsInAssignment,
                        });
                    }
                    text.idx += 1;
                    text.skip_whitespace();
                    if text.is_empty() {
                        return Err(Error {
                            start: key.span().start,
                            end: text.idx,
                            kind: ErrorKind::NoValueInAssignment,
                        });
                    }

                    let value = parser::parse_value(&mut text)?;
                    text.idx += 1;
                    values.insert(key, value);
                }
            }

            text.skip_whitespace_and_newlines();
        }

        let table = Table {
            map: values,
            source: Span {
                start: 0,
                end: text.len(),
                source: text.text,
            },
        };

        Ok(Self {
            text: text.text,
            table,
        })
    }
}
impl<'a> Deref for Toml<'a> {
    type Target = Table<'a>;

    fn deref(&self) -> &Self::Target {
        &self.table
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
    /// There was no `=` sign in a key/value assignment.
    NoEqualsInAssignment,
    /// There was no key in a key/value assignment.
    NoKeyInAssignment,
    /// There was no value in a key/value assignment.
    NoValueInAssignment,
    /// A string literal or quoted key didn't have a closing quote.
    UnclosedString,
    /// The value in a key/value assignment wasn't recognised.
    UnrecognisedValue,
    /// The same key was used twice.
    ReusedKey,
    /// A number was too big to fit in an i64. This will also be thrown
    /// for numbers that are "too little", ie, are too negative to fit.
    NumberTooLarge,
    /// A number has an invalid base or a leading zero.
    NumberHasInvalidBaseOrLeadingZero,
    /// A number is malformed/not parseable.
    InvalidNumber,
    /// A basic string has an unknown escape sequence.
    UnknownEscapeSequence,
    /// A unicode escape in a basic string has an unknown unicode scalar value.
    UnknownUnicodeScalar,
}

mod crate_prelude {
    pub use super::{
        table::Table,
        text::{Span, Text},
        types::TomlValue,
        Error, ErrorKind,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "underscore = 10_00\n"
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
            "infinity = -inf\n"
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

    /// Test that boml works with weird formats - CRLF, weird spacings, etc.
    #[test]
    fn weird_formats() {
        let toml_source = concat!(
            "val1 = true\r\n",
            "val2=      false",
            "\n\r\n\r\n\n",
            "val3  =true\n",
            "val4=false\n",
            "val5 = true      "
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
    }

    impl<'a> Toml<'a> {
        #[inline]
        pub fn assert_value(&self, key: &str, expected_value: TomlValue<'_>) {
            assert_eq!(*self.get(key).unwrap(), expected_value);
        }
        #[inline]
        pub fn assert_values(&self, expected_values: Vec<(&str, TomlValue<'_>)>) {
            for (key, expected_value) in expected_values {
                self.assert_value(key, expected_value);
            }
        }
        pub fn assert_strings(&self, strings: Vec<(&str, &str)>) {
            for (key, expected_string) in strings {
                let value = self.get_string(key);
                assert!(value.is_ok());
                assert_eq!(value.unwrap(), expected_string);
            }
        }
    }
}
