//! See [`TomlTable`].

use {
	crate::{
		text::{CowSpan, Text},
		types::{TomlValue, TomlValueType},
		TomlError, TomlErrorKind,
	},
	std::{
		collections::{
			hash_map::{Entry, VacantEntry},
			HashMap,
		},
		ops::Deref,
	},
};

/// A set of key/value pairs in TOML.
#[derive(Debug, PartialEq, Default)]
pub struct TomlTable<'a> {	
	pub(crate) map: HashMap<CowSpan<'a>, TomlValue<'a>>,
}
impl<'a> TomlTable<'a> {
	/// Gets the value for a key, if that value is a table.
	pub fn get_table(&'a self, key: &str) -> Result<&'a Self, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Table(table) = val {
					Ok(table)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.ty()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a string.
	pub fn get_string(&'a self, key: &str) -> Result<&'a str, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => match val {
				TomlValue::String(string) => Ok(string.as_str()),
				other_val => Err(TomlGetError::TypeMismatch(other_val, other_val.ty())),
			},
		}
	}
	/// Gets the value for a key, if that value is an integer.
	pub fn get_integer(&'a self, key: &str) -> Result<i64, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Integer(int) = val {
					Ok(*int)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.ty()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a float.
	pub fn get_float(&'a self, key: &str) -> Result<f64, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Float(float) = val {
					Ok(*float)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.ty()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is a boolean.
	pub fn get_boolean(&'a self, key: &str) -> Result<bool, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Boolean(bool) = val {
					Ok(*bool)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.ty()))
				}
			}
		}
	}
	/// Gets the value for a key, if that value is an array.
	pub fn get_array(&'a self, key: &str) -> Result<&'a Vec<TomlValue<'a>>, TomlGetError<'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(ref val) => {
				if let TomlValue::Array(array, _) = val {
					Ok(array)
				} else {
					Err(TomlGetError::TypeMismatch(val, val.ty()))
				}
			}
		}
	}

	pub(crate) fn value_entry<'b>(
		&'b mut self,
		text: &mut Text<'a>,
	) -> Result<VacantEntry<'b, CowSpan<'a>, TomlValue<'a>>, TomlError<'a>> {
		let start = text.idx();
		let (table, key) = crate::parser::key::parse_nested(text, self)?;

		match table.map.entry(key) {
			Entry::Occupied(_) => Err(TomlError {
				src: text.excerpt_to_idx(start..),
				kind: TomlErrorKind::ReusedKey,
			}),
			Entry::Vacant(vacant) => Ok(vacant),
		}
	}
}
impl<'a> Deref for TomlTable<'a> {
	type Target = HashMap<CowSpan<'a>, TomlValue<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

/// Errors for the `get_<type>` methods in [`TomlTable`].
#[derive(Debug, PartialEq)]
pub enum TomlGetError<'a> {
	/// There was no value for the provided key.
	InvalidKey,
	/// The value for the provided key had a different type. Stores the
	/// value for that key and its type.
	TypeMismatch(&'a TomlValue<'a>, TomlValueType),
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Tester {
		key: &'static str,
		value: TomlValue<'static>,
	}
	impl Tester {
		fn build(self) -> TomlTable<'static> {
			println!("Running test for key `{}`", self.key);

			let mut table = TomlTable::default();
			table
				.value_entry(&mut Text::new(self.key))
				.unwrap()
				.insert(self.value);
			table
		}
	}

	#[test]
	fn test_table_keys() {
		let basic = Tester {
			key: "bool",
			value: TomlValue::Boolean(true),
		}
		.build();
		assert_eq!(basic.get("bool"), Some(&TomlValue::Boolean(true)));

		let dotted = Tester {
			key: "dot.bool",
			value: TomlValue::Boolean(true),
		}
		.build();
		let Some(TomlValue::Table(subtable)) = dotted.get("dot") else {
			panic!()
		};
		assert_eq!(subtable.get("bool"), Some(&TomlValue::Boolean(true)));

		let quoted = Tester {
			key: "'wowza.hi'",
			value: TomlValue::Boolean(true),
		}
		.build();
		assert_eq!(quoted.get("wowza.hi"), Some(&TomlValue::Boolean(true)));

		let quoted_alt = Tester {
			key: r#""wowza.hi""#,
			value: TomlValue::Boolean(true),
		}
		.build();
		assert_eq!(quoted_alt.get("wowza.hi"), Some(&TomlValue::Boolean(true)));
	}
}
