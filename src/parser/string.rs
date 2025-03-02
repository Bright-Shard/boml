//! Parses single-line and multi-line strings, handling string escapes
//! automatically.

use crate::{
	text::{CowSpan, Span, Text},
	TomlError, TomlErrorKind,
};

pub fn parse_string<'a>(text: &mut Text<'a>) -> Result<CowSpan<'a>, TomlError<'a>> {
	match text.current_byte() {
		Some(b'\'') => {
			if text.local_excerpt(..3).try_as_str() == Some("'''") {
				parse_multiline_literal_string(text).map(CowSpan::Raw)
			} else {
				parse_literal_string(text).map(CowSpan::Raw)
			}
		}
		Some(b'"') => {
			if text.local_excerpt(..3).try_as_str() == Some(r#"""""#) {
				parse_multiline_basic_string(text)
			} else {
				parse_basic_string(text)
			}
		}
		_ => unreachable!(),
	}
}

fn string_escape<'a, const MULTILINE: bool>(
	string: &mut Vec<u8>,
	text: &mut Text<'a>,
) -> Result<(), TomlError<'a>> {
	debug_assert_eq!(text.current_byte(), Some(b'\\'));

	let start = text.idx();
	text.next();

	match text.current_byte() {
		Some(byte) => string.push(match byte {
			b'b' => {
				text.next();
				0x08
			}
			b't' => {
				text.next();
				b'\t'
			}
			b'n' => {
				text.next();
				b'\n'
			}
			b'f' => {
				text.next();
				0x0C
			}
			b'r' => {
				text.next();
				b'\r'
			}
			b'"' => {
				text.next();
				b'"'
			}
			b'\\' => {
				text.next();
				b'\\'
			}
			b'u' => {
				if text.remaining_bytes() < 4 {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::UnknownUnicodeScalar,
					});
				}
				text.next();

				let Some(char) = u32::from_str_radix(text.local_excerpt(..4).as_str(), 16)
					.ok()
					.and_then(char::from_u32)
				else {
					return Err(TomlError {
						src: text.absolute_excerpt(start..start + 6),
						kind: TomlErrorKind::UnknownUnicodeScalar,
					});
				};
				text.next_n(4);

				string.extend_from_slice(char.encode_utf8(&mut [0u8; 4]).as_bytes());
				return Ok(());
			}
			b'U' => {
				if text.remaining_bytes() < 8 {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::UnknownUnicodeScalar,
					});
				}
				text.next();

				let Some(char) = u32::from_str_radix(text.local_excerpt(..8).as_str(), 16)
					.ok()
					.and_then(char::from_u32)
				else {
					return Err(TomlError {
						src: text.absolute_excerpt(start..start + 10),
						kind: TomlErrorKind::UnknownUnicodeScalar,
					});
				};
				text.next_n(8);

				string.extend_from_slice(char.encode_utf8(&mut [0u8; 4]).as_bytes());
				return Ok(());
			}
			other if MULTILINE && other.is_ascii_whitespace() => {
				text.skip_whitespace_allow_comments();
				return Ok(());
			}
			_ => {
				return Err(TomlError {
					src: text.excerpt_to_idx(start..),
					kind: TomlErrorKind::UnknownEscapeSequence,
				})
			}
		}),
		None => {
			return Err(TomlError {
				src: text.excerpt_before_idx(start..),
				kind: TomlErrorKind::UnknownEscapeSequence,
			})
		}
	}

	Ok(())
}

pub fn parse_basic_string<'a>(text: &mut Text<'a>) -> Result<CowSpan<'a>, TomlError<'a>> {
	debug_assert_eq!(text.current_byte(), Some(b'"'));

	text.next();
	let start = text.idx();

	while let Some(byte) = text.current_byte() {
		if byte == b'"' {
			let string = text.absolute_excerpt(start..text.idx());
			text.next();
			return Ok(CowSpan::Raw(string));
		} else if byte == b'\\' {
			let mut string = text
				.absolute_excerpt(start..text.idx())
				.as_str()
				.as_bytes()
				.to_vec();

			string_escape::<false>(&mut string, text)?;

			while let Some(byte) = text.current_byte() {
				if byte == b'"' {
					let span = text.absolute_excerpt(start..text.idx());
					text.next();
					return Ok(CowSpan::Modified(span, String::from_utf8(string).unwrap()));
				} else if byte == b'\\' {
					string_escape::<false>(&mut string, text)?;
				} else {
					string.push(byte as _);
					text.next();
				}
			}
		}

		text.next();
	}

	Err(TomlError {
		src: text.excerpt_before_idx(start..),
		kind: TomlErrorKind::UnclosedBasicString,
	})
}
pub fn parse_multiline_basic_string<'a>(text: &mut Text<'a>) -> Result<CowSpan<'a>, TomlError<'a>> {
	debug_assert_eq!(text.local_excerpt(..3).as_str(), "\"\"\"");

	let start = text.idx();
	text.next_n(3);
	let mut string_start = start + 3;

	if text.current_byte() == Some(b'\n') {
		text.next();
		string_start += 1;
	}

	while let Some(byte) = text.current_byte() {
		if byte == b'"' && text.local_excerpt(..3).try_as_str() == Some(r#"""""#) {
			let mut string = text.absolute_excerpt(string_start..text.idx());
			text.next_n(3);
			if text.current_byte() == Some(b'"') {
				string.end += 1;
				text.next();
			}
			if text.current_byte() == Some(b'"') {
				string.end += 1;
				text.next();
			}
			return Ok(CowSpan::Raw(string));
		} else if byte == b'\\' {
			let mut string = text
				.absolute_excerpt(string_start..text.idx())
				.as_str()
				.as_bytes()
				.to_vec();

			string_escape::<true>(&mut string, text)?;

			while let Some(byte) = text.current_byte() {
				if byte == b'"' && text.local_excerpt(..3).try_as_str() == Some(r#"""""#) {
					let span = text.absolute_excerpt(string_start..text.idx());
					text.next_n(3);
					if text.current_byte() == Some(b'"') {
						string.push(b'"');
						text.next();
					}
					if text.current_byte() == Some(b'"') {
						string.push(b'"');
						text.next();
					}
					return Ok(CowSpan::Modified(span, String::from_utf8(string).unwrap()));
				} else if byte == b'\\' {
					string_escape::<true>(&mut string, text)?;
				} else {
					string.push(byte);
					text.next();
				}
			}
		}

		text.next();
	}

	Err(TomlError {
		src: text.excerpt_before_idx(start..),
		kind: TomlErrorKind::UnclosedBasicString,
	})
}

pub fn parse_literal_string<'a>(text: &mut Text<'a>) -> Result<Span<'a>, TomlError<'a>> {
	debug_assert_eq!(text.current_byte(), Some(b'\''));

	text.next();
	let start = text.idx();

	while let Some(byte) = text.current_byte() {
		if byte == b'\'' {
			let string = text.absolute_excerpt(start..text.idx());
			text.next();
			return Ok(string);
		}
		text.next();
	}

	Err(TomlError {
		src: text.excerpt_before_idx(start..),
		kind: TomlErrorKind::UnclosedLiteralString,
	})
}
pub fn parse_multiline_literal_string<'a>(text: &mut Text<'a>) -> Result<Span<'a>, TomlError<'a>> {
	debug_assert_eq!(text.local_excerpt(..3).as_str(), "'''");

	let start = text.idx();
	text.next_n(3);
	let mut string_start = start + 3;

	if text.current_byte() == Some(b'\n') {
		text.next();
		string_start += 1;
	}

	while let Some(byte) = text.current_byte() {
		if byte == b'\'' && text.local_excerpt(..3).try_as_str() == Some("'''") {
			let mut string = text.absolute_excerpt(string_start..text.idx());
			text.next_n(3);
			if text.current_byte() == Some(b'\'') {
				string.end += 1;
				text.next();
			}
			if text.current_byte() == Some(b'\'') {
				string.end += 1;
				text.next();
			}
			return Ok(string);
		}
		text.next();
	}

	Err(TomlError {
		src: text.excerpt_before_idx(start..),
		kind: TomlErrorKind::UnclosedLiteralString,
	})
}
