use crate::api::Number::{Decimal, Integer};
use crate::api::{Number, Value};
use crate::array::Array;
use crate::object::Object;

/// Convert a serde_json array to an immutable JSON array.
fn from_array(array: &[serde_json::Value]) -> Array {
    Array::from_iter(array.iter().filter_map(from_value))
}

/// Convert a serde_json number to an immutable JSON number.
fn from_number(value: &serde_json::Number) -> Option<Number> {
    value
        .as_i128()
        .map_or_else(|| value.as_f64().map(Decimal), |i| Some(Integer(i)))
}

/// Convert a serde_json object to an immutable JSON object.
fn from_object(object: &serde_json::Map<String, serde_json::Value>) -> Object {
    Object::from_iter(
        object
            .into_iter()
            .filter_map(|e| from_value(e.1).map(|v| (e.0.clone(), v))),
    )
}

/**
Convert a serde_json value to an immutable JSON value.
```rust
# use serde_json::Value;
# use immutable_json::serde::{from_value, to_value};
# use serde_json::Error;
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

    let v: Value = serde_json::from_str(data)?;

    assert_eq!(Some(v.clone()), from_value(&v).and_then(|f| to_value(&f)));
#   Ok(())
# }
```
*/
pub fn from_value(value: &serde_json::Value) -> Option<Value> {
    match value {
        serde_json::Value::Array(v) => Some(Value::Array(from_array(v))),
        serde_json::Value::Bool(v) => Some(Value::Bool(*v)),
        serde_json::Value::Null => Some(Value::Null),
        serde_json::Value::Number(v) => from_number(v).map(Value::Number),
        serde_json::Value::Object(v) => Some(Value::Object(from_object(v))),
        serde_json::Value::String(v) => Some(Value::String(v.clone())),
    }
}

/// Convert an immutable JSON array to a serde_json array.
fn to_array(array: &Array) -> Vec<serde_json::Value> {
    Vec::from_iter(array.iter().filter_map(|v| to_value(&v)))
}

/// Convert an immutable JSON number to a serde_json number.
fn to_number(value: &Number) -> Option<serde_json::Number> {
    match value {
        Decimal(v) => serde_json::Number::from_f64(*v),
        Integer(v) => serde_json::Number::from_i128(*v),
    }
}

/// Convert an immutable JSON object to a serde_json object.
fn to_object(object: &Object) -> serde_json::Map<String, serde_json::Value> {
    serde_json::Map::from_iter(
        object
            .iter()
            .filter_map(|e| to_value(&e.1).map(|v| (e.0, v))),
    )
}

/// Convert an immutable JSON value to a serde_json value.
pub fn to_value(value: &Value) -> Option<serde_json::Value> {
    match value {
        Value::Array(v) => Some(serde_json::Value::Array(to_array(v))),
        Value::Bool(v) => Some(serde_json::Value::Bool(*v)),
        Value::Null => Some(serde_json::Value::Null),
        Value::Number(v) => to_number(v).map(serde_json::Value::Number),
        Value::Object(v) => Some(serde_json::Value::Object(to_object(v))),
        Value::String(v) => Some(serde_json::Value::String(v.clone())),
    }
}
