use crate::array::Array;
use crate::object::Object;

/// A JSON number.
#[derive(Clone, Debug)]
pub enum Number {
    Decimal(f64),
    Integer(i128),
}

/// A JSON value.
#[derive(Clone, Debug)]
pub enum Value {
    Array(Array),
    Bool(bool),
    Null,
    Number(Number),
    Object(Object),
    String(String),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Number::Decimal(n) => Some(n) == other.as_decimal().as_ref(),
            Number::Integer(n) => Some(n) == other.as_integer().as_ref(),
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

impl Value {
    pub fn as_array(&self) -> Option<Array> {
        match self {
            Value::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_decimal(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Number::as_decimal(n),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i128> {
        match self {
            Value::Number(n) => Number::as_integer(n),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match self {
            Value::Number(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<Object> {
        match self {
            Value::Object(o) => Some(o.clone()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_decimal(&self) -> bool {
        match self {
            Value::Number(n) => Number::is_decimal(n),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Value::Number(n) => Number::is_integer(n),
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
}
