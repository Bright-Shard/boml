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

            return Some(Ok(Value::String(value.to_str())));
        } else if &value.as_str()[0..3] == "\"\"\"" {
            todo!("Multiline strings");
        }
    }

    if first == b'\'' {
        if last == b'\'' {
            text.idx = value.end;
            Some(Ok(Value::String(&text.text[value.start + 1..value.end])))
        } else {
            Some(Err(Error {
                start: value.start,
                end: value.end,
                kind: ErrorKind::UnclosedStringLiteral,
            }))
        }
    } else if first == b'"' {
        if last == b'"' {
            todo!("Regular strings - need to add escape characters");
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
