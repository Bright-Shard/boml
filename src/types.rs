use std::{borrow::Borrow, fmt::Display, hash::Hash, ops::Deref};

use crate::crate_prelude::*;

#[derive(Debug, PartialEq)]
pub enum TomlValue<'a> {
    String(TomlString<'a>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array(Vec<Self>),
    Table(Table<'a>),
}
impl<'a> TomlValue<'a> {
    pub fn ty(&self) -> ValueType {
        match *self {
            Self::String(_) => ValueType::String,
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

    pub fn string(&self) -> Option<&TomlString<'a>> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }
    pub fn integer(&self) -> Option<i64> {
        match self {
            Self::Integer(num) => Some(*num),
            _ => None,
        }
    }
    pub fn float(&self) -> Option<f64> {
        match self {
            Self::Float(num) => Some(*num),
            _ => None,
        }
    }
    pub fn boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(bool_) => Some(*bool_),
            _ => None,
        }
    }
    pub fn array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }
    pub fn table(&self) -> Option<&Table<'a>> {
        match self {
            Self::Table(table) => Some(table),
            _ => None,
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
pub enum TomlString<'a> {
    Formatted(Span<'a>, String),
    Raw(Span<'a>),
}
impl<'a> TomlString<'a> {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.borrow()
    }

    pub fn span(&self) -> &Span<'a> {
        match self {
            Self::Formatted(span, _) => span,
            Self::Raw(span) => span,
        }
    }
}
impl<'a> Borrow<str> for TomlString<'a> {
    fn borrow(&self) -> &str {
        match self {
            TomlString::Formatted(_, ref string) => string.as_str(),
            TomlString::Raw(span) => span.as_str(),
        }
    }
}
impl<'a> Deref for TomlString<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}
impl<'a> PartialEq for TomlString<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl<'a> Eq for TomlString<'a> {}
impl<'a> Hash for TomlString<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}
impl<'a> Display for TomlString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
impl<'a> From<Span<'a>> for TomlString<'a> {
    fn from(value: Span<'a>) -> Self {
        Self::Raw(value)
    }
}

pub struct Key<'a> {
    pub text: TomlString<'a>,
    pub child: Option<Box<Key<'a>>>,
}
