//! This module requires the `jaq` crate feature.

use crate::api::Number::{Decimal, Integer};
use crate::api::{Number, Value};
use crate::array::Array;
use crate::error::Error;
use crate::error::Error::{ConvertFrom, JaqException, JaqNoResult};
use crate::object::Object;
use bytes::Bytes;
use jaq_core::data::JustLut;
use jaq_core::load::{Arena, File, Import, Loader};
use jaq_core::{Ctx, Native, Vars};
use jaq_json::{Map, Num, Rc, Val};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Represents a compiled jq filter. It can be reused for multiple runs.
pub struct Filter {
    expression: jaq_core::compile::Filter<Native<JustLut<Val>>>,
}

impl Filter {
    /// Compile a jq filter into a representation that can be reused for multiple runs.
    pub fn compile(jq: &str) -> Result<Self, Error> {
        let loader = Loader::new(
            jaq_core::defs()
                .chain(jaq_std::defs())
                .chain(jaq_json::defs()),
        )
        .with_read(Self::read_file);
        let arena = Arena::default();
        let modules = loader.load(
            &arena,
            File {
                code: jq,
                path: PathBuf::new(),
            },
        )?;
        let filter = jaq_core::Compiler::default()
            .with_funs(
                jaq_core::funs()
                    .chain(jaq_std::funs())
                    .chain(jaq_json::funs()),
            )
            .compile(modules)?;

        Ok(Self { expression: filter })
    }

    fn read_file(import: Import<&str, PathBuf>) -> Result<File<String, PathBuf>, String> {
        let mut path = import.parent.clone();

        path.push(PathBuf::from(&import.path));
        path = path.with_extension("jq");
        Ok(File {
            code: fs::read_to_string(&path).map_err(|e| e.to_string())?,
            path: path.clone(),
        })
    }

    /// Runs a compiled jq filter.
    /// ```rust
    /// # use immutable_json::api::Value;
    /// # use immutable_json::error::Error;
    /// # use immutable_json::jaq::Filter;
    /// # use std::str::FromStr;
    /// # fn main() -> Result<(), Error>{
    /// let filter = Filter::compile(". + {test: \"test2\"}")?;
    /// let source: Value = Value::from_str("{\"test\": \"test\"}")?;
    /// let target: Value = Value::from_str("{\"test\": \"test2\"}")?;
    ///
    /// assert_eq!(target, filter.run(&source)?);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn run(&self, value: &Value) -> Result<Value, Error> {
        self.expression
            .id
            .run((
                Ctx::<JustLut<Val>>::new(&self.expression.lut, Vars::new([])),
                to_value(value),
            ))
            .next()
            .map(|v| v.map_err(|e| JaqException(format!("{:?}", e))))
            .map(|r| r.and_then(|v| from_value(&v).ok_or(ConvertFrom)))
            .ok_or(JaqNoResult)?
    }
}

/// Convert a jaq_json array to an immutable JSON array.
fn from_array(array: &[Val]) -> Array {
    Array::from_iter(array.iter().filter_map(from_value))
}

fn from_bytes(b: &Bytes) -> Option<Value> {
    String::from_utf8(Vec::<u8>::from(b.clone()))
        .ok()
        .map(Value::String)
}

/// Convert a jaq_json number to an immutable JSON number.
fn from_number(value: &Num) -> Option<Number> {
    match value {
        Num::Int(v) => Some(Number::Integer(*v as i128)),
        Num::BigInt(v) => i128::try_from(v.as_ref()).ok().map(Number::Integer),
        Num::Float(v) => Some(Number::Decimal(*v)),
        Num::Dec(v) => f64::from_str(v.as_ref()).ok().map(Number::Decimal),
    }
}

/// Convert a jaq_json object to an immutable JSON object.
fn from_object(object: &Map<Val, Val>) -> Object {
    Object::from_iter(object.into_iter().filter_map(|e| {
        from_value(e.1).map(|v| {
            (
                from_value(e.0)
                    .and_then(|v| v.as_string())
                    .unwrap_or("".to_string()),
                v,
            )
        })
    }))
}

/// Convert a jaq_json value to an immutable JSON value.
/// ```rust
/// # use jaq_json::Val;
/// # use immutable_json::jaq::{from_value, to_value};
/// # use immutable_json::api::Value;
/// # use immutable_json::error::Error;
/// # use std::str::FromStr;
/// # fn main() -> Result<(), Error>{
/// let data = r#"
///         {
///             "string": "string",
///             "int": 43,
///             "float": 5.8,
///             "boolean": true,
///             "object": {"test": "test"},
///             "array": [
///                 "string",
///                 1,
///                 3.0,
///                 false,
///                 {"test": "test"},
///                 [1]
///             ]
///         }"#;
///
///     let v: Value = Value::from_str(data)?;
///
///     assert_eq!(Some(v.clone()), from_value(&to_value(&v)));
/// #   Ok(())
/// # }
/// ```
pub fn from_value(value: &Val) -> Option<Value> {
    match value {
        Val::Arr(v) => Some(Value::Array(from_array(v))),
        Val::BStr(v) | Val::TStr(v) => from_bytes(v),
        Val::Bool(v) => Some(Value::Bool(*v)),
        Val::Null => Some(Value::Null),
        Val::Num(v) => from_number(v).map(Value::Number),
        Val::Obj(v) => Some(Value::Object(from_object(v))),
    }
}

/// Convert an immutable JSON array to a jaq_json array.
fn to_array(array: &Array) -> Vec<Val> {
    Vec::from_iter(array.iter().map(|v| to_value(&v)))
}

/// Convert an immutable JSON number to a jaq_json number.
fn to_number(value: &Number) -> Num {
    match value {
        Decimal(v) => Num::Float(*v),
        Integer(v) => Num::Int(*v as isize),
    }
}

/// Convert an immutable JSON object to a jaq_json object.
fn to_object(object: &Object) -> Map<Val, Val> {
    Map::from_iter(
        object
            .iter()
            .map(|e| (to_value(&Value::String(e.0)), to_value(&e.1))),
    )
}

/// Convert an immutable JSON value to a jaq_json value.
pub fn to_value(value: &Value) -> Val {
    match value {
        Value::Array(v) => Val::Arr(Rc::new(to_array(v))),
        Value::Bool(v) => Val::Bool(*v),
        Value::Null => Val::Null,
        Value::Number(v) => Val::Num(to_number(v)),
        Value::Object(v) => Val::Obj(Rc::new(to_object(v))),
        Value::String(v) => Val::TStr(Box::new(Bytes::from(v.clone()))),
    }
}
