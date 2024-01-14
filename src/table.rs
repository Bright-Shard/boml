use crate::types::TomlString;

use {
    crate::{
        text::Span,
        types::{TomlValue, ValueType},
    },
    std::collections::HashMap,
};

#[derive(Debug, PartialEq)]
pub struct Table<'a> {
    pub map: HashMap<TomlString<'a>, TomlValue<'a>>,
    pub source: Span<'a>,
}
impl<'a> Table<'a> {
    /// Gets the value for a key. If you know what type the value should be,
    /// it's recommended to use a `get_<type>` method instead, as they simplify
    /// some issues (like literal vs basic strings).
    #[inline(always)]
    pub fn get(&self, key: &str) -> Option<&TomlValue<'a>> {
        self.map.get(key)
    }

    /// Gets the value for a key, if that value is a table.
    pub fn get_table(&self, key: &str) -> Result<&Self, FetchError<'_, 'a>> {
        match self.get(key) {
            None => Err(FetchError::InvalidKey),
            Some(val) => {
                if let TomlValue::Table(table) = val {
                    Ok(table)
                } else {
                    Err(FetchError::TypeMismatch(val, val.ty()))
                }
            }
        }
    }

    /// Gets the value for a key, if that value is a string. This works for both
    /// basic and literal strings.
    pub fn get_string(&self, key: &str) -> Result<&str, FetchError<'_, 'a>> {
        match self.get(key) {
            None => Err(FetchError::InvalidKey),
            Some(val) => match val {
                TomlValue::String(string) => Ok(string.as_str()),
                other_val => Err(FetchError::TypeMismatch(other_val, other_val.ty())),
            },
        }
    }

    /// Gets the value for a key, if that value is an integer.
    pub fn get_integer(&self, key: &str) -> Result<i64, FetchError<'_, 'a>> {
        match self.get(key) {
            None => Err(FetchError::InvalidKey),
            Some(val) => {
                if let TomlValue::Integer(int) = val {
                    Ok(*int)
                } else {
                    Err(FetchError::TypeMismatch(val, val.ty()))
                }
            }
        }
    }

    /// Gets the value for a key, if that value is a float.
    pub fn get_float(&self, key: &str) -> Result<f64, FetchError<'_, 'a>> {
        match self.get(key) {
            None => Err(FetchError::InvalidKey),
            Some(val) => {
                if let TomlValue::Float(float) = val {
                    Ok(*float)
                } else {
                    Err(FetchError::TypeMismatch(val, val.ty()))
                }
            }
        }
    }

    /// Gets the value for a key, if that value is a boolean.
    pub fn get_boolean(&self, key: &str) -> Result<bool, FetchError<'_, 'a>> {
        match self.get(key) {
            None => Err(FetchError::InvalidKey),
            Some(val) => {
                if let TomlValue::Boolean(bool) = val {
                    Ok(*bool)
                } else {
                    Err(FetchError::TypeMismatch(val, val.ty()))
                }
            }
        }
    }
}

/// Errors for the `get_<type>` methods in [`Table`].
#[derive(Debug, PartialEq)]
pub enum FetchError<'a, 'table> {
    /// There was no value for this key.
    InvalidKey,
    /// The value for this key had a different type.
    TypeMismatch(&'a TomlValue<'table>, ValueType),
}
