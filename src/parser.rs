use std::{collections::hash_map::Entry, hint::unreachable_unchecked};

use crate::{
	table::TomlTable,
	text::Text,
	types::{TomlValue, TomlValueType},
	Toml, TomlError, TomlErrorKind,
};

pub mod inline_table;
pub mod key;
pub mod num;
pub mod string;
pub mod time;
pub mod value;

pub fn parse_str(str: &str) -> Result<Toml<'_>, TomlError> {
	let mut txt = Text::new(str);
	let mut root = TomlTable::default();

	parse(&mut txt, &mut root, true)?;

	Ok(Toml {
		source: str,
		table: root,
	})
}

pub fn parse<'a>(
	text: &mut Text<'a>,
	current_table: &mut TomlTable<'a>,
	is_root: bool,
) -> Result<(), TomlError<'a>> {
	text.skip_whitespace();
	while let Some(byte) = text.current_byte() {
		match byte {
			// Table or array of tables
			b'[' => {
				if !is_root {
					return Ok(());
				}

				let start = text.idx();
				if text.next_byte() == Some(b'[') {
					text.next_n(2);

					text.skip_whitespace();

					let (table, key) = crate::parser::key::parse_nested(text, current_table)?;

					text.skip_whitespace();

					if text.current_byte() != Some(b']') || text.next_byte() != Some(b']') {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::UnclosedArrayOfTablesBracket,
						});
					}
					text.next_n(2);
					text.skip_whitespace();

					let entry = table.map.entry(key.clone());

					let mut table = TomlTable::default();
					parse(text, &mut table, false)?;

					let value_entry = entry.or_insert(TomlValue::Array(Vec::new(), true));
					let TomlValue::Array(ref mut array, _) = value_entry else {
						return Err(TomlError {
							src: text.excerpt_before_idx(start..),
							kind: TomlErrorKind::ReusedKey,
						});
					};
					array.push(TomlValue::Table(table));
				} else {
					text.next();
					text.skip_whitespace();

					let (table, key) = crate::parser::key::parse_nested(text, current_table)?;
					let mut entry = table.map.entry(key);
					let table = match entry {
						Entry::Occupied(ref mut entry)
							if entry.get().ty() == TomlValueType::Table =>
						{
							entry.get_mut()
						}
						Entry::Vacant(entry) => {
							entry.insert(TomlValue::Table(TomlTable::default()))
						}
						_ => {
							return Err(TomlError {
								src: text.excerpt_to_idx(start..),
								kind: TomlErrorKind::ReusedKey,
							})
						}
					};
					let TomlValue::Table(table) = table else {
						unsafe { unreachable_unchecked() }
					};

					text.skip_whitespace();

					if text.current_byte() != Some(b']') {
						return Err(TomlError {
							src: text.excerpt_to_idx(start..),
							kind: TomlErrorKind::UnclosedTableBracket,
						});
					}
					text.next();
					text.skip_whitespace();

					parse(text, table, false)?;
				}
			}
			// Key assignment
			_ => {
				let start = text.idx();

				let entry = current_table.value_entry(text)?;
				text.skip_whitespace();

				if text.current_byte() != Some(b'=') {
					return Err(TomlError {
						src: text.excerpt_to_idx(start..),
						kind: TomlErrorKind::NoEqualsInAssignment,
					});
				}
				text.next();
				text.skip_whitespace();

				entry.insert(crate::parser::value::parse_value(text)?);
			}
		}
		text.skip_whitespace();
	}

	Ok(())
}
