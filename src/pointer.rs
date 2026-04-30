use crate::api::Number::{Decimal, Integer};
use crate::api::{Number, Value};
use crate::array::Array;
use crate::error::Error;
use crate::object::Object;
use crate::util::{push_back, remove_first, remove_last};
use imbl::{Vector, vector};
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fmt::{Display, Formatter};
use std::iter::zip;
use std::str::FromStr;
use take_until::TakeUntilExt;

/// Represents a JSON pointer.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct JsonPointer {
    path: Vector<String>,
}

impl Default for JsonPointer {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for JsonPointer {
    /// Convert a JSON pointer to a string.
    /// ```rust
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() {
    ///let p = "/a/b~1c/~0d";
    ///
    ///assert_eq!(p, JsonPointer::from_str(p).unwrap().to_string())
    /// # }
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.path.iter().map(|segment| escape(segment)).fold(
                "".to_string(),
                |mut p, segment| {
                    p.push('/');
                    p.push_str(&segment);
                    p
                }
            )
        )
    }
}

impl FromStr for JsonPointer {
    type Err = Error;

    /// Create a JSON pointer from a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("/") {
            Err(Error::JsonPointer(s.to_string()))
        } else {
            Ok(Self {
                path: s
                    .split("/")
                    .filter(|segment| !segment.is_empty())
                    .map(unescape)
                    .fold(Vector::new(), |v, segment| {
                        push_back(&v, segment.to_string())
                    }),
            })
        }
    }
}

impl Ord for JsonPointer {
    /// The comparison takes into account array indexes, which are compared numerically.
    /// ```rust
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() {
    ///assert!(JsonPointer::from_str("/a").unwrap() == JsonPointer::from_str("/a").unwrap());
    ///assert!(JsonPointer::from_str("/a/b").unwrap() < JsonPointer::from_str("/a/c").unwrap());
    ///assert!(JsonPointer::from_str("/a/b/0").unwrap() < JsonPointer::from_str("/a/b/1").unwrap());
    ///assert!(JsonPointer::from_str("/a/b/10").unwrap() > JsonPointer::from_str("/a/b/2").unwrap());
    ///assert!(JsonPointer::from_str("/a/b/-").unwrap() > JsonPointer::from_str("/a/b/2").unwrap());
    ///assert!(JsonPointer::from_str("/a/b/1").unwrap() < JsonPointer::from_str("/a/b/-").unwrap())
    /// # }
    /// ```
    fn cmp(&self, other: &Self) -> Ordering {
        match zip(self.path.iter(), other.path.iter())
            .map(|(s1, s2)| Self::cmp_segment(s1, s2))
            .take_until(|cmp| *cmp != Equal)
            .last()
            .unwrap_or(Equal)
        {
            Less => Less,
            Equal => self.path.len().cmp(&other.path.len()),
            Greater => Greater,
        }
    }
}

impl PartialOrd for JsonPointer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl JsonPointer {
    /// Add a value to a JSON object or array at the location specified by the JSON pointer.
    /// ```rust
    /// # use immutable_json::api::Number::Integer;
    /// # use immutable_json::api::Value;
    /// # use immutable_json::error::Error;
    /// # use immutable_json::object::Object;
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() -> Result<(), Error>{
    ///let data = r#"
    ///    [
    ///        "string",
    ///        {"test": "test"},
    ///        ["string"]
    ///    ]"#;
    ///let array = Value::from_str(data)?.as_array().unwrap();
    ///
    ///assert_eq!(array.insert_string(0, "string2").ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/0")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.add(&Value::Array(array.clone()), &Value::String("string2".to_string()))
    ///        }));
    ///assert_eq!(None,
    ///    JsonPointer::from_str("/4")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.add(&Value::Array(array.clone()), &Value::String("string2".to_string()))
    ///        }));
    ///assert_eq!(array.insert_integer(3, 0).ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/-")
    ///        .ok()
    ///        .and_then(|p| p.add(&Value::Array(array.clone()), &Value::Number(Integer(0)))));
    ///assert_eq!(array.set_object(
    ///        1,
    ///        &Object::new().add_string("test", "test").add_string("test2", "test2")
    ///    ).ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/1/test2")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.add(&Value::Array(array.clone()), &Value::String("test2".to_string()))
    ///        }));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn add(&self, target: &Value, value: &Value) -> Option<Value> {
        self.modify(target, |v, key| match v {
            Value::Array(a) => usize::from_str(key)
                .ok()
                .and_then(|i| a.insert(i, value).ok().map(Value::Array)),
            Value::Object(o) => Some(Value::Object(o.add(key, value))),
            _ => None,
        })
    }

    /// Add a JSON array to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_array(pointer: &str, target: &Value, value: &Array) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Array(value.clone()))
    }

    /// Add a Boolean value to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_bool(pointer: &str, target: &Value, value: bool) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Bool(value))
    }

    /// Add a decimal value to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_decimal(pointer: &str, target: &Value, value: f64) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Number(Decimal(value)))
    }

    /// Add an integer value to a JSON object or array at the location specified by the JSON
    /// pointer.
    pub fn add_integer(pointer: &str, target: &Value, value: i128) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Number(Integer(value)))
    }

    /// Add a number to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_number(pointer: &str, target: &Value, value: Number) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Number(value))
    }

    /// Add a JSON object to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_object(pointer: &str, target: &Value, value: &Object) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::Object(value.clone()))
    }

    fn add_pointer(pointer: &str, target: &Value, value: &Value) -> Option<Value> {
        Self::from_str(pointer)
            .ok()
            .and_then(|p| p.add(target, value))
    }

    /// Add a string value to a JSON object or array at the location specified by the JSON pointer.
    pub fn add_string(pointer: &str, target: &Value, value: &str) -> Option<Value> {
        Self::add_pointer(pointer, target, &Value::String(value.to_string()))
    }

    /// If the pointer refers to a value in an array, the index within the array is returned.
    pub fn array_index(&self) -> Option<usize> {
        self.path.last().and_then(|last| usize::from_str(last).ok())
    }

    fn as_integer(&self) -> Option<usize> {
        usize::from_str(&self.path[0]).ok()
    }

    /// Returns a JSON pointer with an extra path segment.
    pub fn child(&self, segment: &str) -> Self {
        Self {
            path: push_back(&self.path, segment.to_string()),
        }
    }

    fn cmp_segment(s1: &str, s2: &str) -> Ordering {
        usize::from_str(s1)
            .ok()
            .and_then(|n1| usize::from_str(s2).ok().map(|n2| n1.cmp(&n2)))
            .or_else(|| {
                if s2 == "-" {
                    usize::from_str(s1).ok().map(|_| Less)
                } else {
                    None
                }
            })
            .or_else(|| {
                if s1 == "-" {
                    usize::from_str(s2).ok().map(|_| Greater)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| s1.cmp(s2))
    }

    /// Get a value from a JSON object or array through a JSON pointer.
    /// ```rust
    /// # use immutable_json::api::Value;
    /// # use immutable_json::error::Error;
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() -> Result<(), Error>{
    ///let data = r#"
    ///    {
    ///        "string": "string",
    ///        "int": 43,
    ///        "float": 5.8,
    ///        "boolean": true,
    ///        "object": {"test": "test"},
    ///        "array": [
    ///            "string",
    ///            1,
    ///            3.0,
    ///            false,
    ///            {"test": "test"},
    ///            [1]
    ///        ],
    ///        "es/cape": true
    ///    }"#;
    ///let object = Value::from_str(data)?;
    ///
    ///assert_eq!(Some(Value::String("string".to_string())),
    ///    JsonPointer::from_str("/string").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(Some(Value::String("test".to_string())),
    ///    JsonPointer::from_str("/object/test").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(None, JsonPointer::from_str("/object/test2").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(None, JsonPointer::from_str("/object2/test").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(Some(Value::String("test".to_string())),
    ///    JsonPointer::from_str("/array/4/test").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(None, JsonPointer::from_str("/array/3/test").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(None, JsonPointer::from_str("/array2/4/test").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(Some(Value::Bool(true)),
    ///    JsonPointer::from_str("/es~1cape").ok().and_then(|p| p.get(&object)));
    ///assert_eq!(Some(object.clone()), JsonPointer::from_str("/").ok().and_then(|p| p.get(&object)));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get(&self, target: &Value) -> Option<Value> {
        match target {
            Value::Array(a) => self.get_from_array(a),
            Value::Object(o) => self.get_from_object(o),
            _ => None,
        }
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is an array.
    /// Otherwise, `None` is returned.
    pub fn get_array(pointer: &str, target: &Value) -> Option<Array> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_array())
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is a Boolean.
    /// Otherwise, `None` is returned.
    pub fn get_bool(pointer: &str, target: &Value) -> Option<bool> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_bool())
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is a decimal.
    /// Otherwise, `None` is returned.
    pub fn get_decimal(pointer: &str, target: &Value) -> Option<f64> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_decimal())
    }

    fn get_from_array(&self, target: &Array) -> Option<Value> {
        if self.path.is_empty() {
            Some(Value::Array(target.clone()))
        } else {
            self.as_integer()
                .and_then(|i| target.get(i).ok())
                .and_then(|v| self.next_level_or(&v))
        }
    }

    fn get_from_object(&self, target: &Object) -> Option<Value> {
        if self.path.is_empty() {
            Some(Value::Object(target.clone()))
        } else {
            target
                .get(&self.path[0])
                .and_then(|v| self.next_level_or(v))
        }
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is an integer.
    /// Otherwise, `None` is returned.
    pub fn get_integer(pointer: &str, target: &Value) -> Option<i128> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_integer())
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is a number.
    /// Otherwise, `None` is returned.
    pub fn get_number(pointer: &str, target: &Value) -> Option<Number> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_number())
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is an object.
    /// Otherwise, `None` is returned.
    pub fn get_object(pointer: &str, target: &Value) -> Option<Object> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_object())
    }

    fn get_pointer(pointer: &str, target: &Value) -> Option<Value> {
        Self::from_str(pointer).ok().and_then(|p| p.get(target))
    }

    /// Get a value from a JSON object or array through a JSON pointer if it is a string.
    /// Otherwise, `None` is returned.
    pub fn get_string(pointer: &str, target: &Value) -> Option<String> {
        Self::get_pointer(pointer, target).and_then(|v| v.as_string())
    }

    fn modify<F>(&self, target: &Value, update: F) -> Option<Value>
    where
        F: Fn(&Value, &str) -> Option<Value>,
    {
        match target {
            Value::Array(a) => self.modify_array(a, update).map(Value::Array),
            Value::Object(o) => self.modify_object(o, update).map(Value::Object),
            _ => None,
        }
    }

    fn modify_array<F>(&self, target: &Array, update: F) -> Option<Array>
    where
        F: Fn(&Value, &str) -> Option<Value>,
    {
        if self.path.is_empty() {
            None
        } else if self.path.len() == 1 {
            self.update_index(target)
                .and_then(|i| update(&Value::Array(target.clone()), &i.to_string()))
                .and_then(|v| v.as_array())
        } else {
            self.next()
                .and_then(|p| {
                    self.as_integer()
                        .and_then(|i| target.get(i).ok())
                        .and_then(|v| p.modify(&v, update))
                })
                .and_then(|v| self.as_integer().and_then(|i| target.set(i, &v).ok()))
        }
    }

    fn modify_object<F>(&self, target: &Object, update: F) -> Option<Object>
    where
        F: Fn(&Value, &str) -> Option<Value>,
    {
        if self.path.is_empty() {
            None
        } else if self.path.len() == 1 {
            update(&Value::Object(target.clone()), &self.path[0]).and_then(|v| v.as_object())
        } else {
            self.next()
                .and_then(|p| target.get(&self.path[0]).and_then(|v| p.modify(v, update)))
                .map(|v| target.add(&self.path[0], &v))
        }
    }

    /// Creates a JSON pointer that refers to the root.
    pub fn new() -> Self {
        Self { path: vector!() }
    }

    fn next(&self) -> Option<JsonPointer> {
        if self.path.len() <= 1 {
            None
        } else {
            Some(Self {
                path: remove_first(&self.path),
            })
        }
    }

    fn next_level_or(&self, target: &Value) -> Option<Value> {
        match self.next() {
            Some(n) => match target {
                Value::Array(a) => n.get_from_array(a),
                Value::Object(o) => n.get_from_object(o),
                _ => None,
            },
            None => Some(target.clone()),
        }
    }

    /// Returns a JSON pointer that refers to the parent.
    pub fn parent(&self) -> Self {
        Self {
            path: remove_last(&self.path),
        }
    }

    /// Remove a value in a JSON object or array at the location specified by the JSON pointer.
    /// ```rust
    /// # use immutable_json::api::Number;
    /// # use immutable_json::api::Value;
    /// # use immutable_json::error::Error;
    /// # use immutable_json::object::Object;
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() -> Result<(), Error>{
    ///let data = r#"
    ///    [
    ///        "string",
    ///        {"test": "test", "test2": "test2"},
    ///        ["string"]
    ///    ]"#;
    ///let array = Value::from_str(data)?.as_array().unwrap();
    ///
    ///assert_eq!(array.remove(0).ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/0")
    ///        .ok()
    ///        .and_then(|p| p.remove(&Value::Array(array.clone()))));
    ///assert_eq!(None,
    ///    JsonPointer::from_str("/4")
    ///        .ok()
    ///        .and_then(|p| p.remove(&Value::Array(array.clone()))));
    ///assert_eq!(array.set_object(1, &Object::new().add_string("test", "test"))
    ///        .ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/1/test2")
    ///        .ok()
    ///        .and_then(|p| p.remove(&Value::Array(array.clone()))));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn remove(&self, target: &Value) -> Option<Value> {
        self.modify(target, |v, key| match v {
            Value::Array(a) => usize::from_str(key)
                .ok()
                .and_then(|i| a.remove(i).ok().map(Value::Array)),
            Value::Object(o) => Some(Value::Object(o.remove(key))),
            _ => None,
        })
    }

    /// Set a value in a JSON object or array at the location specified by the JSON pointer.
    /// ```rust
    /// # use immutable_json::api::Number;
    /// # use immutable_json::api::Value;
    /// # use immutable_json::error::Error;
    /// # use immutable_json::object::Object;
    /// # use immutable_json::pointer::JsonPointer;
    /// # use std::str::FromStr;
    /// # fn main() -> Result<(), Error>{
    ///let data = r#"
    ///    [
    ///        "string",
    ///        {"test": "test"},
    ///        ["string"]
    ///    ]"#;
    ///let array = Value::from_str(data)?.as_array().unwrap();
    ///
    ///assert_eq!(array.set_string(0, "string2").ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/0")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.set(&Value::Array(array.clone()), &Value::String("string2".to_string()))
    ///        }));
    ///assert_eq!(None,
    ///    JsonPointer::from_str("/4")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.set(&Value::Array(array.clone()), &Value::String("string2".to_string()))
    ///        }));
    ///assert_eq!(array.set_object(1, &Object::new().add_string("test", "test2"))
    ///        .ok().map(|a| Value::Array(a)),
    ///    JsonPointer::from_str("/1/test")
    ///        .ok()
    ///        .and_then(|p| {
    ///            p.set(&Value::Array(array.clone()), &Value::String("test2".to_string()))
    ///        }));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn set(&self, target: &Value, value: &Value) -> Option<Value> {
        self.modify(target, |v, key| match v {
            Value::Array(a) => usize::from_str(key)
                .ok()
                .and_then(|i| a.set(i, value).ok().map(Value::Array)),
            Value::Object(o) => Some(Value::Object(o.add(key, value))),
            _ => None,
        })
    }

    /// Set an array in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_array(pointer: &str, target: &Value, value: &Array) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Array(value.clone()))
    }

    /// Set a Boolean value in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_bool(pointer: &str, target: &Value, value: bool) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Bool(value))
    }

    /// Set a decimal value in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_decimal(pointer: &str, target: &Value, value: f64) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Number(Decimal(value)))
    }

    /// Set an integer value in a JSON object or array at the location specified by the JSON
    /// pointer.
    pub fn set_integer(pointer: &str, target: &Value, value: i128) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Number(Integer(value)))
    }

    /// Set a number in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_number(pointer: &str, target: &Value, value: Number) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Number(value))
    }

    /// Set a JSON object in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_object(pointer: &str, target: &Value, value: &Object) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::Object(value.clone()))
    }

    fn set_pointer(pointer: &str, target: &Value, value: &Value) -> Option<Value> {
        Self::from_str(pointer)
            .ok()
            .and_then(|p| p.set(target, value))
    }

    /// Set a string value in a JSON object or array at the location specified by the JSON pointer.
    pub fn set_string(pointer: &str, target: &Value, value: &str) -> Option<Value> {
        Self::set_pointer(pointer, target, &Value::String(value.to_string()))
    }

    fn update_index(&self, array: &Array) -> Option<usize> {
        if self.path[0] == "-" {
            Some(array.len())
        } else {
            self.as_integer().filter(|i| *i <= array.len())
        }
    }
}

fn escape(s: &str) -> String {
    s.replace("~", "~0").replace("/", "~1")
}

fn unescape(s: &str) -> String {
    s.replace("~1", "/").replace("~0", "~")
}
