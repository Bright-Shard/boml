//! Utilities BOML uses to parse text.

use std::{
	borrow::Borrow,
	fmt::{Debug, Display},
	hash::Hash,
	ops::{Bound, RangeBounds},
};

/// A helper struct used by BOML to parse strings.
#[derive(Debug)]
pub struct Text<'a> {
	/// The text to be parsed.
	pub text: &'a str,
	/// The next byte that needs to be parsed.
	idx: usize,
}
impl<'a> Text<'a> {
	pub fn new(text: &'a str) -> Self {
		Self { text, idx: 0 }
	}

	/// Creates a [`Span`] from the range provided to this method.
	pub fn absolute_excerpt<R: RangeBounds<usize>>(&self, range: R) -> Span<'a> {
		let start = match range.start_bound() {
			Bound::Excluded(start) => start.saturating_sub(1),
			Bound::Included(start) => *start,
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Excluded(end) => end.saturating_sub(1),
			Bound::Included(end) => *end,
			Bound::Unbounded => self.text.len().saturating_sub(1),
		};

		Span {
			start,
			end: end.min(self.text.len().saturating_sub(1)),
			source: self.text,
		}
	}
	/// [`Self::excerpt`], except it starts at the cursor instead of the start
	/// of the text.
	pub fn local_excerpt<R: RangeBounds<usize>>(&self, range: R) -> Span<'a> {
		let start = match range.start_bound() {
			Bound::Excluded(start) => self.idx + (start.saturating_sub(1)),
			Bound::Included(start) => self.idx + *start,
			Bound::Unbounded => self.idx,
		};
		let end = match range.end_bound() {
			Bound::Excluded(end) => self.idx + (end.saturating_sub(1)),
			Bound::Included(end) => self.idx + *end,
			Bound::Unbounded => self.text.len().saturating_sub(1),
		};

		Span {
			start,
			end: end.min(self.text.len().saturating_sub(1)),
			source: self.text,
		}
	}
	/// [`Self::excerpt`], except it ends at the cursor instead of the end
	/// of the text.
	pub fn excerpt_to_idx<R: RangeBounds<usize>>(&self, range: R) -> Span<'a> {
		let start = match range.start_bound() {
			Bound::Excluded(start) => start.saturating_sub(1),
			Bound::Included(start) => *start,
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Excluded(end) => self.idx + (end.saturating_sub(1)),
			Bound::Included(end) => self.idx + *end,
			Bound::Unbounded => self.idx,
		};

		Span {
			start,
			end: end.min(self.text.len().saturating_sub(1)),
			source: self.text,
		}
	}
	/// [`Self::excerpt`], except it ends before cursor instead of  at the end
	/// of the text.
	pub fn excerpt_before_idx<R: RangeBounds<usize>>(&self, range: R) -> Span<'a> {
		let start = match range.start_bound() {
			Bound::Excluded(start) => start.saturating_sub(1),
			Bound::Included(start) => *start,
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Excluded(end) => self.idx.saturating_sub(1) + (end.saturating_sub(1)),
			Bound::Included(end) => self.idx.saturating_sub(1) + *end,
			Bound::Unbounded => self.idx.saturating_sub(1),
		};

		Span {
			start,
			end: end.min(self.text.len().saturating_sub(1)),
			source: self.text,
		}
	}

	/// Read the current byte from the source text.
	pub fn current_byte(&self) -> Option<u8> {
		self.text.as_bytes().get(self.idx).copied()
	}
	/// Read the next byte from the source text. This does not progress the
	/// cursor.
	pub fn next_byte(&self) -> Option<u8> {
		self.text
			.as_bytes()
			.get((self.idx + 1).min(self.end()))
			.copied()
	}

	/// Moves the index ahead 1 byte.
	pub fn next(&mut self) {
		self.idx += 1;
	}
	/// Moves the index ahead n bytes.
	pub fn next_n(&mut self, n: usize) {
		self.idx += n;
	}
	/// The index of the current byte.
	pub fn idx(&self) -> usize {
		self.idx
	}

	/// The number of remaining bytes in the text, not including the current
	/// byte.
	#[inline]
	pub fn remaining_bytes(&self) -> usize {
		if self.idx >= self.text.len() {
			0
		} else {
			self.text.len() - self.idx - 1
		}
	}

	/// The last valid index into the text.
	pub fn end(&self) -> usize {
		if self.text.is_empty() {
			0
		} else {
			self.text.len() - 1
		}
	}

	/// Skips past all ASCII whitespace and any TOML comments.
	pub fn skip_whitespace(&mut self) {
		while let Some(byte) = self.current_byte() {
			match byte {
				b'\t' | b' ' | b'\n' | b'\r' => self.next(),
				_ => break,
			}
		}

		if self.current_byte() == Some(b'#') {
			self.skip_current_line();
			self.skip_whitespace();
		}
	}
	/// Skips past all ASCII whitespace, but not TOML comments.
	pub fn skip_whitespace_allow_comments(&mut self) {
		while let Some(byte) = self.current_byte() {
			match byte {
				b'\t' | b' ' | b'\n' | b'\r' => self.next(),
				_ => break,
			}
		}
	}
	/// Skip to the end of the current line.
	pub fn skip_current_line(&mut self) {
		while let Some(byte) = self.current_byte() {
			if byte == b'\n' {
				break;
			}
			self.next();
		}
	}
}

/// A region of text from a string.
#[derive(Clone, Copy)]
pub struct Span<'a> {
	/// Inclusive start of this span of text.
	pub start: usize,
	/// Inclusive end of this span of text.
	pub end: usize,
	/// The full text that this [`Span`] is an excerpt of.
	pub source: &'a str,
}
impl<'a> Span<'a> {
	/// The number of bytes in this span of text.
	#[inline]
	pub fn len(&self) -> usize {
		(self.end - self.start) + 1
	}
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	#[inline]
	pub fn try_as_str(&self) -> Option<&'a str> {
		if self.end >= self.source.len() {
			return None;
		}

		Some(&self.source[self.start..=self.end])
	}
	/// A string covering just the bytes within this span.
	#[inline]
	pub fn as_str(&self) -> &'a str {
		&self.source[self.start..=self.end]
	}
}
impl Debug for Span<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

/// Copy-on-write [`Span`]s. This is used for handling TOML strings, which may
/// require formatting.
#[derive(Clone)]
pub enum CowSpan<'a> {
	Raw(Span<'a>),
	Modified(Span<'a>, String),
}
impl CowSpan<'_> {
	/// Converts the `CowSpan` to a [`str`].
	#[inline]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Raw(ref raw) => &raw.source[raw.start..=raw.end],
			Self::Modified(_, ref modified) => modified,
		}
	}

	/// The [`Span`] for the original, unformatted string.
	#[inline]
	pub fn span(&self) -> &Span<'_> {
		match self {
			Self::Raw(ref span) => span,
			Self::Modified(ref span, _) => span,
		}
	}
}
impl Hash for CowSpan<'_> {
	#[inline(always)]
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}
impl Borrow<str> for CowSpan<'_> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}
impl PartialEq for CowSpan<'_> {
	#[inline(always)]
	fn eq(&self, other: &Self) -> bool {
		self.as_str().eq(other.as_str())
	}
}
impl Eq for CowSpan<'_> {}
impl Debug for CowSpan<'_> {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Raw(span) => {
				write!(f, "{span:?}")
			}
			Self::Modified(_, string) => {
				write!(f, "{string}",)
			}
		}
	}
}
impl Display for CowSpan<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
