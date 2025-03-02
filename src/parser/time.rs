use crate::{
	text::Text,
	types::{OffsetTomlDateTime, TomlDate, TomlDateTime, TomlOffset, TomlTime, TomlValue},
	TomlError, TomlErrorKind,
};

fn parse_two_digits(text: &mut Text) -> Option<u8> {
	let mut num = 0u8;

	let first = text.current_byte()?;
	if !first.is_ascii_digit() {
		return None;
	}
	num += first - b'0';
	num *= 10;
	text.next();

	let second = text.current_byte()?;
	if !second.is_ascii_digit() {
		return None;
	}
	num += second - b'0';
	text.next();

	Some(num)
}

pub fn parse_date<'a>(
	year: u16,
	start: usize,
	text: &mut Text<'a>,
) -> Result<TomlValue<'a>, TomlError<'a>> {
	debug_assert_eq!(text.current_byte(), Some(b'-'));
	text.next();

	let Some(month) = parse_two_digits(text) else {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::DateMissingMonth,
		});
	};
	if text.current_byte() != Some(b'-') {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::DateMissingDash,
		});
	}
	text.next();

	let Some(month_day) = parse_two_digits(text) else {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::DateMissingDay,
		});
	};

	let date = TomlDate {
		year,
		month,
		month_day,
	};

	if text.current_byte() == Some(b' ')
		|| text.current_byte() == Some(b'T')
		|| text.current_byte() == Some(b't')
	{
		text.next();

		if let Some(hour) = parse_two_digits(text) {
			let time = parse_time(hour, start, text)?;

			if text.current_byte() == Some(b'Z') || text.current_byte() == Some(b'z') {
				text.next();
				return Ok(TomlValue::OffsetDateTime(OffsetTomlDateTime {
					offset: TomlOffset { hour: 0, minute: 0 },
					date,
					time,
				}));
			} else if text.current_byte() == Some(b'+') || text.current_byte() == Some(b'-') {
				let negative = text.current_byte() == Some(b'-');
				text.next();

				let Some(hour) = parse_two_digits(text) else {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::TimeMissingMinute,
					});
				};
				if text.current_byte() != Some(b':') {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::OffsetMissingHour,
					});
				}
				text.next();

				let Some(minute) = parse_two_digits(text) else {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::OffsetMissingMinute,
					});
				};

				let hour = if negative { -(hour as i8) } else { hour as i8 };

				return Ok(TomlValue::OffsetDateTime(OffsetTomlDateTime {
					offset: TomlOffset { hour, minute },
					date,
					time,
				}));
			} else {
				return Ok(TomlValue::DateTime(TomlDateTime { date, time }));
			}
		}
	}

	Ok(TomlValue::Date(date))
}

pub fn parse_time<'a>(
	hour: u8,
	start: usize,
	text: &mut Text<'a>,
) -> Result<TomlTime, TomlError<'a>> {
	debug_assert_eq!(text.current_byte(), Some(b':'));
	text.next();

	let Some(minute) = parse_two_digits(text) else {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::TimeMissingMinute,
		});
	};
	if text.current_byte() != Some(b':') {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::TimeMissingColon,
		});
	}
	text.next();

	let Some(second) = parse_two_digits(text) else {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::TimeMissingSecond,
		});
	};

	let nanosecond = if text.current_byte() == Some(b'.') {
		const NANOSECOND_DIGIT: u8 = 8;
		text.next();

		let mut digits = 0u8;
		let mut nanosecond = 0u32;
		while text.current_byte().is_some() {
			let byte = text.current_byte().unwrap();

			if !byte.is_ascii_digit() {
				break;
			}

			nanosecond += (byte - b'0') as u32;
			nanosecond *= 10;
			digits += 1;
			text.next();

			// truncate past nanoseconds
			if digits > NANOSECOND_DIGIT {
				while text.current_byte().is_some() {
					let byte = text.current_byte().unwrap();
					if !byte.is_ascii_digit() {
						break;
					}
					text.next();
				}
				break;
			}
		}

		for _ in digits..NANOSECOND_DIGIT {
			nanosecond *= 10;
		}

		nanosecond
	} else {
		0
	};

	Ok(TomlTime {
		hour,
		minute,
		second,
		nanosecond,
	})
}

// Date and time tests with Chrono. If these pass the rest of the date/time
// types are assumed to be correct because they're built off the date and time
// types.
#[cfg(test)]
mod tests {
	use {
		super::*,
		chrono::{NaiveDate, NaiveTime},
	};

	#[test]
	fn date_tests() {
		fn test(year: u16, month: u8, day: u8) {
			let TomlValue::Date(toml_date) =
				parse_date(year, 0, &mut Text::new(&format!("-{month:02}-{day:02}"))).unwrap()
			else {
				unreachable!()
			};

			let chrono_date: NaiveDate = toml_date.try_into().unwrap();

			assert_eq!(
				chrono_date.to_string(),
				format!("{year:04}-{month:02}-{day:02}")
			);
		}

		test(2025, 2, 3);
		test(1999, 12, 23);
		test(100, 2, 15);
		test(13, 11, 9);
		test(1, 1, 1);
	}

	#[test]
	fn time_tests() {
		fn test(hour: u8, minute: u8, second: u8, nanosecond: u32) {
			let toml_time = parse_time(
				hour,
				0,
				&mut Text::new(&format!(":{minute:02}:{second:02}.{nanosecond}")),
			)
			.unwrap();

			let chrono_time: NaiveTime = toml_time.try_into().unwrap();

			let mut toml_time_formatted = format!("{hour:02}:{minute:02}:{second:02}");
			if nanosecond > 0 {
				toml_time_formatted += &format!(".{:0<3}", nanosecond / 1000)
			}

			assert_eq!(chrono_time.to_string(), toml_time_formatted);
		}

		test(20, 33, 25, 500_000);
		test(1, 3, 5, 5_000);
		test(20, 6, 7, 0);
		test(1, 13, 5, 55_000);
		test(1, 3, 35, 54_000);
	}
}
