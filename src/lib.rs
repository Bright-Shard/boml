#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod parser;
pub mod table;
mod text;
pub mod types;

use {
	crate::table::TomlTable, std::{
		fmt::{Debug, Display},
		ops::Deref,
	}, table::TomlGetError, text::Span, types::{TomlValue, TomlValueType}
};

/// Attempts to parse the given TOML.
pub fn parse(str: &str) -> Result<Toml<'_>, TomlError> {
	parser::parse_str(str)
}


/// A parsed TOML file.
#[derive(Debug)]
pub struct Toml<'a> {
	source: &'a str,
	table: TomlTable<'a>,
}
impl<'a> Toml<'a> {
	/// Attempts to parse the given TOML.
	pub fn new(str: &'a str) -> Result<Self, TomlError<'a>> {
		parser::parse_str(str)
	}
	/// Attempts to parse the given TOML.
	pub fn parse(str: &'a str) -> Result<Self, TomlError<'a>> {
		parser::parse_str(str)
	}

	/// The source code of this TOML.
	pub fn source(&self) -> &str {
		self.source
	}
}
impl<'a> From<Toml<'a>> for TomlTable<'a> {
	fn from(value: Toml<'a>) -> TomlTable<'a> {
		value.table
	}
}
impl<'a> Deref for Toml<'a> {
	type Target = TomlTable<'a>;

	fn deref(&self) -> &Self::Target {
		&self.table
	}
}

/// An error while parsing TOML.
pub struct TomlError<'a> {
	/// An excerpt of the region of text that caused the error.
	pub src: Span<'a>,
	/// The type of parsing error; see [`TomlErrorKind`].
	pub kind: TomlErrorKind,
}
impl Debug for TomlError<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut start = self.src.source[..self.src.start].bytes().enumerate().rev();
		let mut newlines = 3u8;
		while newlines != 0 {
			match start.next() {
				None => break,
				Some((_, b'\n')) => newlines -= 1,
				_ => {}
			}
		}
		let start = start.next().map(|(idx, _)| idx + 2).unwrap_or(0);

		let mut end = self.src.source[self.src.end..].bytes().enumerate();
		let mut newlines = 3u8;
		while newlines != 0 {
			match end.next() {
				None => break,
				Some((_, b'\n')) => newlines -= 1,
				_ => {}
			}
		}
		let end = end
			.next()
			.map(|(idx, _)| self.src.end + idx - 1)
			.unwrap_or(self.src.source.len());

		write!(
			f,
			"Error: {:?} at `{}`\nIn:\n{}",
			self.kind,
			self.src.as_str(),
			&self.src.source[start..end]
		)
	}
}
impl Display for TomlError<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}
/// A type of error while parsing TOML.
#[derive(Debug, PartialEq, Eq)]
pub enum TomlErrorKind {
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
	/// A basic string (`"hello"`) didn'a have a closing quote.
	UnclosedBasicString,
	/// A literal string (`'hello'`) didn'a have a closing quote.
	UnclosedLiteralString,
	/// A quoted key didn'a have a closing quote.
	UnclosedQuotedKey,
	/// The value in a key/value assignment wasn'a recognised.
	UnrecognisedValue,
	/// The same key was used twice.
	ReusedKey,
	/// A number was too big to fit in an i64. This will be thrown for both
	/// positive and negative numbers.
	NumberTooLarge,
	/// An integer has an invalid base. Valid bases are hex (0x), octal (0o),
	/// and binary (0b).
	NumberHasInvalidBase,
	/// A literal number starts with a 0.
	NumberHasLeadingZero,
	/// A number is malformed/not parseable.
	InvalidNumber,
	/// A basic string has an unknown escape sequence.
	UnknownEscapeSequence,
	/// A unicode escape in a basic string has an unknown unicode scalar value.
	UnknownUnicodeScalar,
	/// A table (`[table]`) had an unclosed bracket.
	UnclosedTableBracket,
	/// An inline table (`{key = "val", one = 2}`) had an unclosed bracket.
	UnclosedInlineTableBracket,
	/// An array of tables (`[[array_table]]`) was missing closing brackets.
	UnclosedArrayOfTablesBracket,
	/// An array literal (`[true, "hi", 123]`) was missing a closing bracket.
	UnclosedArrayBracket,
	/// There was no `,` in between values in an inline table or array.
	NoCommaDelimeter,
	/// One section (year, month, day, hour, etc) of a date/time value had too
	/// many digits.
	DateTimeTooManyDigits,
	/// A date value was missing its month.
	DateMissingMonth,
	/// A date value was missing its day.
	DateMissingDay,
	/// A date value was missing the `-` between a year/month/day.
	DateMissingDash,
	/// A time value was missing its minute.
	TimeMissingMinute,
	/// A time value was missing its second.
	TimeMissingSecond,
	/// A time value was missing the `:` between its hour/minute/second.
	TimeMissingColon,
	/// The offset portion of an offset datetime was missing its hour.
	OffsetMissingHour,
	/// The offset portion of an offset datetime was missing its minute.
	OffsetMissingMinute,
}

/// Types that may be useful to have imported while using BOML.
pub mod prelude {
	pub use crate::{
		table::{TomlGetError, TomlTable},
		types::{TomlValue, TomlValueType},
		Toml, TomlError, TomlErrorKind,
	};
}
///
pub trait FromToml<'a>: Sized {
	///
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>>;
}

impl<'a> FromToml<'a> for bool {
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::Boolean(b)) => Ok(*b),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::Boolean)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a> FromToml<'a> for i64 {
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::Integer(i)) => Ok(*i),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::Integer)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a> FromToml<'a> for f64 {
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::Float(f)) => Ok(*f),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::Float)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a> FromToml<'a> for String {
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::String(s)) => Ok(s.to_string()),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::String)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a> FromToml<'a> for &'a str {
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::String(s)) => Ok(s.as_str()),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::String)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a, T> FromToml<'a> for Vec<T>
where
	T: FromToml<'a>,
{
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {
			Some(TomlValue::Array(arr, _)) => arr.iter().map(|v| T::from_toml(Some(v))).collect(),
			Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::Array)),
			None => Err(TomlGetError::InvalidKey),
		}
	}
}

impl<'a, T> FromToml<'a> for Option<T>
where
	T: FromToml<'a>,
{
	fn from_toml(value: Option<&'a TomlValue<'a>>) -> Result<Self, TomlGetError<'a, 'a>> {
		match value {			
			Some(v) => Ok(Some(T::from_toml(Some(v))?)),
			None => Ok(None),
		}
	}
}

///
pub trait TomlTryInto<'a, T>: Sized {
	///
	fn toml_try_into(self) -> Result<T, TomlGetError<'a, 'a>>;
}
impl <'a, T> TomlTryInto<'a, T> for Option<&'a TomlValue<'a>>
where T: FromToml<'a> {
	fn toml_try_into(self) -> Result<T, TomlGetError<'a, 'a>> {
		T::from_toml(self)
	}
}
