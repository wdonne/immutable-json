use crate::array::Array;
use crate::error::Error;
use crate::object::Object;
use crate::serde::{from_value, to_value};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A JSON number.
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Decimal(f64),
    Integer(i128),
}

/// A JSON value.
#[derive(Clone, Debug, Eq)]
pub enum Value {
    Array(Array),
    Bool(bool),
    Null,
    Number(Number),
    Object(Object),
    String(String),
}

impl Eq for Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Number::Decimal(v) => v.to_string().hash(state),
            Number::Integer(v) => state.write_i128(*v),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::Decimal(s), Number::Decimal(o)) => s == o,
            (Number::Integer(s), Number::Integer(o)) => s == o,
            _ => false,
        }
    }
}

impl Number {
    pub fn as_decimal(&self) -> Option<f64> {
        match self {
            Number::Decimal(v) => Some(*v),
            Number::Integer(_) => None,
        }
    }

    pub fn as_integer(&self) -> Option<i128> {
        match self {
            Number::Decimal(_) => None,
            Number::Integer(v) => Some(*v),
        }
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, Number::Decimal(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Number::Integer(_))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Array(v) => Some(v) == other.as_array().as_ref(),
            Value::Bool(v) => Some(v) == other.as_bool().as_ref(),
            Value::Null => other.is_null(),
            Value::Number(v) => Some(v) == other.as_number().as_ref(),
            Value::Object(v) => Some(v) == other.as_object().as_ref(),
            Value::String(v) => Some(v) == other.as_string().as_ref(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            to_value(self).map_or("".to_string(), |v| v.to_string())
        )
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: serde_json::Value = serde_json::from_str(s)?;

        match from_value(&v) {
            Some(c) => Ok(c),
            None => Err(Error::ConvertFrom),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Array(v) => v.hash(state),
            Value::Bool(v) => v.hash(state),
            Value::Null => state.write_u8(0),
            Value::Number(v) => v.hash(state),
            Value::Object(v) => v.hash(state),
            Value::String(v) => v.hash(state),
        }
    }
}

impl TryFrom<serde_json::Value> for Value {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match from_value(&value) {
            Some(v) => Ok(v),
            None => Err(Error::ConvertFrom),
        }
    }
}

impl TryFrom<Value> for serde_json::Value {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match to_value(&value) {
            Some(v) => Ok(v),
            None => Err(Error::ConvertTo),
        }
    }
}

impl Value {
    pub fn as_array(&self) -> Option<Array> {
        match self {
            Self::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_decimal(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Number::as_decimal(n),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i128> {
        match self {
            Self::Number(n) => Number::as_integer(n),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<Object> {
        match self {
            Self::Object(o) => Some(o.clone()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_decimal(&self) -> bool {
        match self {
            Self::Number(n) => Number::is_decimal(n),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Self::Number(n) => Number::is_integer(n),
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_scalar(&self) -> bool {
        self.is_null() || self.is_bool() || self.is_number() || self.is_string()
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_structure(&self) -> bool {
        self.is_array() || self.is_object()
    }
}
