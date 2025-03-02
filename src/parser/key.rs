//! Parses bare and quoted keys.

use {
	crate::{
		table::TomlTable,
		text::{CowSpan, Text},
		types::TomlValue,
		TomlError, TomlErrorKind,
	},
	std::{collections::hash_map::Entry, hint::unreachable_unchecked},
};

const VALID_BARE_KEY_CHARS: &[u8] =
	b"ABCDEFGHIJKLMNOPQRSTUVWXYZ-abcdefghijklmnopqrstuvwxyz_0123456789";

pub fn parse_key<'a>(text: &mut Text<'a>) -> Result<CowSpan<'a>, TomlError<'a>> {
	if text.current_byte() == Some(b'\'') {
		return crate::parser::string::parse_literal_string(text).map(CowSpan::Raw);
	} else if text.current_byte() == Some(b'"') {
		return crate::parser::string::parse_basic_string(text);
	}

	let start = text.idx();
	while text
		.current_byte()
		.map(|byte| VALID_BARE_KEY_CHARS.contains(&byte))
		== Some(true)
	{
		text.next();
	}

	if text.idx() == start {
		return Err(TomlError {
			src: text.absolute_excerpt(start..=start),
			kind: TomlErrorKind::NoKeyInAssignment,
		});
	}

	let key = text.absolute_excerpt(start..text.idx());
	Ok(CowSpan::Raw(key))
}

pub fn parse_nested<'a, 't>(
	text: &mut Text<'a>,
	mut root: &'t mut TomlTable<'a>,
) -> Result<(&'t mut TomlTable<'a>, CowSpan<'a>), TomlError<'a>> {
	let start = text.idx();

	loop {
		let key = parse_key(text)?;
		text.skip_whitespace();

		if text.current_byte() != Some(b'.') {
			return Ok((root, key));
		}
		text.next();
		text.skip_whitespace();

		let entry = root.map.entry(key);

		if let Entry::Occupied(entry) = entry {
			root = match entry.into_mut() {
				TomlValue::Table(table) => table,
				TomlValue::Array(array, true) => {
					let Some(TomlValue::Table(table)) = array.last_mut() else {
						unreachable!()
					};
					table
				}
				_ => {
					return Err(TomlError {
						src: text.absolute_excerpt(start..text.idx() - 2),
						kind: TomlErrorKind::ReusedKey,
					});
				}
			};
		} else {
			let Entry::Vacant(entry) = entry else {
				unsafe { unreachable_unchecked() }
			};
			let TomlValue::Table(table) = entry.insert(TomlValue::Table(TomlTable::default()))
			else {
				unsafe { unreachable_unchecked() }
			};
			root = table;
		}
	}
}
