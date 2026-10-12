use crate::{api::Value, array::Array, object::Object};

/**
Replaces JSON values recursively. If the replacement function produces an object or array,
then they will be recursed into as well.
```rust
# use immutable_json::api::Value;
# use immutable_json::error::Error;
# use immutable_json::replace::replace;
# use std::str::FromStr;
# fn main() -> Result<(), Error>{
let data_in = r#"
   {
       "string": "test",
       "int": 43,
       "float": 5.8,
       "boolean": true,
       "object": {"test": "test"},
       "array": [
           "test",
           1,
           "notest",
           false,
           {"test": "test"},
           [1]
       ]
   }"#;
   let data_out = r#"
      {
          "string": "test2",
          "int": 43,
          "float": 5.8,
          "boolean": true,
          "object": {"test": "test2"},
          "array": [
              "test2",
              1,
              "notest",
              false,
              {"test": "test2"},
              [1]
          ]
      }"#;

let v_in = Value::from_str(data_in)?;
let v_out = Value::from_str(data_out)?;

assert_eq!(v_out,
    replace(&v_in,
        &|v: &Value| {
            v.as_string()
                .filter(|s| s == "test")
                .map(|s| Value::String("test2".to_string()))
        }));
#   Ok(())
# }
```
*/
pub fn replace<F>(value: &Value, f: &F) -> Value
where
    F: Fn(&Value) -> Option<Value>,
{
    match value {
        Value::Array(a) => Value::Array(replace_array(a, f)),
        Value::Object(o) => Value::Object(replace_object(o, f)),
        _ => value.clone(),
    }
}

pub fn replace_array<F>(array: &Array, f: &F) -> Array
where
    F: Fn(&Value) -> Option<Value>,
{
    replace_array_changed(array, f).unwrap_or(array.clone())
}

fn replace_array_changed<F>(array: &Array, f: &F) -> Option<Array>
where
    F: Fn(&Value) -> Option<Value>,
{
    let mut updated = false;
    let result = array.iter().enumerate().fold(array.clone(), |a, (i, v)| {
        replace_value(&v, f).map_or(a.clone(), |changed| {
            updated = true;
            a.set(i, &changed).unwrap()
        })
    });

    if updated { Some(result) } else { None }
}

fn replace_changed<F>(value: &Value, f: &F) -> Option<Value>
where
    F: Fn(&Value) -> Option<Value>,
{
    match value {
        Value::Array(a) => replace_array_changed(a, f).map(Value::Array),
        Value::Object(o) => replace_object_changed(o, f).map(Value::Object),
        _ => Some(value.clone()),
    }
}

pub fn replace_object<F>(object: &Object, f: &F) -> Object
where
    F: Fn(&Value) -> Option<Value>,
{
    replace_object_changed(object, f).unwrap_or(object.clone())
}

fn replace_object_changed<F>(object: &Object, f: &F) -> Option<Object>
where
    F: Fn(&Value) -> Option<Value>,
{
    let mut updated = false;
    let result = object.iter().fold(object.clone(), |o, (k, v)| {
        replace_value(&v, f).map_or(o.clone(), |changed| {
            updated = true;
            o.set(&k, &changed)
        })
    });

    if updated { Some(result) } else { None }
}

fn replace_value<F>(value: &Value, f: &F) -> Option<Value>
where
    F: Fn(&Value) -> Option<Value>,
{
    f(value).map_or_else(
        || replace_changed(value, f),
        |new_v| replace_changed(&new_v, f),
    )
}
