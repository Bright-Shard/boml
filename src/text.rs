use std::ops::{Bound, RangeBounds};

/// Represents a text input to be parsed.
#[derive(Debug)]
pub struct Text<'a> {
	/// The text to be parsed.
	pub text: &'a str,
	/// The next byte that needs to be parsed.
	pub idx: usize,
}
impl<'a: 'b, 'b> Text<'a> {
	/// Creates a [`Span`] from the range provided to this method.
	pub fn excerpt<R: RangeBounds<usize>>(&self, range: R) -> Span<'b> {
		let start = match range.start_bound() {
			Bound::Excluded(start) => start - 1,
			Bound::Included(start) => *start,
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Excluded(end) => end - 1,
			Bound::Included(end) => *end,
			Bound::Unbounded => self.text.len() - 1,
		};

		Span {
			start,
			end,
			source: self.text,
		}
	}

	/// Gets a byte at `idx` from the input text.
	pub fn byte(&self, idx: usize) -> Option<u8> {
		self.text.as_bytes().get(idx).copied()
	}
	/// Gets the byte at `self.idx` from the input text.
	#[inline(always)]
	pub fn current_byte(&self) -> Option<u8> {
		self.byte(self.idx)
	}

	/// The number of remaining bytes in the text, not including the current byte.
	#[inline]
	pub fn remaining_bytes(&self) -> usize {
		self.text.len() - self.idx - 1
	}

	/// The last valid index into the text.
	pub fn end(&self) -> usize {
		self.text.len() - 1
	}

	/// Increments `self.idx` until it hits a non-whitespace character.
	pub fn skip_whitespace(&mut self) {
		while let Some(byte) = self.current_byte() {
			match byte {
				b'\t' | b' ' => self.idx += 1,
				_ => break,
			}
		}
	}
	/// Increments `self.idx` until it hits a non-whitespace, non-newline character.
	pub fn skip_whitespace_and_newlines(&mut self) {
		while let Some(byte) = self.current_byte() {
			match byte {
				b'\t' | b' ' | b'\n' | b'\r' => self.idx += 1,
				_ => break,
			}
		}
	}
}

/// A specific section of text from [`Text`].
#[derive(Debug, PartialEq)]
pub struct Span<'a> {
	/// Inclusive start of this span of text.
	pub start: usize,
	/// Inclusive end of this span of text.
	pub end: usize,
	/// The entire text this span is extracted from.
	pub source: &'a str,
}
impl<'a: 'b, 'b> Span<'a> {
	/// Finds the location of a character in this span, and returns its location,
	/// relative to the entire text this span comes from.
	pub fn find(&self, val: u8) -> Option<usize> {
		for (idx, char_) in self.as_str().bytes().enumerate() {
			if char_ == val {
				return Some(idx + self.start);
			}
		}

		None
	}

	// Finds the start of the next whitespace or newline, and returns its location,
	/// relative to the entire text this span comes from.
	pub fn find_next_whitespace_or_newline(&self) -> Option<usize> {
		let end = self.source.len();
		let space_idx = self.find(b' ').unwrap_or(end);
		let tab_idx = self.find(b'\t').unwrap_or(end);
		let mut newline_idx = self.find(b'\n').unwrap_or(end);

		// CRLF compat
		if self.source.as_bytes()[newline_idx - 1] == b'\r' {
			newline_idx -= 1;
		}

		let nearest_whitespace = if space_idx < tab_idx {
			space_idx
		} else {
			tab_idx
		};

		let nearest = if nearest_whitespace < newline_idx {
			nearest_whitespace
		} else {
			newline_idx
		};

		if nearest == end {
			None
		} else {
			Some(nearest)
		}
	}

	/// The number of bytes in this span of text.
	#[inline]
	pub fn len(&self) -> usize {
		(self.end - self.start) + 1
	}
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// A string covering just the bytes within this span.
	#[inline]
	pub fn as_str(&self) -> &str {
		&self.source[self.start..=self.end]
	}
	/// Identical to [`Span::as_str`], but it consumes `self`.
	#[inline]
	pub fn to_str(self) -> &'b str {
		&self.source[self.start..=self.end]
	}
}
