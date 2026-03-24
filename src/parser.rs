use std::{collections::hash_map::Entry, hint::unreachable_unchecked};

use crate::{
	Toml, TomlError, TomlErrorKind,
	table::TomlTable,
	text::Text,
	types::{TomlArray, TomlValue, TomlValueType},
};

pub mod inline_table;
pub mod key;
pub mod num;
pub mod string;
pub mod time;
pub mod value;

pub fn parse_str<'a>(str: &'a str) -> Result<Toml<'a>, TomlError<'a>> {
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
					match entry {
						Entry::Vacant(_) => {}
						Entry::Occupied(ref o)
							if o.get().as_array().is_some_and(|a| a.is_array_of_tables) => {}
						_ => {
							return Err(TomlError {
								src: text.excerpt_before_idx(start..),
								kind: TomlErrorKind::ReusedKey,
							});
						}
					}
					let TomlValue::Array(array) = entry.or_insert(TomlValue::Array(TomlArray {
						values: Vec::new(),
						is_array_of_tables: true,
					})) else {
						unreachable!()
					};

					let mut table = TomlTable::default();
					parse(text, &mut table, false)?;
					array.values.push(TomlValue::Table(table));
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
							});
						}
					};
					let TomlValue::Table(table) = table else {
						unsafe { unreachable_unchecked() }
					};

					if table.defined {
						return Err(TomlError {
							src: text.absolute_excerpt(start..text.idx()),
							kind: TomlErrorKind::ReusedKey,
						});
					}
					table.defined = true;

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
