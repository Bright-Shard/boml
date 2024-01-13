use crate::crate_prelude::*;

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
    Array(Vec<&'a Self>),
    Table(Table<'a>),
}
impl<'a> Value<'a> {
    pub fn ty(&self) -> ValueType {
        match *self {
            Self::BasicString(_) | Self::LiteralString(_) => ValueType::String,
            Self::Integer(_) => ValueType::Integer,
            Self::Float(_) => ValueType::Float,
            Self::Boolean(_) => ValueType::Boolean,
            Self::OffsetDateTime => ValueType::OffsetDateTime,
            Self::LocalDateTime => ValueType::LocalDateTime,
            Self::LocalDate => ValueType::LocalDate,
            Self::LocalTime => ValueType::LocalTime,
            Self::Array(_) => ValueType::Array,
            Self::Table(_) => ValueType::Table,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    Table,
}

#[derive(Debug)]
pub struct TomlData<'a> {
    pub value: Value<'a>,
    pub source: Span<'a>,
}
