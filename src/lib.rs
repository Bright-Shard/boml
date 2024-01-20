pub mod parser;
pub mod table;
pub mod text;
pub mod types;

use {crate_prelude::*, std::ops::Deref};

/// BOML's TOML parser. Create a new one with [`new()`] or [`parse()`], then use
/// it just like a [`Table`].
///
/// [`new()`]: Toml::new()
/// [`parse()`]: Toml::parse()
#[derive(Debug)]
pub struct Toml<'a> {
	table: Table<'a>,
}
impl<'a> Toml<'a> {
	/// A wrapper around [`Toml::parse()`].
	#[inline(always)]
	pub fn new(text: &'a str) -> Result<Self, Error> {
		Self::parse(text)
	}

	/// Attempts to parse the provided string as TOML.
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

						current_table = Some((table_name, Table::default(), false));
					}
				}
				// Key definition
				_ => {
					let (key, value) = parser::parse_assignment(&mut text)?;

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

		Ok(Self { table: root_table })
	}

	/// Consumes the [`Toml<'_>`], producing a [`Table<'_>`].
	pub fn into_table(self) -> Table<'a> {
		self.table
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
		let Some(TomlValue::Table(to_insert)) =
			root_table.get_or_insert_mut(key, TomlValue::Table(Table::default()))
		else {
			return Err(Error {
				start,
				end,
				kind: ErrorKind::ReusedKey,
			});
		};

		for (key, value) in table.map {
			let (start, end) = (key.span().start, key.span().end);
			let old = to_insert.map.insert(key, value);

			if old.is_some() {
				return Err(Error {
					start,
					end,
					kind: ErrorKind::ReusedKey,
				});
			}
		}
	}

	Ok(())
}

/// An error while parsing TOML, and the range of text that caused
/// that error.
#[derive(Debug)]
pub struct Error {
	/// The first byte (inclusive) of the text that caused a parsing
	/// error.
	pub start: usize,
	/// The last byte (inclusive) of the text that caused a parsing
	/// error.
	pub end: usize,
	/// The type of parsing error; see the [`ErrorKind`] docs.
	pub kind: ErrorKind,
}

/// A type of error while parsing TOML.
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
	/// A number has an invalid base or a leading zero. This error will be thrown
	/// for floats or times with bases, since they cannot have bases.
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
		text::{CowSpan, Span, Text},
		types::{Key, TomlValue, TomlValueType},
		Error, ErrorKind,
	};
}

pub mod prelude {
	pub use crate::{
		table::{Table as TomlTable, TomlGetError},
		types::{TomlValue, TomlValueType},
		Error as TomlError, ErrorKind as TomlErrorKind, Toml,
	};
}
