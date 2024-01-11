use super::crate_prelude::*;

pub mod values;

/// Parses and returns a bare key. The index must be at the first character of the key.
pub fn parse_bare_key<'a>(text: &mut Text<'a>) -> Result<Span<'a>, Error> {
    let start_idx = text.idx;
    let mut key = text.excerpt(start_idx..);

    if let Some(equals_idx) = key.find(b'=') {
        key.end = equals_idx - 1;
        key.trim();

        if key.is_empty() {
            return Err(Error {
                start: start_idx,
                end: equals_idx,
                kind: ErrorKind::NoKeyInAssignment,
            });
        }

        // Verify the key is entirely one of `A-Za-z0-9_-`, as per the TOML specs
        for (idx, char_) in key.as_str().bytes().enumerate() {
            if !char_.is_ascii_alphanumeric() && char_ != b'_' && char_ != b'-' {
                return Err(Error {
                    start: idx + key.start,
                    end: idx + key.start,
                    kind: ErrorKind::InvalidBareKey,
                });
            }
        }

        text.idx = equals_idx;
        Ok(key)
    } else {
        // If the `=` isn't found, boml emits an error around the bare key. It tries to locate the bare key by:
        // 1. Finding where it ends with a space
        // 2. Finding the end of the current line
        // 3. If all else fails, error until the end of the input string
        let end = text
            .find(' ')
            .or_else(|| text.find('\n'))
            .unwrap_or(key.end);

        Err(Error {
            start: start_idx,
            end,
            kind: ErrorKind::NoEqualsInAssignment,
        })
    }
}

/// Parses and returns a quoted key. The index must be at the opening quote of the key.
pub fn parse_quoted_key<'a>(text: &mut Text<'a>) -> Result<Span<'a>, Error> {
    let start_idx = text.idx;
    let quote = text.byte(start_idx).unwrap();
    let mut key = text.excerpt(start_idx + 1..);

    let Some(end_idx) = key.find(quote) else {
        // If the closing qute isn't found, boml emits an error around the key. It tries to locate the key by:
        // 1. Finding where it ends with a space
        // 2. Finding the end of the current line
        // 3. If all else fails, error until the end of the input string
        let end = text
            .find(' ')
            .or_else(|| text.find('\n'))
            .unwrap_or(key.end);

        return Err(Error {
            start: start_idx,
            end,
            kind: ErrorKind::UnclosedQuotedKey,
        });
    };
    let end_idx = end_idx - 1;

    // The value parser expects to be at the `=` sign, so the index needs to be set to there
    let Some(equals_idx) = key.find(b'=') else {
        return Err(Error {
            start: start_idx,
            end: end_idx,
            kind: ErrorKind::NoEqualsInAssignment,
        });
    };
    text.idx = equals_idx;

    key.end = end_idx;
    key.trim();

    Ok(key)
}

/// Parses a value in a key/value assignment. The index must be at the `=` symbol.
pub fn parse_value<'a>(text: &mut Text<'a>) -> Result<TomlData<'a>, Error> {
    let mut value_src = text.excerpt(text.idx + 1..);
    let line_end = value_src.find(b'\n').unwrap_or(value_src.end);
    value_src.end = line_end;
    value_src.trim();

    if value_src.is_empty() {
        return Err(Error {
            start: text.idx + 1,
            end: line_end,
            kind: ErrorKind::NoValueInAssignment,
        });
    }

    let Some(value) = values::try_parse_bool(text, &value_src)
        .map(Ok)
        .or_else(|| values::try_parse_int(text, &mut value_src))
        .or_else(|| values::try_parse_string(text, &value_src))
    else {
        todo!("Ints, floats, tables, dates, times, datetimes");
    };

    text.idx += 1;

    // TODO: Check for extraneous data

    Ok(TomlData {
        value: value?,
        source: value_src,
    })
}
