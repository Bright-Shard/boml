use std::collections::HashMap;

use crate::{
	table::{TomlGetError, TomlTable},
	types::{TomlValue, TomlValueType}, Toml,
};

/// Error type returned by `FromToml::from_toml`.
#[derive(Debug)]
pub enum FromTomlError<'a> {
	/// There was no value to convert.
	Missing,
	/// The key was invalid
	InvalidKey(&'a str),
	/// The value had a different type than expected.
	TypeMismatch(&'a TomlValue<'a>, TomlValueType),
}

impl<'a> FromTomlError<'a> {
	/// Converts to `InvalidKey` if the error variant is `Missing`.
	pub fn add_key_context(self, key: &'a str) -> Self {
		match self {
			FromTomlError::Missing => FromTomlError::InvalidKey(key),
			other => other,
		}
	}
}

impl<'a> From<TomlGetError<'a>> for FromTomlError<'a> {
	fn from(e: TomlGetError<'a>) -> Self {
		match e {
			TomlGetError::InvalidKey => FromTomlError::Missing,
			TomlGetError::TypeMismatch(v, ty) => FromTomlError::TypeMismatch(v, ty),
		}
	}
}

/// A trait for types that can be constructed from a TOML value. Used by the derive macro.
///
/// This trait is implemented for all types that implement `TryFrom<&'a TomlValue<'a>, Error = ()>`.
pub trait FromTomlValue<'a>: Sized {
	/// Constructs a datatype from a `TomlValue`.
	fn from_toml_value(value: Option<&'a TomlValue<'a>>) -> Result<Self, FromTomlError<'a>>;
}
/// A trait for types that can be constructed from a TOML value. Used by the derive macro.
pub trait FromToml<'a>: Sized {
	/// Constructs a datatype from a TomlTable.
	fn from_toml_table(table: &'a TomlTable<'a>) -> Result<Self, FromTomlError<'a>>;

	/// Constructs a datatype from a parsed Toml file.
	fn from_toml(toml: &'a Toml<'a>) -> Result<Self, FromTomlError<'a>> {
		Self::from_toml_table(&toml.table)
	}
}

impl<'a, T> FromTomlValue<'a> for T
where
	T: TryFrom<&'a TomlValue<'a>, Error = ()>,
{
	fn from_toml_value(value: Option<&'a TomlValue<'a>>) -> Result<Self, FromTomlError<'a>> {
		match value {
			Some(v) => T::try_from(v).map_err(|_| FromTomlError::TypeMismatch(v, v.ty())),
			None => Err(FromTomlError::Missing),
		}
	}
}

impl<'a, T> FromTomlValue<'a> for Vec<T>
where
	T: FromTomlValue<'a>,
{
	fn from_toml_value(value: Option<&'a TomlValue<'a>>) -> Result<Self, FromTomlError<'a>> {
		match value {
			Some(TomlValue::Array(arr, _)) => arr.iter().map(|v| T::from_toml_value(Some(v))).collect(),
			Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Array)),
			None => Err(FromTomlError::Missing),
		}
	}
}

impl<'a, T> FromTomlValue<'a> for Option<T>
where
	T: FromTomlValue<'a>,
{
	fn from_toml_value(value: Option<&'a TomlValue<'a>>) -> Result<Self, FromTomlError<'a>> {
		match value {
			Some(v) => Ok(Some(T::from_toml_value(Some(v))?)),
			None => Ok(None),
		}
	}
}
impl<'a, T> FromTomlValue<'a> for HashMap<&'a str, T>
where
	T: FromTomlValue<'a>,
{
	fn from_toml_value(value: Option<&'a TomlValue<'a>>) -> Result<Self, FromTomlError<'a>> {
		match value {
			Some(TomlValue::Table(table)) => table
				.map
				.iter()
				.map(|(k, v)| Ok((k.as_str(), T::from_toml_value(Some(v))?)))
				.collect(),
			Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
			None => Err(FromTomlError::Missing),
		}
	}
}

/// Inverse trait of `FromToml`. Used to convert a TOML value into a type.
pub trait TomlTryInto<'a, T>: Sized {
	/// Converts the TOML value into `T``.
	fn toml_try_into(self) -> Result<T, FromTomlError<'a>>;
}

impl<'a, T> TomlTryInto<'a, T> for Option<&'a TomlValue<'a>>
where
	T: FromTomlValue<'a>,
{
	fn toml_try_into(self) -> Result<T, FromTomlError<'a>> {
		T::from_toml_value(self)
	}
}
