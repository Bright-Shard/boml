use crate::crate_prelude::*;

/// A value in TOML.
#[derive(Debug, PartialEq)]
pub enum TomlValue<'a> {
	/// A basic or literal string. If it's a basic string with escapes,
	/// those escapes have already been processed.
	String(TomlString<'a>),
	/// An integer.
	Integer(i64),
	/// A float.
	Float(f64),
	/// A boolean.
	Boolean(bool),
	/// Time values are currently unsupported.
	OffsetDateTime,
	/// Time values are currently unsupported.
	LocalDateTime,
	/// Time values are currently unsupported.
	LocalDate,
	/// Time values are currently unsupported.
	LocalTime,
	/// An array of TOML values. They do not have to be the same type.
	Array(Vec<Self>),
	/// A table of key/value pairs.
	Table(Table<'a>),
}
impl<'a> TomlValue<'a> {
	/// The type of this value.
	pub fn value_type(&self) -> TomlValueType {
		match *self {
			Self::String(_) => TomlValueType::String,
			Self::Integer(_) => TomlValueType::Integer,
			Self::Float(_) => TomlValueType::Float,
			Self::Boolean(_) => TomlValueType::Boolean,
			Self::OffsetDateTime => TomlValueType::OffsetDateTime,
			Self::LocalDateTime => TomlValueType::LocalDateTime,
			Self::LocalDate => TomlValueType::LocalDate,
			Self::LocalTime => TomlValueType::LocalTime,
			Self::Array(_) => TomlValueType::Array,
			Self::Table(_) => TomlValueType::Table,
		}
	}

	/// Returns the string within this value, if it's a string; otherwise, fails.
	pub fn string(&self) -> Option<&str> {
		match self {
			Self::String(string) => Some(string.as_str()),
			_ => None,
		}
	}
	/// Returns the number within this value, if it's an integer; otherwise, fails.
	pub fn integer(&self) -> Option<i64> {
		match self {
			Self::Integer(num) => Some(*num),
			_ => None,
		}
	}
	/// Returns the number within this value, if it's a float; otherwise, fails.
	pub fn float(&self) -> Option<f64> {
		match self {
			Self::Float(num) => Some(*num),
			_ => None,
		}
	}
	/// Returns the boolean within this value, if it's a boolean; otherwise, fails.
	pub fn boolean(&self) -> Option<bool> {
		match self {
			Self::Boolean(bool_) => Some(*bool_),
			_ => None,
		}
	}
	/// Returns the array within this value, if it's an array; otherwise, fails.
	pub fn array(&self) -> Option<&Vec<Self>> {
		match self {
			Self::Array(array) => Some(array),
			_ => None,
		}
	}
	/// Returns the table within this value, if it's a table; otherwise, fails.
	pub fn table(&self) -> Option<&Table<'a>> {
		match self {
			Self::Table(table) => Some(table),
			_ => None,
		}
	}
}

/// The basic value types in TOML.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TomlValueType {
	String,
	Integer,
	Float,
	Boolean,
	OffsetDateTime,
	LocalDateTime,
	LocalDate,
	LocalTime,
	Array,
	Table,
}

/// A key in a key/value pair or table name.
pub struct Key<'a> {
	/// The name of this key.
	pub text: TomlString<'a>,
	/// This is only present in dotted keys. It stores the next "child" key
	/// that comes after the dot.
	pub child: Option<Box<Key<'a>>>,
}
