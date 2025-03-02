//! Parses TOML values.

use std::f64;

use crate::{table::TomlTable, text::Text, types::TomlValue, TomlError, TomlErrorKind};

pub fn parse_value<'a>(text: &mut Text<'a>) -> Result<TomlValue<'a>, TomlError<'a>> {
	match text.current_byte() {
		Some(b'\'') | Some(b'"') => {
			crate::parser::string::parse_string(text).map(TomlValue::String)
		}
		Some(b'[') => {
			let start = text.idx();
			text.next();

			let mut array = Vec::new();

			loop {
				text.skip_whitespace();
				if text.current_byte() == Some(b']') {
					text.next();
					break;
				} else if text.current_byte().is_none() {
					return Err(TomlError {
						src: text.excerpt_before_idx(start..),
						kind: TomlErrorKind::UnclosedArrayBracket,
					});
				}

				array.push(parse_value(text)?);
				text.skip_whitespace();

				match text.current_byte() {
					Some(b',') => text.next(),
					Some(b']') => {
						text.next();
						break;
					}
					None => {
						return Err(TomlError {
							src: text.excerpt_before_idx(start..),
							kind: TomlErrorKind::UnclosedArrayBracket,
						})
					}
					_ => {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::UnclosedArrayBracket,
						})
					}
				}
				text.skip_whitespace();
			}

			Ok(TomlValue::Array(array, false))
		}
		Some(b'{') => {
			let start = text.idx();
			text.next();

			let mut table = TomlTable::default();

			loop {
				text.skip_whitespace();
				if text.current_byte() == Some(b'}') {
					text.next();
					break;
				} else if text.current_byte().is_none() {
					return Err(TomlError {
						src: text.excerpt_before_idx(start..),
						kind: TomlErrorKind::UnclosedInlineTableBracket,
					});
				}

				let entry = table.value_entry(text)?;
				text.skip_whitespace();

				if text.current_byte() != Some(b'=') {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::NoEqualsInAssignment,
					});
				}
				text.next();
				text.skip_whitespace();

				entry.insert(parse_value(text)?);
				text.skip_whitespace();

				match text.current_byte() {
					Some(b',') => text.next(),
					Some(b'}') => {
						text.next();
						break;
					}
					None => {
						return Err(TomlError {
							src: text.excerpt_before_idx(start..),
							kind: TomlErrorKind::UnclosedInlineTableBracket,
						})
					}
					_ => {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::UnclosedInlineTableBracket,
						});
					}
				}
				text.skip_whitespace();
			}

			Ok(TomlValue::Table(table))
		}
		Some(b't') if text.local_excerpt(..4).try_as_str() == Some("true") => {
			text.next_n(4);
			Ok(TomlValue::Boolean(true))
		}
		Some(b'f') if text.local_excerpt(..5).try_as_str() == Some("false") => {
			text.next_n(5);
			Ok(TomlValue::Boolean(false))
		}
		Some(b'+') | Some(b'-') => crate::parser::num::parse_sign(text),
		Some(b'i') if text.local_excerpt(..3).try_as_str() == Some("inf") => {
			text.next_n(3);
			Ok(TomlValue::Float(f64::INFINITY))
		}
		Some(b'n') if text.local_excerpt(..3).try_as_str() == Some("nan") => {
			text.next_n(3);
			Ok(TomlValue::Float(f64::NAN))
		}
		Some(i) if i.is_ascii_digit() => crate::parser::num::parse_number(text, false),
		_ => Err(TomlError {
			src: text.absolute_excerpt(text.idx()..=text.idx()),
			kind: TomlErrorKind::UnrecognisedValue,
		}),
	}
}
