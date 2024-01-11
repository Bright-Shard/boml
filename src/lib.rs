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
            // println!(
            //     "Main loop is at: |{}|",
            //     text.byte(text.idx).unwrap().to_owned() as char
            // );
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
    /// A number was too big to fit in an i64. This will also be thrown
    /// for numbers that are "too little", ie, are too negative to fit.
    NumberTooLarge,
    /// A number has an invalid base or a leading zero.
    NumberHasInvalidBaseOrLeadingZero,
    /// A number is malformed/not parseable.
    InvalidNumber,
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
        let toml = TOML::parse(toml_source).unwrap();
        toml.assert_values(vec![
            ("val1", Value::Boolean(true)),
            ("val2", Value::Boolean(false)),
            ("5678", Value::Boolean(true)),
            ("dash-ed", Value::Boolean(true)),
            ("under_score", Value::Boolean(true)),
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

    /// Test that boml can parse literal strings and multiline literal strings.
    #[test]
    fn literal_strings() {
        let single = "Me when I have to write a demo sentence to test my incredible TOML parser but dunno what to say";
        let multi = "Bruhhhh I gotta write\n*another*\ndemo sentence???\n:(";
        let toml_source = format!("single = '{single}'\n") + &format!("multi = '''{multi}'''");
        let toml = TOML::parse(&toml_source).unwrap();
        toml.assert_values(vec![
            ("single", Value::String(single)),
            ("multi", Value::String(multi)),
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
            "lmao = -0\n"
        );
        let toml = TOML::parse(toml_source).unwrap();
        toml.assert_values(vec![
            ("hex", Value::Integer(16)),
            ("decimal", Value::Integer(10)),
            ("octal", Value::Integer(8)),
            ("binary", Value::Integer(2)),
            ("neghex", Value::Integer(-16)),
            ("posoctal", Value::Integer(8)),
            ("lmao", Value::Integer(0)),
        ]);
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
            "neg_fractional = -.123\n",
            "capital_exponential = 2E2\n",
            "combined = 7.27e2\n"
        );
        let toml = TOML::parse(toml_source).unwrap();
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
            ]
            .into_iter()
            .map(|(key, val)| (key, Value::Float(val)))
            .collect(),
        );
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
        let toml = TOML::new(toml_source).unwrap();
        toml.assert_values(vec![
            ("val1", Value::Boolean(true)),
            ("val2", Value::Boolean(false)),
            ("val3", Value::Boolean(true)),
            ("val4", Value::Boolean(false)),
            ("val5", Value::Boolean(true)),
        ]);
    }

    impl<'a> TOML<'a> {
        #[inline]
        pub fn assert_value(&self, key: &str, expected_value: Value<'_>) {
            assert_eq!(*self.get(key).unwrap(), expected_value);
        }
        #[inline]
        pub fn assert_values(&self, expected_values: Vec<(&str, Value<'_>)>) {
            for (key, expected_value) in expected_values {
                self.assert_value(key, expected_value);
                // println!("Asserted value");
            }
        }
    }
}
