//! Defines the [`Table`] type.

use {
	crate::crate_prelude::*,
	std::{collections::HashMap, ops::Deref},
};

/// A set of key/value pairs in TOML.
#[derive(Debug, PartialEq, Default)]
pub struct Table<'a> {
	pub(crate) map: HashMap<CowSpan<'a>, TomlValue<'a>>,
}
impl<'a> Table<'a> {
	/// Gets the value for a key, if that value is a table.
	pub fn get_table(&self, key: &str) -> Result<&Self, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Table(table) = val {
					Ok(table)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.value_type()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a string.
	pub fn get_string(&self, key: &str) -> Result<&str, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => match val {
				TomlValue::String(string) => Ok(string.as_str()),
				other_val => Err(TomlGetError::TypeMismatch(
					other_val,
					other_val.value_type(),
				)),
			},
		}
	}
	/// Gets the value for a key, if that value is an integer.
	pub fn get_integer(&self, key: &str) -> Result<i64, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Integer(int) = val {
					Ok(*int)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.value_type()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a float.
	pub fn get_float(&self, key: &str) -> Result<f64, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Float(float) = val {
					Ok(*float)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.value_type()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a boolean.
	pub fn get_boolean(&self, key: &str) -> Result<bool, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Boolean(bool) = val {
					Ok(*bool)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.value_type()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is an array.
	pub fn get_array(&self, key: &str) -> Result<&Vec<TomlValue<'a>>, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Array(array) = val {
					Ok(array)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.value_type()))
				}
			}
		}
	}

	/// Inserts a value into the table, handling dotted keys automatically. Returns true if
	/// inserting the value overwrote another value.
	pub(crate) fn insert(&mut self, key: Key<'a>, value: TomlValue<'a>) -> bool {
		if let Some(child) = key.child {
			let possible_table = self
				.map
				.entry(key.text)
				.or_insert(TomlValue::Table(Table::default()));

			let table = match possible_table {
				TomlValue::Array(array) => {
					let Some(TomlValue::Table(table)) = array.last_mut() else {
						return true;
					};
					table
				}
				TomlValue::Table(table) => table,
				_ => return true,
			};

			table.insert(*child, value)
		} else {
			self.map.insert(key.text, value).is_some()
		}
	}
	/// Gets a value from the table, or inserts one if it doesn't exist. This handles dotted keys automatically,
	/// but will return `None` if the key is invalid (ie indexes into something that isn't a table).
	pub(crate) fn get_or_insert_mut(
		&mut self,
		key: Key<'a>,
		value: TomlValue<'a>,
	) -> Option<&mut TomlValue<'a>> {
		if let Some(child) = key.child {
			let possible_table = self
				.map
				.entry(key.text)
				.or_insert(TomlValue::Table(Table::default()));

			let table = match possible_table {
				TomlValue::Array(array) => {
					let Some(TomlValue::Table(table)) = array.last_mut() else {
						return None;
					};
					table
				}
				TomlValue::Table(table) => table,
				_ => return None,
			};

			table.get_or_insert_mut(*child, value)
		} else {
			Some(self.map.entry(key.text).or_insert(value))
		}
	}

	/// Iterates over the (key, value) pairs in this table. This replaces the [`HashMap`]'s normal iter method,
	/// so that the keys are normal `&str`s instead of boml's internal [`CowSpan`] string type.
	pub fn iter(&self) -> impl Iterator<Item = (&str, &TomlValue<'_>)> {
		self.map.iter().map(|(k, v)| (k.as_str(), v))
	}
}
impl<'a> Deref for Table<'a> {
	type Target = HashMap<CowSpan<'a>, TomlValue<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

/// Errors for the `get_<type>` methods in [`Table`].
#[derive(Debug, PartialEq)]
pub enum TomlGetError<'a, 'table> {
	/// There was no value for the provided key.
	InvalidKey,
	/// The value for the provided key had a different type. Stores the
	/// value for that key and its type.
	TypeMismatch(&'a TomlValue<'table>, TomlValueType),
}
