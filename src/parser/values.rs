use crate::crate_prelude::*;

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

    let mut chars = value.as_str().chars();
    let first = chars.next().unwrap();
    let last = chars.next_back().unwrap();

    if value.len() >= 6 {
        let second = chars.next().unwrap();
        let second_last = chars.next_back().unwrap();
        let third = chars.next().unwrap();
        let third_last = chars.next_back().unwrap();

        if first == '\'' && second == '\'' && third == '\'' {
            if last == '\'' && second_last == '\'' && third_last == '\'' {
                todo!("Multiline literal strings");
            } else {
                return Some(Err(Error {
                    start: value.start,
                    end: value.end,
                    kind: ErrorKind::UnclosedStringLiteral,
                }));
            }
        } else if first == '"' && second == '"' && third == '"' {
            if last == '"' && second_last == '"' && third_last == '"' {
                todo!("Multiline strings");
            } else {
                return Some(Err(Error {
                    start: value.start,
                    end: value.end,
                    kind: ErrorKind::UnclosedStringLiteral,
                }));
            }
        }
    }

    if first == '\'' {
        if last == '\'' {
            text.idx = value.end;
            Some(Ok(Value::String(
                &text.text[value.start + 1..value.end - 1],
            )))
        } else {
            Some(Err(Error {
                start: value.start,
                end: value.end,
                kind: ErrorKind::UnclosedStringLiteral,
            }))
        }
    } else if first == '"' {
        if last == '"' {
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
