//! Parsers for each part of TOML - keys, values, and arrays. Each parser is only responsible
//! for the length of the data it parses. Extraneous whitespace, comments, or invalid characters
//! fall outside the scope of the parsers. Parsers assume that the current index in the [`Text`]
//! is the first character of what they should parse - ie, the first letter of a key, opening
//! quote of a quoted key, opening bracket of a table, etc. Each parser should leave `text.idx`
//! at the last byte of the value it parsed.

use {crate::crate_prelude::*, std::num::IntErrorKind};

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

pub fn parse_key<'a>(text: &mut Text<'a>) -> Result<Key<'a>, Error> {
    match text.current_byte().unwrap() {
        b'\'' | b'"' => parse_string(text).map(|text| Key { text, child: None }),
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

            let span = text.excerpt(start..current);

            // Check for dotted key
            text.idx = current;
            text.skip_whitespace();
            if text.current_byte() == Some(b'.') {
                text.idx += 1;
                text.skip_whitespace();

                Ok(Key {
                    text: TomlString::Raw(span),
                    child: Some(Box::new(parse_key(text)?)),
                })
            } else {
                text.idx = current - 1;
                Ok(Key {
                    text: TomlString::Raw(span),
                    child: None,
                })
            }
        }
    }
}

pub fn parse_value<'a>(text: &mut Text<'a>) -> Result<TomlValue<'a>, Error> {
    match text.current_byte().unwrap() {
        // Integer, time, or float
        b'0'..=b'9' => {
            let mut span = text.excerpt(text.idx..);
            if let Some(end_idx) = span.find_next_whitespace_or_newline() {
                span.end = end_idx - 1;
            }

            // Integer with custom radix
            let radix = if text.remaining_bytes() > 1 && text.current_byte().unwrap() == b'0' {
                match text.byte(text.idx + 1).unwrap() {
                    b'b' => Some(2),
                    b'o' => Some(8),
                    b'x' => Some(16),
                    _ => None,
                }
            } else {
                None
            };
            if radix.is_some() {
                span.start += 2;
            }

            let source = if span.find(b'_').is_some() {
                let mut string = String::with_capacity(span.len());
                for char_ in span.as_str().chars() {
                    if char_ != '_' {
                        string.push(char_);
                    }
                }

                TomlString::Formatted(span, string)
            } else {
                TomlString::Raw(span)
            };
            let span = source.span();

            // Float
            if radix.is_none()
                && (span.find(b'.').is_some()
                    || span.find(b'e').is_some()
                    || span.find(b'E').is_some())
            {
                // Unfortunately, the f64 parser doesn't give detailed error information, so this is the best we can do.
                if let Ok(num) = source.parse() {
                    text.idx = span.end;
                    return Ok(TomlValue::Float(num));
                }
            }

            // Time
            if radix.is_none() && (span.find(b'-').is_some() || span.find(b':').is_some()) {
                todo!("Dates/times")
            }

            // Integer
            match i64::from_str_radix(source.as_str(), radix.unwrap_or(10)) {
                Ok(num) => {
                    text.idx = span.end;
                    return Ok(TomlValue::Integer(num));
                }
                Err(e) => match e.kind() {
                    IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                        return Err(Error {
                            start: span.start,
                            end: span.end,
                            kind: ErrorKind::NumberTooLarge,
                        });
                    }
                    IntErrorKind::InvalidDigit => {}
                    _ => unreachable!(),
                },
            }

            let span = text.excerpt(text.idx - 1..);
            Err(Error {
                start: span.start,
                end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
                kind: ErrorKind::UnrecognisedValue,
            })
        }

        // Infinity/NaN float
        b'i' if text.remaining_bytes() >= 2 => {
            let span = text.excerpt(text.idx..text.idx + 3);
            if span.as_str() == "inf" {
                text.idx = span.end;
                Ok(TomlValue::Float(f64::INFINITY))
            } else {
                let span = text.excerpt(text.idx - 1..);
                Err(Error {
                    start: span.start,
                    end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
                    kind: ErrorKind::UnrecognisedValue,
                })
            }
        }
        b'n' if text.remaining_bytes() >= 2 => {
            let span = text.excerpt(text.idx..text.idx + 3);
            if span.as_str() == "nan" {
                text.idx = span.end;
                Ok(TomlValue::Float(f64::NAN))
            } else {
                let span = text.excerpt(text.idx - 1..);
                Err(Error {
                    start: span.start,
                    end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
                    kind: ErrorKind::UnrecognisedValue,
                })
            }
        }

        // Integer or float with +/- modifier
        b'+' if text.remaining_bytes() > 0 => {
            text.idx += 1;
            parse_value(text)
        }
        b'-' if text.remaining_bytes() > 0 => {
            text.idx += 1;

            match parse_value(text) {
                Ok(val) => match val {
                    TomlValue::Integer(num) => Ok(TomlValue::Integer(-num)),
                    TomlValue::Float(num) => Ok(TomlValue::Float(-num)),
                    _ => {
                        let span = text.excerpt(text.idx - 1..);
                        Err(Error {
                            start: span.start,
                            end: span.find_next_whitespace_or_newline().unwrap_or(text.end()),
                            kind: ErrorKind::UnrecognisedValue,
                        })
                    }
                },
                Err(mut e) => {
                    e.end -= 1;
                    Err(e)
                }
            }
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

            text.idx += 1;

            loop {
                text.skip_whitespace_and_newlines();

                // Trailing comma support
                if let Some(b']') = text.current_byte() {
                    break;
                }

                let value = parse_value(text)?;
                array.push(value);
                span.end = text.idx;

                text.idx += 1;
                text.skip_whitespace_and_newlines();
                match text.current_byte() {
                    Some(b']') => break,
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

pub fn parse_string<'a>(text: &mut Text<'a>) -> Result<TomlString<'a>, Error> {
    let mut span = text.excerpt(text.idx..);

    match text.current_byte().unwrap() {
        b'\'' => {
            let (end, offset) = if text.remaining_bytes() > 5
                && text.excerpt(text.idx..text.idx + 3).to_str() == "'''"
            {
                // Multi-line string
                span.start += 3;
                (span.as_str().find("'''").map(|idx| span.start + idx), 3)
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

            Ok(TomlString::Raw(span))
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

            text.idx = span.end + offset;

            if span.find(b'\\').is_some() {
                handle_basic_string_escapes(text, span)
            } else {
                Ok(TomlString::Raw(span))
            }
        }
        _ => unreachable!(),
    }
}

fn find_basic_string_end(span: &mut Span<'_>, text: &Text<'_>, multiline: bool) -> Option<usize> {
    let end = if multiline {
        // Multi-line string
        span.start += 3;
        span.as_str().find("\"\"\"").map(|idx| span.start + idx)
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

fn handle_basic_string_escapes<'a>(
    text: &Text<'a>,
    span: Span<'a>,
) -> Result<TomlString<'a>, Error> {
    let mut string = String::with_capacity(span.len());

    let mut chars = span.as_str().char_indices().peekable();
    while let Some((idx, char_)) = chars.next() {
        if char_ == '\\' {
            let Some((_, char_)) = chars.next() else {
                return Err(Error {
                    start: span.start + idx,
                    end: span.start + idx,
                    kind: ErrorKind::UnknownEscapeSequence,
                });
            };

            let to_push = match char_ {
                'b' => '\u{0008}',
                't' => '\t',
                'n' => '\n',
                'f' => '\u{000C}',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                'u' => {
                    let Some(char_) = text
                        .excerpt(idx + 2..idx + 6)
                        .as_str()
                        .parse()
                        .ok()
                        .and_then(char::from_u32)
                    else {
                        return Err(Error {
                            start: idx,
                            end: idx + 5,
                            kind: ErrorKind::UnknownUnicodeScalar,
                        });
                    };

                    char_
                }
                'U' => {
                    let Some(char_) = text
                        .excerpt(idx + 2..idx + 10)
                        .as_str()
                        .parse()
                        .ok()
                        .and_then(char::from_u32)
                    else {
                        return Err(Error {
                            start: idx,
                            end: idx + 9,
                            kind: ErrorKind::UnknownUnicodeScalar,
                        });
                    };

                    char_
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

        string.push(char_);
    }

    Ok(TomlString::Formatted(span, string))
}
