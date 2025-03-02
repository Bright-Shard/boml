use {
	crate::{text::Text, types::TomlValue, TomlError, TomlErrorKind},
	core::mem::MaybeUninit,
};

fn is_end_of_int(byte: u8) -> bool {
	byte.is_ascii_whitespace() || b",.]}#".contains(&byte)
}
fn is_end_of_float(byte: u8) -> bool {
	byte.is_ascii_whitespace() || b",]}#".contains(&byte)
}

// TODO: This doesn't prevent parsing date/times with a sign in front, which
// isn't valid TOML
pub fn parse_sign<'a>(text: &mut Text<'a>) -> Result<TomlValue<'a>, TomlError<'a>> {
	match text.current_byte() {
		Some(b'+') => {
			text.next();
			parse_number(text, false)
		}
		Some(b'-') => {
			text.next();
			parse_number(text, true)
		}
		_ => unreachable!(),
	}
}

pub fn parse_number<'a>(
	text: &mut Text<'a>,
	negative: bool,
) -> Result<TomlValue<'a>, TomlError<'a>> {
	let start = text.idx();

	if text.current_byte() == Some(b'0') {
		text.next();
		return match text.current_byte() {
			Some(b'x') => {
				text.next();
				parse_hex_int(text, negative).map(TomlValue::Integer)
			}
			Some(b'o') => {
				text.next();
				parse_oct_int(text, negative).map(TomlValue::Integer)
			}
			Some(b'b') => {
				text.next();
				parse_bin_int(text, negative).map(TomlValue::Integer)
			}
			Some(b'.') | Some(b'e') | Some(b'E') => {
				parse_float(text, start, negative).map(TomlValue::Float)
			}
			Some(other) if other.is_ascii_digit() => {
				let mut num: u16 = (other - b'0').into();
				text.next();

				while let Some(byte) = text.current_byte() {
					if (text.idx() - start) > 4 {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::NumberHasLeadingZero,
						});
					} else if !byte.is_ascii_digit() {
						match (text.idx() - start, byte) {
							(2, b':') => {
								return crate::parser::time::parse_time(num as u8, start, text)
									.map(TomlValue::Time)
							}
							(4, b'-') => return crate::parser::time::parse_date(num, start, text),
							_ => {
								return Err(TomlError {
									src: text.excerpt_to_idx(start..),
									kind: TomlErrorKind::NumberHasLeadingZero,
								})
							}
						}
					}

					num *= 10;
					num += (byte - b'0') as u16;
					text.next();
				}

				Err(TomlError {
					src: text.excerpt_to_idx(start..),
					kind: TomlErrorKind::NumberHasLeadingZero,
				})
			}
			None => Ok(TomlValue::Integer(0)),
			Some(b) if is_end_of_int(b) => Ok(TomlValue::Integer(0)),
			_ => Err(TomlError {
				src: text.excerpt_to_idx(start..),
				kind: TomlErrorKind::InvalidNumber,
			}),
		};
	}

	match text.current_byte() {
		Some(b'i') if text.local_excerpt(..3).try_as_str() == Some("inf") => {
			text.next_n(3);
			return Ok(TomlValue::Float(if negative {
				f64::NEG_INFINITY
			} else {
				f64::INFINITY
			}));
		}
		Some(b'n') if text.local_excerpt(..3).try_as_str() == Some("nan") => {
			text.next_n(3);
			return Ok(TomlValue::Float(if negative {
				-f64::NAN
			} else {
				f64::NAN
			}));
		}
		_ => {}
	}

	let mut running_num = 0i64;
	while let Some(byte) = text.current_byte() {
		match byte {
			b'_' => {}
			b'.' | b'e' | b'E' => return parse_float(text, start, negative).map(TomlValue::Float),
			b'-' => {
				if text.idx() - start == 4 {
					return crate::parser::time::parse_date((-running_num) as u16, start, text);
				} else {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::DateTimeTooManyDigits,
					});
				}
			}
			b':' => {
				if text.idx() - start == 2 {
					return crate::parser::time::parse_time((-running_num) as u8, start, text)
						.map(TomlValue::Time);
				} else {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::DateTimeTooManyDigits,
					});
				}
			}
			other if other.is_ascii_digit() => {
				running_num = match running_num.checked_mul(10) {
					Some(num) => num,
					None => {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::NumberTooLarge,
						})
					}
				};
				running_num = match running_num.checked_sub((other - b'0') as i64) {
					Some(num) => num,
					None => {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::NumberTooLarge,
						})
					}
				};
			}
			other if is_end_of_int(other) => break,
			_ => {
				return Err(TomlError {
					src: text.excerpt_to_idx(start..),
					kind: TomlErrorKind::InvalidNumber,
				});
			}
		}
		text.next();
	}

	let running_num = if negative {
		running_num
	} else if running_num.unsigned_abs() > i64::MAX as u64 {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::NumberTooLarge,
		});
	} else {
		-running_num
	};
	Ok(TomlValue::Integer(running_num))
}

fn parse_int_with_base<'a, const BASE: i64>(
	text: &mut Text<'a>,
	negative: bool,
	matcher: impl Fn(u8) -> Option<i64>,
) -> Result<i64, TomlError<'a>> {
	let start = text.idx() - 2;
	let mut running_int = 0i64;

	while let Some(byte) = text.current_byte() {
		text.next();
		if byte == b'_' {
			continue;
		} else if is_end_of_int(byte) {
			break;
		} else if let Some(num) = matcher(byte) {
			running_int = match running_int.checked_mul(BASE) {
				Some(num) => num,
				None => {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::NumberTooLarge,
					})
				}
			};
			running_int = match running_int.checked_sub(num) {
				Some(num) => num,
				None => {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::NumberTooLarge,
					})
				}
			}
		}
	}

	let running_int = if negative {
		running_int
	} else if running_int.unsigned_abs() > i64::MAX as u64 {
		return Err(TomlError {
			src: text.excerpt_to_idx(start..),
			kind: TomlErrorKind::NumberTooLarge,
		});
	} else {
		-running_int
	};

	Ok(running_int)
}
fn parse_hex_int<'a>(text: &mut Text<'a>, negative: bool) -> Result<i64, TomlError<'a>> {
	parse_int_with_base::<16>(text, negative, |byte| {
		Some(match byte {
			b'0' => 0,
			b'1' => 1,
			b'2' => 2,
			b'3' => 3,
			b'4' => 4,
			b'5' => 5,
			b'6' => 6,
			b'7' => 7,
			b'8' => 8,
			b'9' => 9,
			b'A' | b'a' => 10,
			b'B' | b'b' => 11,
			b'C' | b'c' => 12,
			b'D' | b'd' => 13,
			b'E' | b'e' => 14,
			b'F' | b'f' => 15,
			_ => return None,
		})
	})
}
fn parse_oct_int<'a>(text: &mut Text<'a>, negative: bool) -> Result<i64, TomlError<'a>> {
	parse_int_with_base::<8>(text, negative, |byte| {
		Some(match byte {
			b'0' => 0,
			b'1' => 1,
			b'2' => 2,
			b'3' => 3,
			b'4' => 4,
			b'5' => 5,
			b'6' => 6,
			b'7' => 7,
			_ => return None,
		})
	})
}
fn parse_bin_int<'a>(text: &mut Text<'a>, negative: bool) -> Result<i64, TomlError<'a>> {
	parse_int_with_base::<2>(text, negative, |byte| {
		Some(match byte {
			b'0' => 0,
			b'1' => 1,
			_ => return None,
		})
	})
}

// Float parsing is actually *really* complicated, so instead of trying to do it
// from scratch, we pass it off to the Rust compiler.
// We allocate a 768-byte buffer on the stack and copy all non-`_` bytes from
// the float to the buffer. Then we read the buffer as a string and parse it
// as an f64 using the standard `str::parse` method.
// `MaybeUninit` is used as an optimisation to avoid a `memset` call that would
// zero the buffer.
// The buffer is 768 bytes because, according to the standard library's float
// parser, that's the "maximum amount of digits required to unambiguously round
// a float" - see
// https://doc.rust-lang.org/src/core/num/dec2flt/decimal.rs.html#58.
fn parse_float<'a>(
	text: &mut Text<'a>,
	start: usize,
	negative: bool,
) -> Result<f64, TomlError<'a>> {
	const BUFFER_SIZE: usize = 768;

	let mut stack_buffer = MaybeUninit::<[u8; BUFFER_SIZE]>::uninit();
	let mut remaining: &mut [u8] = unsafe {
		core::slice::from_raw_parts_mut(stack_buffer.as_mut_ptr() as *mut u8, BUFFER_SIZE)
	};
	for (idx, byte) in text.absolute_excerpt(start..).as_str().bytes().enumerate() {
		if byte == b'_' {
			continue;
		}
		if is_end_of_float(byte) {
			let diff = (start + idx) - text.idx();
			text.next_n(diff);
			break;
		}
		remaining[0] = byte;
		remaining = &mut remaining[1..];
	}
	let remaining_len = remaining.len();
	let len = BUFFER_SIZE - remaining_len;

	let slice = unsafe { core::slice::from_raw_parts(stack_buffer.as_ptr() as *const u8, len) };
	let str = unsafe { std::str::from_utf8_unchecked(slice) };

	match str.parse::<f64>() {
		Ok(float) => Ok(if negative { -float } else { float }),
		Err(_) => Err(TomlError {
			src: text.absolute_excerpt(start..start + len),
			kind: TomlErrorKind::InvalidNumber,
		}),
	}
}
