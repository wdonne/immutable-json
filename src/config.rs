//! This module requires the `config` crate feature.

use crate::{
    api::{
        Number,
        Value::{self, Null},
    },
    array::Array,
    error::Error,
    object::Object,
};

impl TryFrom<config::Value> for Value {
    type Error = Error;

    fn try_from(value: config::Value) -> Result<Self, Self::Error> {
        match from_config(&value) {
            Some(v) => Ok(v),
            None => Err(Error::ConvertFrom),
        }
    }
}

fn from_array(array: &[config::Value]) -> Array {
    array
        .iter()
        .filter_map(from_config)
        .fold(Array::new(), |a, v| a.add(&v))
}

/**
Convert a `Config` value to an immutable JSON value.
```rust
# use config::Config;
# use immutable_json::config::from_config;
# use immutable_json::error::Error;
# use immutable_json::serde;
# use std::str::FromStr;
# fn main() -> Result<(), Error>{
let data = r#"
        {
            "string": "string",
            "int": 43,
            "float": 5.8,
            "boolean": true,
            "object": {"test": "test"},
            "timestamp": "2026-07-16T11:26:00.123Z",
            "array": [
                "string",
                1,
                3.0,
                false,
                {"test": "test"},
                [1]
            ]
        }"#;

    let v: serde_json::Value = serde_json::Value::from_str(data)?;
    let c: config::Value = Config::try_from(&v).map_err(|e| Error::ConvertFrom)?.cache;

    assert_eq!(Some(v.clone()), from_config(&c).and_then(|val| serde::to_value(&val)));
#   Ok(())
# }
```
*/
pub fn from_config(config: &config::Value) -> Option<Value> {
    match &config.kind {
        config::ValueKind::Array(v) => Some(Value::Array(from_array(v))),
        config::ValueKind::Boolean(v) => Some(Value::Bool(*v)),
        config::ValueKind::Float(v) => Some(Value::Number(Number::Decimal(*v))),
        config::ValueKind::I64(v) => Some(Value::Number(Number::Integer(i128::from(*v)))),
        config::ValueKind::I128(v) => Some(Value::Number(Number::Integer(*v))),
        config::ValueKind::Nil => Some(Null),
        config::ValueKind::String(v) => Some(Value::String(v.clone())),
        config::ValueKind::Table(v) => Some(Value::Object(from_map(v))),
        config::ValueKind::U64(v) => Some(Value::Number(Number::Integer(i128::from(*v)))),
        config::ValueKind::U128(_) => None,
    }
}

fn from_map(map: &config::Map<String, config::Value>) -> Object {
    map.iter()
        .filter_map(|(k, v)| from_config(v).map(|converted| (k, converted)))
        .fold(Object::new(), |o, (k, v)| o.add(k, &v))
}
