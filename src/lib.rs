use types::Key;

pub mod parser;
pub mod table;
pub mod text;
pub mod types;

use {crate_prelude::*, std::ops::Deref};

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
        let mut root_table = Table::default();
        // (table name, table, if it's a member of an array of tables)
        let mut current_table: Option<(Key<'_>, Table<'_>, bool)> = None;

        while text.idx < text.end() {
            match text.current_byte().unwrap() {
                // Comment
                b'#' => {
                    if let Some(newline_idx) = text.excerpt(text.idx..).find(b'\n') {
                        text.idx = newline_idx;
                    } else {
                        // Comment is at end of file
                        break;
                    }
                }
                // Table definition
                b'[' => {
                    if let Some((key, table, array)) = current_table.take() {
                        insert_subtable(&mut root_table, key, table, array)?;
                    }

                    if text.byte(text.idx + 1) == Some(b'[') {
                        text.idx += 2;
                        text.skip_whitespace();
                        let table_name = parser::parse_key(&mut text)?;
                        text.idx += 1;
                        text.skip_whitespace();

                        if text.current_byte() != Some(b']')
                            || text.byte(text.idx + 1) != Some(b']')
                        {
                            return Err(Error {
                                start: table_name.text.span().start - 1,
                                end: table_name.text.span().end,
                                kind: ErrorKind::UnclosedBracket,
                            });
                        }
                        text.idx += 2;

                        current_table = Some((table_name, Table::default(), true));
                    } else {
                        text.idx += 1;
                        text.skip_whitespace();
                        let table_name = parser::parse_key(&mut text)?;
                        text.idx += 1;
                        text.skip_whitespace();

                        if text.current_byte() != Some(b']') {
                            return Err(Error {
                                start: table_name.text.span().start - 1,
                                end: table_name.text.span().end,
                                kind: ErrorKind::UnclosedBracket,
                            });
                        }
                        text.idx += 1;

                        println!("In table: `{}`", table_name.text);

                        current_table = Some((table_name, Table::default(), false));
                    }
                }
                // Key definition
                _ => {
                    let (key, value) = parser::parse_assignment(&mut text)?;
                    println!("Key: {}\nValue:{value:?}", key.text);

                    let table = if let Some((_, ref mut table, _)) = current_table {
                        table
                    } else {
                        &mut root_table
                    };

                    table.insert(key, value);

                    text.idx += 1;
                }
            }

            text.skip_whitespace_and_newlines();
        }

        if let Some((key, table, array)) = current_table.take() {
            insert_subtable(&mut root_table, key, table, array)?;
        }

        Ok(Self {
            text: text.text,
            table: root_table,
        })
    }
}
impl<'a> Deref for Toml<'a> {
    type Target = Table<'a>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

fn insert_subtable<'a>(
    root_table: &mut Table<'a>,
    key: Key<'a>,
    table: Table<'a>,
    array: bool,
) -> Result<(), Error> {
    let (start, end) = (key.text.span().start, key.text.span().end);
    if array {
        let Some(TomlValue::Array(array)) =
            root_table.get_or_insert_mut(key, TomlValue::Array(Vec::new()))
        else {
            return Err(Error {
                start,
                end,
                kind: ErrorKind::ReusedKey,
            });
        };
        array.push(TomlValue::Table(table));
    } else {
        let old = root_table.insert(key, TomlValue::Table(table));
        if old {
            return Err(Error {
                start,
                end,
                kind: ErrorKind::ReusedKey,
            });
        }
    }

    Ok(())
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
    /// A table, inline table, or array didn't have a closing bracket.
    UnclosedBracket,
    /// There was no `,` in between values in an inline table or array.
    NoCommaDelimeter,
}

mod crate_prelude {
    pub use super::{
        table::Table,
        text::{Span, Text},
        types::{Key, TomlString, TomlValue, ValueType},
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
            "inline = { name = 'inline', num = inf }\n",
            "\n",
            "[table1]\n",
            "name = 'table1'\n",
            "\n",
            "[table2]\n",
            "name = 'table2'\n",
            "num = 420\n"
        );
        let toml = Toml::parse(toml_source).unwrap();

        let inline = toml.get_table("inline").unwrap();
        assert_eq!(inline.get_string("name"), Ok("inline"));
        assert_eq!(inline.get_float("num"), Ok(f64::INFINITY));

        let table1 = toml.get_table("table1").unwrap();
        assert_eq!(table1.get_string("name"), Ok("table1"));

        let table2 = toml.get_table("table2").unwrap();
        assert_eq!(table2.get_string("name"), Ok("table2"));
        assert_eq!(table2.get_integer("num"), Ok(420));
    }

    /// Test that boml can parse arrays.
    #[test]
    fn arrays() {
        let toml_source = concat!(
            "strings = ['hi', 'hello', 'how are you']\n",
            "nested = ['me', ['when i', 'nest'], 'arrays']\n",
            "tables = [{name = 'bruh'}, {name = 'bruh 2 electric boogaloo'}]\n"
        );
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
