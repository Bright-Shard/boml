//! TOML data types.

use crate::{table::TomlTable, text::CowSpan};

/// A value in TOML.
#[derive(Debug, PartialEq)]
pub enum TomlValue<'a> {
	/// A string value.
	///
	/// This type is used for both TOML's basic string and literal string types.
	/// If the string was a basic string with escape sequences, those escapes
	/// have already been handled.
	String(CowSpan<'a>),
	/// A 64-bit signed integer.
	Integer(i64),
	/// A 64-bit float.
	Float(f64),
	/// A boolean.
	Boolean(bool),
	/// A time value.
	///
	/// BOML performs no checks on date/time types and *only* guarantees that
	/// this time value is formatted according to
	/// [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). Therefore
	/// this time value may or may not actually be valid. See the crate-level
	/// docs for more info.
	Time(TomlTime),
	/// A date value.
	///
	/// BOML performs no checks on date/time types and *only* guarantees that
	/// this time value is formatted according to
	/// [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). Therefore
	/// this time value may or may not actually be valid. See the crate-level
	/// docs for more info.
	Date(TomlDate),
	/// A date and time value.
	///
	/// BOML performs no checks on date/time types and *only* guarantees that
	/// this time value is formatted according to
	/// [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). Therefore
	/// this time value may or may not actually be valid. See the crate-level
	/// docs for more info.
	DateTime(TomlDateTime),
	/// A date and time value, offset to a specific timezone.
	///
	/// BOML performs no checks on date/time types and *only* guarantees that
	/// this time value is formatted according to
	/// [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). Therefore
	/// this time value may or may not actually be valid. See the crate-level
	/// docs for more info.
	OffsetDateTime(OffsetTomlDateTime),
	/// An array of TOML values. Note that, unlike Rust arrays, TOML arrays can
	/// store multiple types (i.e. `["string", 1234, []]` is valid).
	Array(Vec<Self>, bool),
	/// A table of key/value pairs.
	Table(TomlTable<'a>),
}
impl<'a> TomlValue<'a> {
	/// The type of this value.
	pub fn ty(&self) -> TomlValueType {
		match *self {
			Self::String(_) => TomlValueType::String,
			Self::Integer(_) => TomlValueType::Integer,
			Self::Float(_) => TomlValueType::Float,
			Self::Boolean(_) => TomlValueType::Boolean,
			Self::Time(_) => TomlValueType::Time,
			Self::Date(_) => TomlValueType::Date,
			Self::DateTime(_) => TomlValueType::DateTime,
			Self::OffsetDateTime(_) => TomlValueType::OffsetDateTime,
			Self::Array(_, _) => TomlValueType::Array,
			Self::Table(_) => TomlValueType::Table,
		}
	}

	/// Attempt to return the value as a string.
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(string) => Some(string.as_str()),
			_ => None,
		}
	}
	/// Attempt to return the value as an integer.
	pub fn as_integer(&self) -> Option<i64> {
		match self {
			Self::Integer(num) => Some(*num),
			_ => None,
		}
	}
	/// Attempt to return the value as a float.
	pub fn as_float(&self) -> Option<f64> {
		match self {
			Self::Float(num) => Some(*num),
			_ => None,
		}
	}
	/// Attempt to return the value as a bool.
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(bool) => Some(*bool),
			_ => None,
		}
	}
	/// Attempt to return the value as an array.
	pub fn as_array(&self) -> Option<&[Self]> {
		match self {
			Self::Array(array, _) => Some(array),
			_ => None,
		}
	}
	/// Attempt to return the value as a table.
	pub fn as_table(&self) -> Option<&TomlTable<'a>> {
		match self {
			Self::Table(table) => Some(table),
			_ => None,
		}
	}
	/// Attempt to return the value as a date.
	pub fn as_date(&self) -> Option<TomlDate> {
		match self {
			Self::Date(date) => Some(*date),
			_ => None,
		}
	}
	/// Attempt to return the value as a time.
	pub fn as_time(&self) -> Option<TomlTime> {
		match self {
			Self::Time(time) => Some(*time),
			_ => None,
		}
	}
	/// Attempt to return the value as a datetime.
	pub fn as_datetime(&self) -> Option<TomlDateTime> {
		match self {
			Self::DateTime(datetime) => Some(*datetime),
			_ => None,
		}
	}
	/// Attempt to return the value as an offset datetime.
	pub fn as_offset_datetime(&self) -> Option<OffsetTomlDateTime> {
		match self {
			Self::OffsetDateTime(offset_datetime) => Some(*offset_datetime),
			_ => None,
		}
	}

	/// Attempt to convert the value to a bool. This will return the value if
	/// it's a bool, and will also try to convert other types to a bool like so:
	/// - Strings: "true" and "True" are converted to true, "false" and "False"
	///   are converted to false
	/// - Integers and Floats: 1 is converted to true, 0 is converted to false
	pub fn coerce_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(bool) => Some(*bool),
			Self::String(str) => {
				let str = str.as_str();
				match str {
					"true" | "True" => Some(true),
					"false" | "False" => Some(false),
					_ => None,
				}
			}
			Self::Integer(int) => {
				if *int == 0 {
					Some(false)
				} else if *int == 1 {
					Some(true)
				} else {
					None
				}
			}
			Self::Float(float) => {
				if *float == 0.0 {
					Some(false)
				} else if *float == 1.0 {
					Some(true)
				} else {
					None
				}
			}
			_ => None,
		}
	}	
}

/// The basic value types in TOML. See [`TomlValue`] for descriptions of each
/// type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(missing_docs)]
pub enum TomlValueType {
	String,
	Integer,
	Float,
	Boolean,
	Time,
	Date,
	DateTime,
	OffsetDateTime,
	Array,
	Table,
}

/// An offset from UTC time.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TomlOffset {
	/// The hour and sign of the offset. The hour will be negative if the offset
	/// is negative.
	pub hour: i8,
	/// The minute of the offset.
	pub minute: u8,
}
/// A calendar date.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TomlDate {
	/// The date's year.
	pub year: u16,
	/// The date's month.
	pub month: u8,
	/// The day of the month.
	pub month_day: u8,
}
/// A time, with nanosecond precision.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TomlTime {
	/// The time's hour.
	pub hour: u8,
	/// The time's minute.
	pub minute: u8,
	/// The time's second.
	pub second: u8,
	/// The time's fractional second, stored in nanoseconds.
	pub nanosecond: u32,
}
/// A date and time value.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TomlDateTime {
	/// See [`TomlDate`].
	pub date: TomlDate,
	/// See [`TomlTime`].
	pub time: TomlTime,
}
/// A date and time value, offset to a specific timezone.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OffsetTomlDateTime {
	/// See [`TomlOffset`].
	pub offset: TomlOffset,
	/// See [`TomlDate`].
	pub date: TomlDate,
	/// See [`TomlTime`].
	pub time: TomlTime,
}

#[cfg(any(test, feature = "chrono"))]
mod chrono_into_from {
	use {
		super::*,
		chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime},
	};

	impl TryInto<NaiveDate> for TomlDate {
		type Error = ();

		fn try_into(self) -> Result<NaiveDate, Self::Error> {
			NaiveDate::from_ymd_opt(self.year.into(), self.month.into(), self.month_day.into())
				.ok_or(())
		}
	}
	impl TryInto<NaiveTime> for TomlTime {
		type Error = ();

		fn try_into(self) -> Result<NaiveTime, Self::Error> {
			NaiveTime::from_hms_nano_opt(
				self.hour.into(),
				self.minute.into(),
				self.second.into(),
				self.nanosecond,
			)
			.ok_or(())
		}
	}
	impl TryInto<FixedOffset> for TomlOffset {
		type Error = ();

		fn try_into(self) -> Result<FixedOffset, Self::Error> {
			let hour: i32 = self.hour.into();
			let mut minute: i32 = self.minute.into();
			minute *= hour.signum();

			FixedOffset::east_opt(
				hour.checked_mul(60).ok_or(())?.checked_mul(60).ok_or(())?
					+ minute.checked_mul(60).ok_or(())?,
			)
			.ok_or(())
		}
	}

	impl TryInto<NaiveDateTime> for TomlDateTime {
		type Error = ();

		fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
			let date: NaiveDate = self.date.try_into()?;
			Ok(date.and_time(self.time.try_into()?))
		}
	}
	impl TryInto<DateTime<FixedOffset>> for OffsetTomlDateTime {
		type Error = ();

		fn try_into(self) -> Result<DateTime<FixedOffset>, Self::Error> {
			let offset: FixedOffset = self.offset.try_into()?;

			let date: NaiveDate = self.date.try_into()?;
			let datetime = date.and_time(self.time.try_into()?);

			datetime.and_local_timezone(offset).single().ok_or(())
		}
	}
}
