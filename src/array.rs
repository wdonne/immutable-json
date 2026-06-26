use crate::api::Number::{Decimal, Integer};
use crate::api::{Number, Value};
use crate::error::JsonIndexError;
use crate::object::Object;
use crate::pointer::JsonPointer;
use imbl::Vector;
use imbl::shared_ptr::DefaultSharedPtr;
use imbl::vector::Iter;
use imbl_util::vector;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

/// Represents a JSON array.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Array {
    vec: Vector<Value>,
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Array {
    /**
    Converts a JSON array to a string.
    ```rust
    # use immutable_json::api::Value;
    # use immutable_json::error::Error;
    # use std::str::FromStr;
    # fn main() -> Result<(), Error>{
    let data = r#"
       [
           "string",
           1,
           3.0,
           false,
           {"test": "test"},
           [1]
       ]"#;

    let v: serde_json::Value = serde_json::from_str(data)?;

    assert_eq!(Some(v), serde_json::from_str(&Value::from_str(data)?.to_string()).ok());
    #   Ok(())
    # }
    ```
    */
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&Value::Array(self.clone()), f)
    }
}

impl FromIterator<Value> for Array {
    /// Creates a JSON array from a stream of JSON values.
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |a, v| a.add(&v))
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = Value;
    type IntoIter = ArrayIter<'a>;

    /// Iterates over the values in a JSON array,
    fn into_iter(self) -> ArrayIter<'a> {
        self.iter()
    }
}

impl Array {
    /**
    Adds a JSON value to an array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Value;
    # fn main() {
    assert_eq!(Some(true), Array::new().add(&Value::Bool(true)).get_bool(0).ok().flatten());
    # }
    ```
    */
    pub fn add(&self, value: &Value) -> Self {
        Self {
            vec: vector::push_back(&self.vec, value.clone()),
        }
    }

    /**
    Adds an array to an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(
        Some(0),
        Array::new()
            .add_array(&Array::new().add_integer(0))
            .get_array(0).ok().flatten().and_then(|a| a.get_integer(0).ok().flatten()));
    # }
    ```
    */
    pub fn add_array(&self, value: &Array) -> Self {
        self.add(&Value::Array(value.clone()))
    }

    pub fn add_array_p(&self, pointer: &str, value: &Array) -> Option<Self> {
        self.add_p(pointer, &Value::Array(value.clone()))
    }

    /**
    Adds a bool to an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(Some(true), Array::new().add_bool(true).get_bool(0).ok().flatten());
    # }
    ```
    */
    pub fn add_bool(&self, value: bool) -> Self {
        self.add(&Value::Bool(value))
    }

    pub fn add_bool_p(&self, pointer: &str, value: bool) -> Option<Self> {
        self.add_p(pointer, &Value::Bool(value))
    }

    /**
    Adds a decimal to an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(Some(2.0), Array::new().add_decimal(2.0).get_decimal(0).ok().flatten());
    # }
    ```
    */
    pub fn add_decimal(&self, value: f64) -> Self {
        self.add(&Value::Number(Decimal(value)))
    }

    pub fn add_decimal_p(&self, pointer: &str, value: f64) -> Option<Self> {
        self.add_p(pointer, &Value::Number(Number::Decimal(value)))
    }

    /**
    Adds an integer to an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(Some(0), Array::new().add_integer(0).get_integer(0).ok().flatten());
    # }
    ```
    */
    pub fn add_integer(&self, value: i128) -> Self {
        self.add(&Value::Number(Integer(value)))
    }

    pub fn add_integer_p(&self, pointer: &str, value: i128) -> Option<Self> {
        self.add_p(pointer, &Value::Number(Number::Integer(value)))
    }

    /**
    Adds a number to an array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Number::Integer;
    # fn main() {
    assert_eq!(
       Some(Integer(0)),
       Array::new().add_number(Integer(0)).get_number(0).ok().flatten());
    # }
    ```
    */
    pub fn add_number(&self, value: Number) -> Self {
        self.add(&Value::Number(value))
    }

    pub fn add_number_p(&self, pointer: &str, value: Number) -> Option<Self> {
        self.add_p(pointer, &Value::Number(value))
    }

    /**
    Adds an object to an array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::object::Object;
    # fn main() {
    assert_eq!(
       Some(0),
       Array::new()
           .add_object(&Object::new().add_integer("test", 0))
           .get_object(0).ok().flatten().and_then(|o| o.get_integer("test")));
    # }
    ```
    */
    pub fn add_object(&self, value: &Object) -> Self {
        self.add(&Value::Object(value.clone()))
    }

    pub fn add_object_p(&self, pointer: &str, value: &Object) -> Option<Self> {
        self.add_p(pointer, &Value::Object(value.clone()))
    }

    pub fn add_p(&self, pointer: &str, value: &Value) -> Option<Self> {
        JsonPointer::add_pointer(pointer, &Value::Array(self.clone()), value)
            .and_then(|v| v.as_array())
    }

    /**
    Adds a string to an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(
       Some("test".to_string()),
       Array::new().add_string("test").get_string(0).ok().flatten());
    # }
    ```
    */
    pub fn add_string(&self, value: &str) -> Self {
        self.add(&Value::String(value.to_string()))
    }

    pub fn add_string_p(&self, pointer: &str, value: &str) -> Option<Self> {
        self.add_p(pointer, &Value::String(value.to_string()))
    }

    /**
    Appends an array to the current array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(
       Array::new().add_integer(0).add_integer(1),
       Array::new().add_integer(0).append(&Array::new().add_integer(1)))
    # }
    ```
    */
    pub fn append(&self, array: &Array) -> Self {
        Self {
            vec: vector::append(&self.vec, &array.vec),
        }
    }

    /// Gets a JSON value from an array at a given index, which is positive and less than the length
    /// of the array.
    pub fn get(&self, index: usize) -> Result<Value, JsonIndexError> {
        self.vec.get(index).cloned().ok_or_else(|| JsonIndexError {
            index,
            len: self.len(),
        })
    }

    /**
    Gets an array from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not an array, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Number::Integer;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_number(Integer(0));

    assert_eq!(Some(Integer(0)), array.get_number(0).ok().flatten());
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_number(1).err());
    # }
    ```
    */
    pub fn get_array(&self, index: usize) -> Result<Option<Array>, JsonIndexError> {
        self.get(index).map(|v| v.as_array())
    }

    pub fn get_array_p(&self, pointer: &str) -> Option<Array> {
        self.get_p(pointer).and_then(|v| v.as_array())
    }

    /**
    Gets a bool from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not a Boolean, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(Some(true), array.get_bool(0).ok().flatten());
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_bool(1).err());
    # }
    ```
    */
    pub fn get_bool(&self, index: usize) -> Result<Option<bool>, JsonIndexError> {
        self.get(index).map(|v| v.as_bool())
    }

    pub fn get_bool_p(&self, pointer: &str) -> Option<bool> {
        self.get_p(pointer).and_then(|v| v.as_bool())
    }

    /**
    Gets a decimal from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not a decimal, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_decimal(2.0);

    assert_eq!(Some(2.0), array.get_decimal(0).ok().flatten());
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_decimal(1).err());
    # }
    ```
    */
    pub fn get_decimal(&self, index: usize) -> Result<Option<f64>, JsonIndexError> {
        self.get(index).map(|v| v.as_decimal())
    }

    pub fn get_decimal_p(&self, pointer: &str) -> Option<f64> {
        self.get_p(pointer).and_then(|v| v.as_decimal())
    }

    /**
    Gets an integer from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not an integer, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_integer(0);

    assert_eq!(Some(0), array.get_integer(0).ok().flatten());
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_integer(1).err());
    # }
    ```
    */
    pub fn get_integer(&self, index: usize) -> Result<Option<i128>, JsonIndexError> {
        self.get(index).map(|v| v.as_integer())
    }

    pub fn get_integer_p(&self, pointer: &str) -> Option<i128> {
        self.get_p(pointer).and_then(|v| v.as_integer())
    }

    /**
    Gets a number from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not a number, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Number::Integer;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_number(Integer(0));

    assert_eq!(Some(Integer(0)), array.get_number(0).ok().flatten());
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_number(1).err());
    # }
    ```
    */
    pub fn get_number(&self, index: usize) -> Result<Option<Number>, JsonIndexError> {
        self.get(index).map(|v| v.as_number())
    }

    pub fn get_number_p(&self, pointer: &str) -> Option<Number> {
        self.get_p(pointer).and_then(|v| v.as_number())
    }

    /**
    Gets an object from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not a JSON object, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::object::Object;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_object(&Object::new().add_integer("test", 0));

    assert_eq!(Some(0),
       array.get_object(0).ok().flatten().and_then(|o| o.get_integer("test")));
    assert_eq!(None, array.get_string(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_object(1).err());
    # }
    ```
    */
    pub fn get_object(&self, index: usize) -> Result<Option<Object>, JsonIndexError> {
        self.get(index).map(|v| v.as_object())
    }

    pub fn get_object_p(&self, pointer: &str) -> Option<Object> {
        self.get_p(pointer).and_then(|v| v.as_object())
    }

    pub fn get_p(&self, pointer: &str) -> Option<Value> {
        JsonPointer::get_pointer(pointer, &Value::Array(self.clone()))
    }

    /**
    Gets a string from an array at a given index, which is positive and less than the length
    of the array. If the value at the position is not a string, `None` is returned.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_string("test");

    assert_eq!(Some("test".to_string()), array.get_string(0).ok().flatten());
    assert_eq!(None, array.get_integer(0).ok().flatten());
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_string(1).err());
    # }
    ```
    */
    pub fn get_string(&self, index: usize) -> Result<Option<String>, JsonIndexError> {
        self.get(index).map(|v| v.as_string())
    }

    pub fn get_string_p(&self, pointer: &str) -> Option<String> {
        self.get_p(pointer).and_then(|v| v.as_string())
    }

    /**
    Inserts a JSON value to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Value;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(Some(true), Array::new().insert(0, &Value::Bool(true))?.get_bool(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert(&self, index: usize, value: &Value) -> Result<Self, JsonIndexError> {
        match vector::insert(&self.vec, index, value.clone()) {
            Some(v) => Ok(Self { vec: v }),
            None => Err(JsonIndexError {
                index,
                len: self.len(),
            }),
        }
    }

    /**
    Inserts an array to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(
       Some(0),
       Array::new()
           .insert_array(0, &Array::new().insert_integer(0, 0)?)?
           .get_array(0).ok().flatten().and_then(|a| a.get_integer(0).ok().flatten()));
    # Ok(())
    # }
    ```
    */
    pub fn insert_array(&self, index: usize, value: &Array) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Array(value.clone()))
    }

    /**
    Inserts a bool to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(Some(true), Array::new().insert_bool(0, true)?.get_bool(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert_bool(&self, index: usize, value: bool) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Bool(value))
    }

    /**
    Inserts a decimal to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(Some(2.0), Array::new().insert_decimal(0, 2.0)?.get_decimal(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert_decimal(&self, index: usize, value: f64) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Number(Decimal(value)))
    }

    /**
    Inserts an integer to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(Some(0), Array::new().insert_integer(0, 0)?.get_integer(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert_integer(&self, index: usize, value: i128) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Number(Integer(value)))
    }

    /**
    Inserts a number to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Number::Integer;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(
       Some(Integer(0)),
       Array::new().insert_number(0, Integer(0))?.get_number(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert_number(&self, index: usize, value: Number) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Number(value))
    }

    /**
    Inserts an object to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::object::Object;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(
       Some(0),
       Array::new()
           .insert_object(0, &Object::new().add_integer("test", 0))?
           .get_object(0).ok().flatten().and_then(|o| o.get_integer("test")));
    # Ok(())
    # }
    ```
    */
    pub fn insert_object(&self, index: usize, value: &Object) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::Object(value.clone()))
    }

    /**
    Inserts a string to an array at a given index. If the index is equal to the length of
    the array, the value is added at the end of it.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::Error;
    # fn main() -> Result<(), Error>{
    assert_eq!(
       Some("test".to_string()),
       Array::new().insert_string(0, "test")?.get_string(0).ok().flatten());
    # Ok(())
    # }
    ```
    */
    pub fn insert_string(&self, index: usize, value: &str) -> Result<Self, JsonIndexError> {
        self.insert(index, &Value::String(value.to_string()))
    }

    /**
    Indicates if the array is empty or not.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(true, Array::new().is_empty());
    assert_eq!(false, Array::new().add_integer(0).is_empty());
    # }
    ```
    */
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /**
    Returns an iterator over the values of the array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    let array = Array::new().add_string("test").add_integer(1);

    assert_eq!(array, Array::from_iter(array.iter()));
    # }
    ```
    */
    pub fn iter(&'_ self) -> ArrayIter<'_> {
        ArrayIter {
            iter: self.vec.iter(),
        }
    }

    /**
    Returns the length of an array.
    ```rust
    # use immutable_json::array::Array;
    # fn main() {
    assert_eq!(0, Array::new().len());
    assert_eq!(1, Array::new().add_integer(0).len());
    # }
    ```
    */
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Creates an empty JSON array.
    pub fn new() -> Self {
        Self { vec: Vector::new() }
    }

    /**
    Removes a JSON value from an array at a given index, which is positive and less than the
    length of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    assert_eq!(Some(0), Array::new().add_integer(0).remove(0).ok().map(|a| a.len()));
    assert_eq!(Some(JsonIndexError { index: 1, len: 0, }), Array::new().remove(1).err());
    # }
    ```
    */
    pub fn remove(&self, index: usize) -> Result<Self, JsonIndexError> {
        if index >= self.len() {
            Err(JsonIndexError {
                index,
                len: self.len(),
            })
        } else {
            let mut new_vec = self.vec.clone();

            new_vec.remove(index);
            Ok(Self { vec: new_vec })
        }
    }

    pub fn remove_p(&self, pointer: &str) -> Option<Self> {
        JsonPointer::remove_pointer(pointer, &Value::Array(self.clone())).and_then(|v| v.as_array())
    }

    /// Sets a JSON value in an array at a given index, which is positive and less than the length
    /// of the array.
    pub fn set(&self, index: usize, value: &Value) -> Result<Self, JsonIndexError> {
        if index >= self.len() {
            Err(JsonIndexError {
                index,
                len: self.len(),
            })
        } else {
            let mut new_vec = self.vec.clone();

            new_vec.set(index, value.clone());
            Ok(Self { vec: new_vec })
        }
    }

    /**
    Sets an array in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some(0),
       array
           .set_array(0, &Array::new().add_integer(0))
           .ok()
           .and_then(|a| {
               a.get_array(0).ok().flatten().and_then(|a| a.get_integer(0).ok().flatten())
           }));
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_bool(1, false).err());
    # }
    ```
    */
    pub fn set_array(&self, index: usize, value: &Array) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Array(value.clone()))
    }

    pub fn set_array_p(&self, pointer: &str, value: &Array) -> Option<Self> {
        self.set_p(pointer, &Value::Array(value.clone()))
    }

    /**
    Sets a bool in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some(false),
       array
           .set_bool(0, false)
           .ok()
           .and_then(|a| a.get_bool(0).ok().flatten()));
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_bool(1, false).err());
    # }
    ```
    */
    pub fn set_bool(&self, index: usize, value: bool) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Bool(value))
    }

    pub fn set_bool_p(&self, pointer: &str, value: bool) -> Option<Self> {
        self.set_p(pointer, &Value::Bool(value))
    }

    /**
    Sets a decimal in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some(3.0),
       array
           .set_decimal(0, 3.0)
           .ok()
           .and_then(|a| a.get_decimal(0).ok().flatten()));
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_decimal(1, 1.0).err());
    # }
    ```
    */
    pub fn set_decimal(&self, index: usize, value: f64) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Number(Decimal(value)))
    }

    pub fn set_decimal_p(&self, pointer: &str, value: f64) -> Option<Self> {
        self.set_p(pointer, &Value::Number(Number::Decimal(value)))
    }

    /**
    Sets an integer in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some(3),
       array
           .set_integer(0, 3)
           .ok()
           .and_then(|a| a.get_integer(0).ok().flatten()));
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_integer(1, 1).err());
    # }
    ```
    */
    pub fn set_integer(&self, index: usize, value: i128) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Number(Integer(value)))
    }

    pub fn set_integer_p(&self, pointer: &str, value: i128) -> Option<Self> {
        self.set_p(pointer, &Value::Number(Number::Integer(value)))
    }

    /**
    Sets a number in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::api::Number::Decimal;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some(Decimal(3.0)),
       array
           .set_number(0, Decimal(3.0))
           .ok()
           .and_then(|a| a.get_number(0).ok().flatten()));
    assert_eq!(
       Some(JsonIndexError { index: 1, len: 1, }),
       array.set_number(1, Decimal(1.0)).err());
    # }
    ```
    */
    pub fn set_number(&self, index: usize, value: Number) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Number(value))
    }

    pub fn set_number_p(&self, pointer: &str, value: Number) -> Option<Self> {
        self.set_p(pointer, &Value::Number(value))
    }

    /**
    Sets an object in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::object::Object;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some("test".to_string()),
       array
           .set_object(0, &Object::new().add_string("test", "test"))
           .ok()
           .and_then(|a| a.get_object(0).ok().flatten().and_then(|o| o.get_string("test"))));
    assert_eq!(
       Some(JsonIndexError { index: 1, len: 1, }),
       array.set_object(1, &Object::new()).err());
    # }
    ```
    */
    pub fn set_object(&self, index: usize, value: &Object) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::Object(value.clone()))
    }

    pub fn set_object_p(&self, pointer: &str, value: &Object) -> Option<Self> {
        self.set_p(pointer, &Value::Object(value.clone()))
    }

    pub fn set_p(&self, pointer: &str, value: &Value) -> Option<Self> {
        JsonPointer::set_pointer(pointer, &Value::Array(self.clone()), value)
            .and_then(|v| v.as_array())
    }

    /**
    Sets a string in an array at a given index, which is positive and less than the length
    of the array.
    ```rust
    # use immutable_json::array::Array;
    # use immutable_json::error::JsonIndexError;
    # fn main() {
    let array = Array::new().add_bool(true);

    assert_eq!(
       Some("test".to_string()),
       array
           .set_string(0, "test")
           .ok()
           .and_then(|a| a.get_string(0).ok().flatten()));
    assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_string(1, "test").err());
    # }
    ```
    */
    pub fn set_string(&self, index: usize, value: &str) -> Result<Self, JsonIndexError> {
        self.set(index, &Value::String(value.to_string()))
    }

    pub fn set_string_p(&self, pointer: &str, value: &str) -> Option<Self> {
        self.set_p(pointer, &Value::String(value.to_string()))
    }
}

#[derive(Clone)]
pub struct ArrayIter<'a> {
    iter: Iter<'a, Value, DefaultSharedPtr>,
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}
