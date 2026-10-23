//! This module requires the `yaml` crate feature.

use crate::api::Number::{Decimal, Integer};
use crate::{
    api::{Number, Value},
    array::Array,
    object::Object,
};
use indexmap::map::IndexMap;
use rust_yaml::value;

impl From<value::Value> for Value {
    fn from(value: value::Value) -> Self {
        from_yaml(&value)
    }
}

fn from_mapping(object: &IndexMap<value::Value, value::Value>) -> Object {
    Object::from_iter(
        object
            .into_iter()
            .filter_map(|(k, v)| k.as_str().map(|s| (s, v)))
            .map(|(k, v)| (k.to_string(), from_yaml(v))),
    )
}

fn from_sequence(array: &[value::Value]) -> Array {
    Array::from_iter(array.iter().map(from_yaml))
}

/**
Convert a YAML value to an immutable JSON value.
```rust
# use immutable_json::api::Value;
# use immutable_json::yaml::{from_yaml, to_yaml};
# use immutable_json::error::Error;
# use std::str::FromStr;
# fn main() -> Result<(), Error>{
let data = r#"
        {
            "string": "string",
            "int": 43,
            "float": 5.8,
            "boolean": true,
            "object": {"test": "test"},
            "array": [
                "string",
                1,
                3.0,
                false,
                {"test": "test"},
                [1]
            ]
        }"#;

    let v: Value = Value::from_str(data)?;

    assert_eq!(v.clone(), from_yaml(&to_yaml(&v)));
#   Ok(())
# }
```
*/
pub fn from_yaml(value: &value::Value) -> Value {
    match value {
        value::Value::Sequence(v) => Value::Array(from_sequence(v)),
        value::Value::Bool(v) => Value::Bool(*v),
        value::Value::Float(v) => Value::Number(Decimal(*v)),
        value::Value::Int(v) => Value::Number(Integer(*v as i128)),
        value::Value::Mapping(v) => Value::Object(from_mapping(v)),
        value::Value::Null => Value::Null,
        value::Value::String(v) => Value::String(v.clone()),
    }
}

fn to_mapping(object: &Object) -> IndexMap<value::Value, value::Value> {
    IndexMap::from_iter(
        object
            .iter()
            .map(|(k, v)| (value::Value::String(k), to_yaml(&v))),
    )
}

fn to_number(value: &Number) -> value::Value {
    match value {
        Decimal(v) => value::Value::Float(*v),
        Integer(v) => value::Value::Int(*v as i64),
    }
}

fn to_sequence(array: &Array) -> Vec<value::Value> {
    Vec::from_iter(array.iter().map(|v| to_yaml(&v)))
}

/// Convert an immutable JSON value to a YAML value.
pub fn to_yaml(value: &Value) -> value::Value {
    match value {
        Value::Array(v) => value::Value::Sequence(to_sequence(v)),
        Value::Bool(v) => value::Value::Bool(*v),
        Value::Null => value::Value::Null,
        Value::Number(v) => to_number(v),
        Value::Object(v) => value::Value::Mapping(to_mapping(v)),
        Value::String(v) => value::Value::String(v.clone()),
    }
}
