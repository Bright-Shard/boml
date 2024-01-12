use {crate::crate_prelude::*, std::num::IntErrorKind};

/// Tries to parse a string literal. This can be a literal string, regular string,
/// multiline string, or multiline literal string. Returns `None` if no string was detected,
/// otherwise `Some` and either the string or a parsing error.
pub fn try_parse_string<'a>(
    text: &mut Text<'a>,
    value: &Span<'_>,
) -> Option<Result<Value<'a>, Error>> {
    if value.len() < 2 {
        return None;
    }

    let chars = value.as_str().as_bytes();
    let first = chars[0];
    let last = chars[chars.len() - 1];

    if value.len() >= 6 {
        if &value.as_str()[0..3] == "'''" {
            let snippet = text.excerpt(value.start + 3..);
            let Some(end_idx) = snippet.as_str().find("'''").map(|idx| idx + snippet.start) else {
                return Some(Err(Error {
                    start: value.start,
                    end: value.end,
                    kind: ErrorKind::UnclosedStringLiteral,
                }));
            };

            let value = Span {
                start: value.start + 3,
                end: end_idx - 1,
                source: text.text,
            };
            text.idx = value.end + 3;

            return Some(Ok(Value::LiteralString(value.to_str())));
        } else if &value.as_str()[0..3] == "\"\"\"" {
            let snippet = text.excerpt(value.start + 3..);
            let Some(end_idx) = snippet
                .as_str()
                .find("\"\"\"")
                .map(|idx| idx + snippet.start)
            else {
                return Some(Err(Error {
                    start: value.start,
                    end: value.end,
                    kind: ErrorKind::UnclosedStringLiteral,
                }));
            };

            let value = Span {
                start: value.start + 3,
                end: end_idx - 1,
                source: text.text,
            };
            text.idx = value.end + 3;
            return handle_basic_string_escapes(text, value.as_str())
                .or(Some(Ok(Value::LiteralString(value.to_str()))));
        }
    }

    if first == b'\'' {
        if last == b'\'' {
            text.idx = value.end;
            Some(Ok(Value::LiteralString(
                &text.text[value.start + 1..value.end],
            )))
        } else {
            Some(Err(Error {
                start: value.start,
                end: value.end,
                kind: ErrorKind::UnclosedStringLiteral,
            }))
        }
    } else if first == b'"' {
        if last == b'"' {
            let str_span = Span {
                start: value.start + 1,
                end: value.end - 1,
                source: text.text,
            }
            .to_str();
            text.idx = value.end;
            handle_basic_string_escapes(text, str_span).or(Some(Ok(Value::LiteralString(str_span))))
        } else {
            Some(Err(Error {
                start: value.start,
                end: value.end,
                kind: ErrorKind::UnclosedStringLiteral,
            }))
        }
    } else {
        None
    }
}

fn handle_basic_string_escapes<'a>(
    text: &mut Text<'a>,
    string: &str,
) -> Option<Result<Value<'a>, Error>> {
    if string.contains('\\') {
        let mut neostring = String::with_capacity(string.len());
        let mut last_idx_copy = 0;
        let mut bytes = string.bytes().enumerate().peekable();

        while let Some((idx, byte)) = bytes.next() {
            if byte == b'\\' {
                let Some((_, next_byte)) = bytes.next() else {
                    return Some(Err(Error {
                        start: idx + 1,
                        end: idx + 2,
                        kind: ErrorKind::UnknownEscapeSequence,
                    }));
                };
                let replace_with = match next_byte {
                    b'b' => '\u{0008}',
                    b't' => '\t',
                    b'n' => '\n',
                    b'f' => '\u{000C}',
                    b'r' => '\r',
                    b'"' => '"',
                    b'\\' => '\\',
                    b'u' => {
                        let Some(char_) = text
                            .excerpt(idx + 2..idx + 6)
                            .as_str()
                            .parse()
                            .ok()
                            .and_then(char::from_u32)
                        else {
                            return Some(Err(Error {
                                start: idx,
                                end: idx + 5,
                                kind: ErrorKind::UnknownUnicodeScalar,
                            }));
                        };

                        char_
                    }
                    b'U' => {
                        let Some(char_) = text
                            .excerpt(idx + 2..idx + 10)
                            .as_str()
                            .parse()
                            .ok()
                            .and_then(char::from_u32)
                        else {
                            return Some(Err(Error {
                                start: idx,
                                end: idx + 9,
                                kind: ErrorKind::UnknownUnicodeScalar,
                            }));
                        };

                        char_
                    }
                    c if c.is_ascii_whitespace() => {
                        neostring.push_str(&string[last_idx_copy..idx]);
                        last_idx_copy = idx + 1;
                        while let Some((_, byte)) = bytes.peek() {
                            last_idx_copy += 1;

                            if !byte.is_ascii_whitespace() {
                                break;
                            }

                            bytes.next();
                        }
                        continue;
                    }
                    _ => {
                        return Some(Err(Error {
                            start: idx,
                            end: idx + 1,
                            kind: ErrorKind::UnknownEscapeSequence,
                        }))
                    }
                };

                neostring.push_str(&string[last_idx_copy..idx]);
                neostring.push(replace_with);
                last_idx_copy = idx + 2;
            }
        }
        if last_idx_copy < string.len() {
            neostring.push_str(&string[last_idx_copy..]);
        }

        Some(Ok(Value::BasicString(neostring)))
    } else {
        None
    }
}

pub fn try_parse_bool<'a>(text: &mut Text<'a>, value: &Span<'_>) -> Option<Value<'a>> {
    let val = value.as_str();

    if val == "true" {
        text.idx = value.end;
        Some(Value::Boolean(true))
    } else if val == "false" {
        text.idx = value.end;
        Some(Value::Boolean(false))
    } else {
        None
    }
}

pub fn try_parse_int<'a>(
    text: &mut Text<'a>,
    value: &mut Span<'_>,
) -> Option<Result<Value<'a>, Error>> {
    let negative = match text.byte(value.start).unwrap() {
        b'-' => {
            value.start += 1;
            true
        }
        b'+' => {
            value.start += 1;
            false
        }
        _ => false,
    };
    let radix = match text.byte(value.start) {
        Some(b'0') => match text.byte(value.start + 1) {
            Some(b'x') => {
                value.start += 2;
                16
            }
            Some(b'o') => {
                value.start += 2;
                8
            }
            Some(b'b') => {
                value.start += 2;
                2
            }
            Some(_) => {
                return Some(Err(Error {
                    start: value.start,
                    end: value.start + 1,
                    kind: ErrorKind::NumberHasInvalidBaseOrLeadingZero,
                }))
            }
            None => 10,
        },
        Some(_) => 10,
        // This can only be reached if there was a sign with nothing after it
        // The `parse_value` fn errors if the value is empty, so the only way to get `None`
        // here is if the sign check above increments `value.start`, which would mean there's
        // a sign but nothing after it.
        None => {
            return Some(Err(Error {
                start: value.start - 1,
                end: value.end,
                kind: ErrorKind::InvalidNumber,
            }))
        }
    };

    match i64::from_str_radix(value.as_str(), radix) {
        Ok(mut num) => {
            if negative {
                num *= -1;
            }
            text.idx = value.end;
            Some(Ok(Value::Integer(num)))
        }
        Err(e) => match *e.kind() {
            IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => Some(Err(Error {
                start: value.start,
                end: value.end,
                kind: ErrorKind::NumberTooLarge,
            })),
            IntErrorKind::InvalidDigit => None,
            _ => unreachable!(),
        },
    }
}

pub fn try_parse_float<'a>(text: &mut Text<'a>, value: &mut Span<'_>) -> Option<Value<'a>> {
    match value.as_str().parse::<f64>().map(Value::Float) {
        Ok(num) => {
            text.idx = value.end;
            Some(num)
        }
        // Rust currently doesn't give us information about why a parse failed. The error enum
        // contains 2 variants - 1 for an empty string and 1 for an invalid float. So we can't
        // really tell why the parse failed.
        Err(_) => None,
    }
}
