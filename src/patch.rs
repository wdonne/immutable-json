use crate::api::Value;
use crate::array::Array;
use crate::error::JsonPatchError;
use crate::object::Object;
use crate::serde::{from_value, to_value};
use json_patch::jsonptr::PointerBuf;
use json_patch::{
    AddOperation, CopyOperation, MoveOperation, Patch, PatchOperation, RemoveOperation,
    ReplaceOperation, TestOperation, patch,
};
use serde_json::Value::Null;

fn add_operation(object: &Object) -> Option<PatchOperation> {
    match (get_path(object, "path"), get_value(object)) {
        (Some(p), Some(v)) => Some(PatchOperation::Add(AddOperation { path: p, value: v })),
        (_, _) => None,
    }
}

fn apply(source: &Value, json_patch: &Array) -> Result<Value, JsonPatchError> {
    let mut value = to_value(source).unwrap_or(Null);
    let p = array_to_patch(json_patch);

    match patch(&mut value, &p) {
        Ok(_) => Ok(from_value(&value).unwrap_or(Value::Null)),
        Err(e) => Err(JsonPatchError {
            source: Box::from(source.clone()),
            patch: Box::from(json_patch.clone()),
            error: e.to_string(),
        }),
    }
}

/// Constructs a new JSON array by applying the given patch to the source.
pub fn apply_array(source: &Array, json_patch: &Array) -> Result<Array, JsonPatchError> {
    apply(&Value::Array(source.clone()), json_patch).and_then(|v| {
        v.as_array().ok_or_else(|| JsonPatchError {
            source: Box::from(Value::Array(source.clone())),
            patch: Box::from(json_patch.clone()),
            error: "the result is not an array".to_string(),
        })
    })
}

/// Constructs a new JSON object by applying the given patch to the source.
pub fn apply_object(source: &Object, json_patch: &Array) -> Result<Object, JsonPatchError> {
    apply(&Value::Object(source.clone()), json_patch).and_then(|v| {
        v.as_object().ok_or_else(|| JsonPatchError {
            source: Box::from(Value::Object(source.clone())),
            patch: Box::from(json_patch.clone()),
            error: "the result is not an object".to_string(),
        })
    })
}

fn array_to_patch(array: &Array) -> Vec<PatchOperation> {
    array
        .iter()
        .flat_map(|v| v.as_object().into_iter())
        .flat_map(|o| object_to_path_operation(&o).into_iter())
        .collect()
}

fn copy_operation(object: &Object) -> Option<PatchOperation> {
    match (get_path(object, "from"), get_path(object, "path")) {
        (Some(f), Some(p)) => Some(PatchOperation::Copy(CopyOperation { from: f, path: p })),
        (_, _) => None,
    }
}

fn diff(source: &Value, target: &Value) -> Array {
    patch_to_array(&json_patch::diff(
        &to_value(source).unwrap_or(Null),
        &to_value(target).unwrap_or(Null),
    ))
}

/**
Generates a JSON patch that, when applied to the source, yields the target.
```rust
# use immutable_json::api::Value;
# use immutable_json::error::Error;
# use immutable_json::patch::apply_array;
# use immutable_json::patch::diff_array;
# use std::str::FromStr;
# fn main() -> Result<(), Error>{
let source = r#"
   [
       "string",
       1,
       3.0,
       false,
       {"test": "test", "obj": {"test2": "test"}},
       [1]
   ]"#;
let target = r#"
   [
       "string",
       3.0,
       3.0,
       false,
       {"test": "test2", "obj": {"test2": "test2"}},
       1,
       [1]
   ]"#;

let s = Value::from_str(source)?.as_array().unwrap();
let t = Value::from_str(target)?.as_array().unwrap();

assert_eq!(t, apply_array(&s, &diff_array(&s, &t))?);
#   Ok(())
# }
```
*/
pub fn diff_array(source: &Array, target: &Array) -> Array {
    diff(&Value::Array(source.clone()), &Value::Array(target.clone()))
}

/**
Generates a JSON patch that, when applied to the source, yields the target.
```rust
# use immutable_json::api::Value;
# use immutable_json::error::Error;
# use immutable_json::patch::apply_object;
# use immutable_json::patch::diff_object;
# use std::str::FromStr;
# fn main() -> Result<(), Error>{
let source = r#"
   {
       "string": "string",
       "int": 43,
       "float": 5.8,
       "boolean": true,
       "object": {"test": "test", "obj": {"test2": "test"}},
       "array": [
           "string",
           1,
           3.0,
           false,
           {"test": "test", "obj": {"test2": "test"}},
           [1]
       ]
   }"#;
let target = r#"
   {
       "string": "string",
       "int": 44,
       "boolean": false,
       "object": {"test": "test"},
       "array": [
           "string",
           3.0,
           3.0,
           false,
           {"test": "test2", "obj": {"test2": "test2"}},
           1,
           [1]
       ],
       "float": 5.8
   }"#;

let s = Value::from_str(source)?.as_object().unwrap();
let t = Value::from_str(target)?.as_object().unwrap();

assert_eq!(t, apply_object(&s, &diff_object(&s, &t))?);
#   Ok(())
# }
```
*/
pub fn diff_object(source: &Object, target: &Object) -> Array {
    diff(
        &Value::Object(source.clone()),
        &Value::Object(target.clone()),
    )
}

fn get_path(object: &Object, field: &str) -> Option<PointerBuf> {
    object.get_string(field).and_then(|v| v.parse().ok())
}

fn get_value(object: &Object) -> Option<serde_json::Value> {
    object.get("value").and_then(to_value)
}

fn move_operation(object: &Object) -> Option<PatchOperation> {
    match (get_path(object, "from"), get_path(object, "path")) {
        (Some(f), Some(p)) => Some(PatchOperation::Move(MoveOperation { from: f, path: p })),
        (_, _) => None,
    }
}

fn object_to_path_operation(object: &Object) -> Option<PatchOperation> {
    object.get_string("op").and_then(|s| match s.as_str() {
        "add" => add_operation(object),
        "copy" => copy_operation(object),
        "move" => move_operation(object),
        "remove" => remove_operation(object),
        "replace" => replace_operation(object),
        "test" => test_operation(object),
        _ => None,
    })
}

fn patch_operation_to_object(op: &PatchOperation) -> Object {
    match op {
        PatchOperation::Add(a) => Object::new()
            .add_string("op", "add")
            .add_string("path", a.path.as_str())
            .add("value", &from_value(&a.value).unwrap_or(Value::Null)),
        PatchOperation::Copy(c) => Object::new()
            .add_string("op", "copy")
            .add_string("from", c.from.as_str())
            .add_string("path", c.path.as_str()),
        PatchOperation::Move(m) => Object::new()
            .add_string("op", "move")
            .add_string("from", m.from.as_str())
            .add_string("path", m.path.as_str()),
        PatchOperation::Remove(r) => Object::new()
            .add_string("op", "remove")
            .add_string("path", r.path.as_str()),
        PatchOperation::Replace(r) => Object::new()
            .add_string("op", "replace")
            .add_string("path", r.path.as_str())
            .add("value", &from_value(&r.value).unwrap_or(Value::Null)),
        PatchOperation::Test(t) => Object::new()
            .add_string("op", "test")
            .add_string("path", t.path.as_str())
            .add("value", &from_value(&t.value).unwrap_or(Value::Null)),
    }
}

fn patch_to_array(patch: &Patch) -> Array {
    patch
        .iter()
        .map(patch_operation_to_object)
        .fold(Array::new(), |a, o| a.add_object(&o))
}

fn remove_operation(object: &Object) -> Option<PatchOperation> {
    get_path(object, "path").map(|p| PatchOperation::Remove(RemoveOperation { path: p }))
}

fn replace_operation(object: &Object) -> Option<PatchOperation> {
    match (get_path(object, "path"), get_value(object)) {
        (Some(p), Some(v)) => Some(PatchOperation::Replace(ReplaceOperation {
            path: p,
            value: v,
        })),
        (_, _) => None,
    }
}

fn test_operation(object: &Object) -> Option<PatchOperation> {
    match (get_path(object, "path"), get_value(object)) {
        (Some(p), Some(v)) => Some(PatchOperation::Test(TestOperation { path: p, value: v })),
        (_, _) => None,
    }
}
