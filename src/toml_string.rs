use {
	crate::crate_prelude::*,
	std::{borrow::Borrow, fmt::Display, hash::Hash, ops::Deref},
};

/// Stores either a `String` or `&str`, and behaves like an `&str`.
///
/// This is necessary because some basic strings in TOML have escapes,
/// which requires formatting, which requires copying and then modifying
/// the original string. This results in a `String`.
///
/// However, literal strings and basic strings without escapes don't need
/// that formatting, and turning them into a `String` requires copying, which
/// BOML tries to avoid. Thus, they end up as `&str`s.
///
/// This enum can store either a formatted `String` or an unformatted `&str`,
/// and then behave exactly like an `&str`.
#[derive(Debug)]
pub enum TomlString<'a> {
	/// A formatted `String`. Used for basic strings with escapes.
	Formatted(Span<'a>, String),
	/// An unformatted `&str`. Used for literal strings and basic strings
	/// without escapes.
	Raw(Span<'a>),
}
impl<'a> TomlString<'a> {
	#[inline]
	pub fn as_str(&self) -> &str {
		self.borrow()
	}

	/// Returns the original span of text this string came from. For formatted
	/// strings, this span is the *original* text, and thus won't be formatted,
	/// meaning the string's contents are different from the span's contents.
	pub fn span(&self) -> &Span<'a> {
		match self {
			Self::Formatted(span, _) => span,
			Self::Raw(span) => span,
		}
	}
}
impl<'a> Borrow<str> for TomlString<'a> {
	fn borrow(&self) -> &str {
		match self {
			TomlString::Formatted(_, ref string) => string.as_str(),
			TomlString::Raw(span) => span.as_str(),
		}
	}
}
impl<'a> Deref for TomlString<'a> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.borrow()
	}
}
impl<'a> PartialEq for TomlString<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}
impl<'a> Eq for TomlString<'a> {}
impl<'a> Hash for TomlString<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}
impl<'a> Display for TomlString<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_str().fmt(f)
	}
}
