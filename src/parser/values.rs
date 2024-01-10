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

    if first == '\'' {
        if last == '\'' {
            text.idx = value.end;
            Some(Ok(Value::String(&text.text[value.start + 1..value.end])))
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
