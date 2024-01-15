use {crate::crate_prelude::*, std::collections::HashMap};

/// A set of key/value pairs in TOML.
#[derive(Debug, PartialEq, Default)]
pub struct Table<'a> {
	map: HashMap<TomlString<'a>, TomlValue<'a>>,
}
impl<'a> Table<'a> {
	/// Gets the value for a key. If you know what type the value should be,
	/// it's recommended to use a `get_<type>` method instead, as they simplify
	/// some issues (like literal vs basic strings).
	#[inline(always)]
	pub fn get(&self, key: &str) -> Option<&TomlValue<'a>> {
		self.map.get(key)
	}

	/// The number of entries in this table.
	#[inline(always)]
	pub fn len(&self) -> usize {
		self.map.len()
	}
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	/// Gets the value for a key, if that value is a table.
	pub fn get_table(&self, key: &str) -> Result<&Self, TomlGetError<'_, 'a>> {
		match self.get(key) {
			None => Err(TomlGetError::InvalidKey),
			Some(val) => {
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
			Some(val) => match val {
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
			Some(val) => {
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
			Some(val) => {
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
			Some(val) => {
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
			Some(val) => {
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
			let TomlValue::Table(ref mut table) = self
				.map
				.entry(key.text)
				.or_insert(TomlValue::Table(Table::default()))
			else {
				return true;
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
			let TomlValue::Table(ref mut table) = self
				.map
				.entry(key.text)
				.or_insert(TomlValue::Table(Table::default()))
			else {
				return None;
			};

			table.get_or_insert_mut(*child, value)
		} else {
			Some(self.map.entry(key.text).or_insert(value))
		}
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
