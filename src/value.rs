use {crate::crate_prelude::*, std::collections::HashMap};

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    BasicString(String),
    LiteralString(&'a str),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array(&'a [Self]),
    Table(HashMap<&'a str, Self>),
}

#[derive(Debug)]
pub struct TomlData<'a> {
    pub value: Value<'a>,
    pub source: Span<'a>,
}
