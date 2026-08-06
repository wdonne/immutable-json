//! This module requires the `bson` crate feature.

use crate::api::{Number, Value};
use crate::array::Array;
use crate::error::Error;
use crate::object::Object;
use bson::{Bson, Document, Timestamp};
use chrono::{DateTime, SecondsFormat};

impl TryFrom<Bson> for Value {
    type Error = Error;

    fn try_from(value: Bson) -> Result<Self, Self::Error> {
        match from_bson(&value) {
            Some(v) => Ok(v),
            None => Err(Error::ConvertFrom),
        }
    }
}

impl TryFrom<Value> for Bson {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match to_bson(&value) {
            Some(v) => Ok(v),
            None => Err(Error::ConvertTo),
        }
    }
}

fn from_array(array: &[Bson]) -> Array {
    Array::from_iter(array.iter().filter_map(from_bson))
}

/**
Convert a BSON value to an immutable JSON value.
```rust
# use bson::Bson;
# use immutable_json::api::Value;
# use immutable_json::bson::{from_bson, to_bson};
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

    let v: Value = Value::from_str(data)?;
    let to = to_bson(&v);

    assert!(to.as_ref().and_then(|v| v.as_document()).and_then(|d| d.get_datetime("timestamp").ok()).is_some());
    assert_eq!(Some(v.clone()), to.and_then(|f| from_bson(&f)));
#   Ok(())
# }
```
*/
pub fn from_bson(bson: &Bson) -> Option<Value> {
    match bson {
        Bson::Array(v) => Some(Value::Array(from_array(v))),
        Bson::Double(v) => Some(Value::Number(Number::Decimal(*v))),
        Bson::String(v) => Some(Value::String(v.clone())),
        Bson::Document(v) => Some(Value::Object(from_document(v))),
        Bson::Boolean(v) => Some(Value::Bool(*v)),
        Bson::Null => Some(Value::Null),
        Bson::RegularExpression(_) => None,
        Bson::JavaScriptCode(_) => None,
        Bson::JavaScriptCodeWithScope(_) => None,
        Bson::Int32(v) => Some(Value::Number(Number::Integer(i128::from(*v)))),
        Bson::Int64(v) => Some(Value::Number(Number::Integer(i128::from(*v)))),
        Bson::Timestamp(v) => from_timestamp(v).map(Value::String),
        Bson::Binary(_) => None,
        Bson::ObjectId(_) => None,
        Bson::DateTime(v) => Some(Value::String(from_date_time(v))),
        Bson::Symbol(_) => None,
        Bson::Decimal128(_) => None,
        Bson::Undefined => None,
        Bson::MaxKey => None,
        Bson::MinKey => None,
        Bson::DbPointer(_) => None,
    }
}

fn from_document(document: &Document) -> Object {
    Object::from_iter(
        document
            .into_iter()
            .filter_map(|(k, v)| from_bson(v).map(|v| (k.clone(), v))),
    )
}

fn from_date_time(date_time: &bson::DateTime) -> String {
    date_time
        .to_chrono()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn from_timestamp(timestamp: &Timestamp) -> Option<String> {
    DateTime::from_timestamp_secs(i64::from(timestamp.time))
        .map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn to_array(array: &Array) -> bson::Array {
    array.iter().filter_map(|v| to_bson(&v)).collect()
}

/// Convert an immutable JSON value to a BSON value.
pub fn to_bson(value: &Value) -> Option<Bson> {
    match value {
        Value::Array(v) => Some(Bson::Array(to_array(v))),
        Value::Bool(v) => Some(Bson::Boolean(*v)),
        Value::Null => Some(Bson::Null),
        Value::Number(v) => to_number(v),
        Value::Object(v) => Some(Bson::Document(to_object(v))),
        Value::String(v) => match try_date_time(v) {
            Some(t) => Some(Bson::DateTime(t)),
            None => Some(Bson::String(v.clone())),
        },
    }
}

fn to_number(value: &Number) -> Option<Bson> {
    match value {
        Number::Decimal(v) => Some(Bson::Double(*v)),
        Number::Integer(v) => i64::try_from(*v).ok().map(Bson::Int64),
    }
}

fn to_object(object: &Object) -> Document {
    Document::from_iter(
        object
            .iter()
            .filter_map(|(k, v)| to_bson(&v).map(|b| (k, b))),
    )
}

fn try_date_time(s: &str) -> Option<bson::DateTime> {
    bson::DateTime::parse_rfc3339_str(s).ok()
}
