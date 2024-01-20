//! Parsers for each part of TOML - keys, values, and arrays.
//!
//! Parser rules:
//! 1. Each parser is only responsible for the length of the data it parses. Extraneous whitespace,
//! comments, or invalid characters fall outside the scope of the parsers.
//! 2. Parsers assume that the current index in the [`Text`] is the first character of what they
//! should parse - ie, the first letter of a key, opening quote of a quoted key, opening bracket
//! of a table, etc.
//! 3. Each parser should leave `text.idx` at the last byte it parsed.

use {crate::crate_prelude::*, std::num::IntErrorKind};

/// Parses a `<key> = <value>` assignment.
pub fn parse_assignment<'a>(text: &mut Text<'a>) -> Result<(Key<'a>, TomlValue<'a>), Error> {
	let key = parse_key(text)?;

	text.idx += 1;
	text.skip_whitespace();
	if text.current_byte() != Some(b'=') {
		return Err(Error {
			start: key.text.span().start,
			end: text.idx,
			kind: ErrorKind::NoEqualsInAssignment,
		});
	}
	text.idx += 1;
	text.skip_whitespace();
	if text.idx >= text.end() {
		return Err(Error {
			start: key.text.span().start,
			end: text.idx,
			kind: ErrorKind::NoValueInAssignment,
		});
	}

	let value = parse_value(text)?;

	Ok((key, value))
}

/// Parses a key. Supports quoted, dotted, and bare keys.
pub fn parse_key<'a>(text: &mut Text<'a>) -> Result<Key<'a>, Error> {
	let maybe_key = match text.current_byte().unwrap() {
		b'\'' | b'"' => parse_string(text)?,
		_ => {
			let start = text.idx;
			let mut current = text.idx;

			while let Some(byte) = text.byte(current) {
				if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'_' {
					break;
				}

				current += 1;
			}

			if text.byte(current).is_none() {
				// Text shouldn't end on a key definition
				return Err(Error {
					start,
					end: current,
					kind: ErrorKind::NoValueInAssignment,
				});
			}
			if start == current {
				// Empty bare keys are not allowed
				return Err(Error {
					start,
					end: current,
					kind: ErrorKind::InvalidBareKey,
				});
			}

			let span = text.excerpt(start..current);
			text.idx = current - 1;

			CowSpan::Raw(span)
		}
	};

	// Check for dotted key
	let key_end = text.idx;
	text.idx += 1;
	text.skip_whitespace();
	if text.current_byte() == Some(b'.') {
		text.idx += 1;
		text.skip_whitespace();

		Ok(Key {
			text: maybe_key,
			child: Some(Box::new(parse_key(text)?)),
		})
	} else {
		text.idx = key_end;
		Ok(Key {
			text: maybe_key,
			child: None,
		})
	}
}

/// Parses a value. Supports all of the non-time-related value types.
pub fn parse_value<'a>(text: &mut Text<'a>) -> Result<TomlValue<'a>, Error> {
	match text.current_byte().unwrap() {
		// Integer, time, or float
		b'0'..=b'9' | b'i' | b'n' => parse_num(text, false),

		// Integer or float with +/- modifier
		b'+' if text.remaining_bytes() > 0 => {
			text.idx += 1;

			parse_num(text, false)
		}
		b'-' if text.remaining_bytes() > 0 => {
			text.idx += 1;

			parse_num(text, true)
		}

		// String
		b'\'' | b'"' => parse_string(text).map(TomlValue::String),

		// Bool
		b't' | b'f' if text.remaining_bytes() >= 3 => {
			let span = text.excerpt(text.idx..text.idx + 4);
			if span.as_str() == "true" {
				text.idx = span.end;
				return Ok(TomlValue::Boolean(true));
			} else if span.as_str() == "fals" && text.byte(text.idx + 4) == Some(b'e') {
				text.idx = span.end + 1;
				return Ok(TomlValue::Boolean(false));
			}

			let span = text.excerpt(text.idx..);
			Err(Error {
				start: span.start,
				end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
				kind: ErrorKind::UnrecognisedValue,
			})
		}

		// Array
		b'[' => {
			if text.remaining_bytes() == 0 {
				return Err(Error {
					start: text.idx,
					end: text.idx,
					kind: ErrorKind::UnclosedBracket,
				});
			}

			let mut array = Vec::new();
			let mut span = text.excerpt(text.idx..);
			let mut seen_comma = true;
			text.idx += 1;

			loop {
				text.skip_whitespace_and_newlines();

				match text.current_byte() {
					Some(b']') => break,
					Some(b',') => {
						text.idx += 1;
						text.skip_whitespace_and_newlines();
						if text.remaining_bytes() == 0 {
							return Err(Error {
								start: span.start,
								end: text.idx,
								kind: ErrorKind::UnclosedBracket,
							});
						}

						seen_comma = true;
						continue;
					}
					Some(b'#') => {
						text.idx = text.excerpt(text.idx..).find(b'\n').unwrap_or(text.end());
						text.skip_whitespace_and_newlines();

						continue;
					}
					Some(_) if !seen_comma => {
						return Err(Error {
							start: text.idx,
							end: text.idx,
							kind: ErrorKind::NoCommaDelimeter,
						})
					}
					Some(_) => {}
					None => {
						return Err(Error {
							start: span.start,
							end: text.idx,
							kind: ErrorKind::UnclosedBracket,
						})
					}
				}

				let value = parse_value(text)?;
				array.push(value);
				span.end = text.idx;

				text.idx += 1;
				seen_comma = false;
			}

			Ok(TomlValue::Array(array))
		}

		// Inline table
		b'{' => {
			if text.remaining_bytes() == 0 {
				return Err(Error {
					start: text.idx,
					end: text.idx,
					kind: ErrorKind::UnclosedBracket,
				});
			}

			let mut table = Table::default();
			let mut span = text.excerpt(text.idx..);

			text.idx += 1;

			loop {
				text.skip_whitespace();

				// Empty table
				if text.current_byte() == Some(b'}') {
					break;
				}

				let (key, value) = parse_assignment(text)?;
				let start = key.text.span().start;
				let end = key.text.span().end;

				let old_value = table.insert(key, value);
				if old_value {
					return Err(Error {
						start,
						end,
						kind: ErrorKind::ReusedKey,
					});
				}
				span.end = text.idx;

				text.idx += 1;
				text.skip_whitespace();
				match text.current_byte() {
					Some(b'}') => break,
					Some(b',') => {}
					Some(_) => {
						return Err(Error {
							start: text.idx,
							end: text.idx,
							kind: ErrorKind::NoCommaDelimeter,
						})
					}
					None => {
						return Err(Error {
							start: span.start,
							end: span.end,
							kind: ErrorKind::UnclosedBracket,
						})
					}
				}

				text.idx += 1;
			}

			Ok(TomlValue::Table(table))
		}

		// ¯\_(ツ)_/¯
		_ => {
			let span = text.excerpt(text.idx..);
			Err(Error {
				start: span.start,
				end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
				kind: ErrorKind::UnrecognisedValue,
			})
		}
	}
}

fn parse_num<'a>(text: &mut Text<'a>, negative: bool) -> Result<TomlValue<'a>, Error> {
	let mut span = Span {
		start: text.idx,
		end: text.idx,
		source: text.text,
	};

	// inf or nan
	let current_byte = text.current_byte().unwrap();
	if (current_byte == b'i' || current_byte == b'n') && text.remaining_bytes() >= 2 {
		span.end += 2;
		if span.as_str() == "inf" {
			text.idx = span.end;
			if negative {
				return Ok(TomlValue::Float(-f64::INFINITY));
			} else {
				return Ok(TomlValue::Float(f64::INFINITY));
			}
		} else if span.as_str() == "nan" {
			text.idx = span.end;
			if negative {
				return Ok(TomlValue::Float(-f64::NAN));
			} else {
				return Ok(TomlValue::Float(f64::NAN));
			}
		}
	}

	let mut has_underscores = false;
	let mut is_float = false;
	let mut is_time = false;

	// Custom radix
	let radix = if current_byte == b'0' {
		match text.byte(span.end + 1) {
			Some(b'b') => {
				span.end += 1;
				while let Some(byte) = text.byte(span.end + 1) {
					if byte == b'0' || byte == b'1' {
						span.end += 1;
					} else if byte == b'_' {
						has_underscores = true;
						span.end += 1;
					} else {
						break;
					}
				}

				Some(2)
			}
			Some(b'o') => {
				span.end += 1;
				while let Some(byte) = text.byte(span.end + 1) {
					match byte {
						b'0'..=b'7' => span.end += 1,
						b'_' => {
							has_underscores = true;
							span.end += 1;
						}
						_ => break,
					}
				}

				Some(8)
			}
			Some(b'x') => {
				span.end += 1;
				while let Some(byte) = text.byte(span.end + 1) {
					match byte {
						b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => span.end += 1,
						b'_' => {
							has_underscores = true;
							span.end += 1;
						}
						_ => break,
					}
				}

				Some(16)
			}
			_ => None,
		}
	} else {
		None
	};

	if radix.is_none() {
		let mut has_dash = false;

		while let Some(byte) = text.byte(span.end + 1) {
			match byte {
				b'0'..=b'9' => {}

				b'.' | b'e' | b'E' | b'+' => is_float = true,

				b':' => is_time = true,

				// Can be in floats (1e-4) and time (1974-12-03)
				b'-' => has_dash = true,

				b'_' => has_underscores = true,

				_ => break,
			}
			span.end += 1;
		}

		if is_float && is_time {
			return Err(Error {
				start: span.start,
				end: span.end,
				kind: ErrorKind::InvalidNumber,
			});
		} else if !is_float && has_dash {
			is_time = true;
		}
	}

	if radix.is_some() {
		span.start += 2;
	}
	text.idx = span.end;

	let source = if has_underscores {
		let mut string = String::with_capacity(span.len());
		for char_ in span.as_str().chars() {
			if char_ != '_' {
				string.push(char_);
			}
		}

		CowSpan::Modified(span, string)
	} else {
		CowSpan::Raw(span)
	};
	let span = source.span();

	if is_float {
		// Unfortunately, the f64 parser doesn't give detailed error information, so this is the best we can do.
		if let Ok(num) = source.as_str().parse::<f64>() {
			if negative {
				return Ok(TomlValue::Float(-num));
			} else {
				return Ok(TomlValue::Float(num));
			}
		}
	}

	if is_time && !negative {
		todo!("Time types")
	}

	match i64::from_str_radix(source.as_str(), radix.unwrap_or(10)) {
		Ok(num) => {
			if negative {
				return Ok(TomlValue::Integer(-num));
			} else {
				return Ok(TomlValue::Integer(num));
			}
		}
		Err(e) => match e.kind() {
			IntErrorKind::PosOverflow => {
				// i64::MIN, as a string, without the sign
				if negative && source.as_str() == "9223372036854775808" {
					return Ok(TomlValue::Integer(i64::MIN));
				}

				return Err(Error {
					start: span.start,
					end: span.end,
					kind: ErrorKind::NumberTooLarge,
				});
			}
			IntErrorKind::InvalidDigit => {}
			IntErrorKind::Empty => {
				return Err(Error {
					start: span.start,
					end: span.end,
					kind: ErrorKind::InvalidNumber,
				})
			}
			_ => unreachable!(),
		},
	}

	Err(Error {
		start: span.start,
		end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
		kind: ErrorKind::UnrecognisedValue,
	})
}

/// Parses a string. Supports literal and basic strings. Handles basic string escapes
/// automatically.
pub fn parse_string<'a>(text: &mut Text<'a>) -> Result<CowSpan<'a>, Error> {
	let mut span = text.excerpt(text.idx..);

	match text.current_byte().unwrap() {
		b'\'' => {
			let (end, offset) = if text.remaining_bytes() > 5
				&& text.excerpt(text.idx..text.idx + 3).to_str() == "'''"
			{
				// Multi-line string
				span.start += 3;
				if text.byte(span.start).unwrap() == b'\n' {
					span.start += 1;
				}
				(
					span.as_str().find("'''").map(|idx| {
						let mut idx = span.start + idx;

						while text.byte(idx) == Some(b'\'') {
							idx += 1;
						}

						idx - 3
					}),
					3,
				)
			} else {
				// Single-line string
				span.start += 1;
				(span.find(b'\''), 1)
			};

			let Some(end) = end else {
				return Err(Error {
					start: text.idx,
					end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
					kind: ErrorKind::UnclosedString,
				});
			};
			span.end = end - 1;
			text.idx = span.end + offset;

			Ok(CowSpan::Raw(span))
		}
		b'"' => {
			let multiline = text.remaining_bytes() > 5
				&& text.excerpt(text.idx..text.idx + 3).to_str() == "\"\"\"";
			let offset = if multiline { 3 } else { 1 };
			let start = span.start;

			let Some(end) = find_basic_string_end(&mut span, text, multiline) else {
				return Err(Error {
					start: text.idx,
					end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
					kind: ErrorKind::UnclosedString,
				});
			};
			span.start = start + offset;
			span.end = end - 1;

			if multiline && text.byte(span.start).unwrap() == b'\n' {
				span.start += 1;
			}

			text.idx = span.end + offset;

			if span.find(b'\\').is_some() {
				handle_basic_string_escapes(text, span)
			} else {
				Ok(CowSpan::Raw(span))
			}
		}
		_ => unreachable!(),
	}
}

fn find_basic_string_end(span: &mut Span<'_>, text: &Text<'_>, multiline: bool) -> Option<usize> {
	let end = if multiline {
		// Multi-line string
		span.start += 3;
		span.as_str().find("\"\"\"").map(|idx| {
			let mut idx = span.start + idx;

			while text.byte(idx) == Some(b'"') {
				idx += 1;
			}

			idx - 3
		})
	} else {
		// Single-line string
		span.start += 1;
		span.find(b'"')
	};

	if let Some(end) = end {
		if text.byte(end - 1).unwrap() == b'\\' && text.byte(end - 2).unwrap() != b'\\' {
			span.start = end;
			find_basic_string_end(span, text, multiline)
		} else {
			Some(end)
		}
	} else {
		None
	}
}

fn handle_basic_string_escapes<'a>(text: &Text<'a>, span: Span<'a>) -> Result<CowSpan<'a>, Error> {
	let mut string = String::with_capacity(span.len());

	let mut chars = span.as_str().char_indices().peekable();
	while let Some((idx, char)) = chars.next() {
		let idx = span.start + idx;
		if char == '\\' {
			let Some((idx, char)) = chars.next() else {
				return Err(Error {
					start: idx,
					end: idx,
					kind: ErrorKind::UnknownEscapeSequence,
				});
			};
			let idx = span.start + idx;

			let to_push = match char {
				'b' => '\u{0008}',
				't' => '\t',
				'n' => '\n',
				'f' => '\u{000C}',
				'r' => '\r',
				'"' => '"',
				'\\' => '\\',
				'u' => {
					if idx + 4 > text.end() {
						return Err(Error {
							start: idx,
							end: idx + 4,
							kind: ErrorKind::UnknownUnicodeScalar,
						});
					}

					let source = text.excerpt(idx + 1..=idx + 4);
					let Some(char) = u32::from_str_radix(source.as_str(), 16)
						.ok()
						.and_then(char::from_u32)
					else {
						return Err(Error {
							start: idx,
							end: idx + 5,
							kind: ErrorKind::UnknownUnicodeScalar,
						});
					};

					chars.nth(3).unwrap();

					char
				}
				'U' => {
					if idx + 8 > text.end() {
						return Err(Error {
							start: idx,
							end: idx + 8,
							kind: ErrorKind::UnknownUnicodeScalar,
						});
					}

					let source = text.excerpt(idx + 1..=idx + 8);
					let Some(char) = u32::from_str_radix(source.as_str(), 16)
						.ok()
						.and_then(char::from_u32)
					else {
						return Err(Error {
							start: idx,
							end: idx + 8,
							kind: ErrorKind::UnknownUnicodeScalar,
						});
					};

					chars.nth(7).unwrap();

					char
				}
				' ' | '\t' | '\n' | '\r' => {
					while let Some((_, char_)) = chars.peek() {
						let char_ = *char_;
						if char_ != ' ' && char_ != '\t' && char_ != '\n' && char_ != '\r' {
							break;
						}
						chars.next();
					}
					continue;
				}
				_ => {
					return Err(Error {
						start: span.start + idx,
						end: span.start + idx + 1,
						kind: ErrorKind::UnknownEscapeSequence,
					})
				}
			};

			string.push(to_push);
			continue;
		}

		string.push(char);
	}

	Ok(CowSpan::Modified(span, string))
}
