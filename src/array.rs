use crate::api::Number::{Decimal, Integer};
use crate::api::{Number, Value};
use crate::error::JsonIndexError;
use crate::object::Object;
use imbl::Vector;
use imbl::shared_ptr::DefaultSharedPtr;
use imbl::vector::Iter;
use std::cmp::PartialEq;

#[derive(Clone, Debug)]
pub struct Array {
    vec: Vector<Value>,
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Value> for Array {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        let mut result = Array::new();

        for i in iter {
            result = result.add(i);
        }

        result
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = Value;
    type IntoIter = ArrayIter<'a>;

    fn into_iter(self) -> ArrayIter<'a> {
        self.iter()
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}

impl Array {
    /// Adds a JSON value to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::api::Value;
    /// # fn main() {
    /// assert_eq!(Some(true), Array::new().add(Value::Bool(true)).get_bool(0).ok().flatten());
    /// # }
    /// ```
    pub fn add(&self, value: Value) -> Self {
        let mut new_vec = self.vec.clone();

        new_vec.push_back(value);
        Array { vec: new_vec }
    }

    /// Adds an array to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(
    ///     Some(0),
    ///     Array::new()
    ///         .add_array(Array::new().add_integer(0))
    ///         .get_array(0).ok().flatten().and_then(|a| a.get_integer(0).ok().flatten()));
    /// # }
    /// ```
    pub fn add_array(&self, value: Array) -> Self {
        self.add(Value::Array(value))
    }

    /// Adds a bool to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(Some(true), Array::new().add_bool(true).get_bool(0).ok().flatten());
    /// # }
    /// ```
    pub fn add_bool(&self, value: bool) -> Self {
        self.add(Value::Bool(value))
    }

    /// Adds a decimal to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(Some(2.0), Array::new().add_decimal(2.0).get_decimal(0).ok().flatten());
    /// # }
    /// ```
    pub fn add_decimal(&self, value: f64) -> Self {
        self.add(Value::Number(Decimal(value)))
    }

    /// Adds an integer to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(Some(0), Array::new().add_integer(0).get_integer(0).ok().flatten());
    /// # }
    /// ```
    pub fn add_integer(&self, value: i128) -> Self {
        self.add(Value::Number(Integer(value)))
    }

    /// Adds a number to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::api::Number::Integer;
    /// # fn main() {
    /// assert_eq!(
    ///   Some(Integer(0)),
    ///   Array::new().add_number(Integer(0)).get_number(0).ok().flatten());
    /// # }
    /// ```
    pub fn add_number(&self, value: Number) -> Self {
        self.add(Value::Number(value))
    }

    /// Adds an object to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(
    ///     Some(0),
    ///     Array::new()
    ///         .add_object(Object::new().add_integer("test", 0))
    ///         .get_object(0).ok().flatten().and_then(|o| o.get_integer("test")));
    /// # }
    /// ```
    pub fn add_object(&self, value: Object) -> Self {
        self.add(Value::Object(value))
    }

    /// Adds a string to an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(
    ///     Some("test".to_string()),
    ///     Array::new().add_string("test").get_string(0).ok().flatten());
    /// # }
    /// ```
    pub fn add_string(&self, value: &str) -> Self {
        self.add(Value::String(value.to_string()))
    }

    /// Gets a JSON value from an array at a given index, which is positive and less than the length
    /// of the array.
    pub fn get(&self, index: usize) -> Result<Value, JsonIndexError> {
        self.vec.get(index).cloned().ok_or_else(|| JsonIndexError {
            index,
            len: self.len(),
        })
    }

    /// Gets an array from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::api::Number::Integer;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_number(Integer(0));
    ///
    /// assert_eq!(Some(Integer(0)), array.get_number(0).ok().flatten());
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_number(1).err());
    /// # }
    /// ```
    pub fn get_array(&self, index: usize) -> Result<Option<Array>, JsonIndexError> {
        self.get(index).map(|v| v.as_array())
    }

    /// Gets a bool from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(Some(true), array.get_bool(0).ok().flatten());
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_bool(1).err());
    /// # }
    /// ```
    pub fn get_bool(&self, index: usize) -> Result<Option<bool>, JsonIndexError> {
        self.get(index).map(|v| v.as_bool())
    }

    /// Gets a decimal from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_decimal(2.0);
    ///
    /// assert_eq!(Some(2.0), array.get_decimal(0).ok().flatten());
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_decimal(1).err());
    /// # }
    /// ```
    pub fn get_decimal(&self, index: usize) -> Result<Option<f64>, JsonIndexError> {
        self.get(index).map(|v| v.as_decimal())
    }

    /// Gets an integer from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_integer(0);
    ///
    /// assert_eq!(Some(0), array.get_integer(0).ok().flatten());
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_integer(1).err());
    /// # }
    /// ```
    pub fn get_integer(&self, index: usize) -> Result<Option<i128>, JsonIndexError> {
        self.get(index).map(|v| v.as_integer())
    }

    /// Gets a number from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::api::Number::Integer;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_number(Integer(0));
    ///
    /// assert_eq!(Some(Integer(0)), array.get_number(0).ok().flatten());
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_number(1).err());
    /// # }
    /// ```
    pub fn get_number(&self, index: usize) -> Result<Option<Number>, JsonIndexError> {
        self.get(index).map(|v| v.as_number())
    }

    /// Gets an object from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::object::Object;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_object(Object::new().add_integer("test", 0));
    ///
    /// assert_eq!(Some(0),
    ///     array.get_object(0).ok().flatten().and_then(|o| o.get_integer("test")));
    /// assert_eq!(None, array.get_string(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_object(1).err());
    /// # }
    /// ```
    pub fn get_object(&self, index: usize) -> Result<Option<Object>, JsonIndexError> {
        self.get(index).map(|v| v.as_object())
    }

    /// Gets a string from an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_string("test");
    ///
    /// assert_eq!(Some("test".to_string()), array.get_string(0).ok().flatten());
    /// assert_eq!(None, array.get_integer(0).ok().flatten());
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.get_string(1).err());
    /// # }
    /// ```
    pub fn get_string(&self, index: usize) -> Result<Option<String>, JsonIndexError> {
        self.get(index).map(|v| v.as_string())
    }

    /// Indicates if the array is empty or not.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(true, Array::new().is_empty());
    /// assert_eq!(false, Array::new().add_integer(0).is_empty());
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns an iterator over the values of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// let array = Array::new().add_string("test").add_integer(1);
    ///
    /// assert_eq!(array, Array::from_iter(array.iter()));
    /// # }
    /// ```
    pub fn iter(&'_ self) -> ArrayIter<'_> {
        ArrayIter {
            iter: self.vec.iter(),
        }
    }

    /// Returns the length of an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # fn main() {
    /// assert_eq!(0, Array::new().len());
    /// assert_eq!(1, Array::new().add_integer(0).len());
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn new() -> Self {
        Array { vec: Vector::new() }
    }

    /// Removes a JSON value from an array at a given index, which is positive and less than the
    /// length of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// assert_eq!(Some(0), Array::new().add_integer(0).remove(0).ok().map(|a| a.len()));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 0, }), Array::new().remove(1).err());
    /// # }
    /// ```
    pub fn remove(&self, index: usize) -> Result<Self, JsonIndexError> {
        if index >= self.len() {
            Err(JsonIndexError {
                index,
                len: self.len(),
            })
        } else {
            let mut new_vec = self.vec.clone();

            new_vec.remove(index);
            Ok(Array { vec: new_vec })
        }
    }

    /// Sets a JSON value in an array at a given index, which is positive and less than the length
    /// of the array.
    pub fn set(&self, index: usize, value: Value) -> Result<Self, JsonIndexError> {
        if index >= self.len() {
            Err(JsonIndexError {
                index,
                len: self.len(),
            })
        } else {
            let mut new_vec = self.vec.clone();

            new_vec.set(index, value);
            Ok(Array { vec: new_vec })
        }
    }

    /// Sets an array in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some(0),
    ///     array
    ///       .set_array(0, Array::new().add_integer(0))
    ///       .ok()
    ///       .and_then(|a|
    ///           a.get_array(0).ok().flatten().and_then(|a| a.get_integer(0).ok().flatten())));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_bool(1, false).err());
    /// # }
    /// ```
    pub fn set_array(&self, index: usize, value: Array) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Array(value))
    }

    /// Sets a bool in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some(false),
    ///     array
    ///       .set_bool(0, false)
    ///       .ok()
    ///       .and_then(|a| a.get_bool(0).ok().flatten()));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_bool(1, false).err());
    /// # }
    /// ```
    pub fn set_bool(&self, index: usize, value: bool) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Bool(value))
    }

    /// Sets a decimal in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some(3.0),
    ///     array
    ///       .set_decimal(0, 3.0)
    ///       .ok()
    ///       .and_then(|a| a.get_decimal(0).ok().flatten()));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_decimal(1, 1.0).err());
    /// # }
    /// ```
    pub fn set_decimal(&self, index: usize, value: f64) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Number(Decimal(value)))
    }

    /// Sets an integer in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some(3),
    ///     array
    ///       .set_integer(0, 3)
    ///       .ok()
    ///       .and_then(|a| a.get_integer(0).ok().flatten()));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_integer(1, 1).err());
    /// # }
    /// ```
    pub fn set_integer(&self, index: usize, value: i128) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Number(Integer(value)))
    }

    /// Sets a number in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::api::Number::Decimal;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some(Decimal(3.0)),
    ///     array
    ///       .set_number(0, Decimal(3.0))
    ///       .ok()
    ///       .and_then(|a| a.get_number(0).ok().flatten()));
    /// assert_eq!(
    ///     Some(JsonIndexError { index: 1, len: 1, }),
    ///     array.set_number(1, Decimal(1.0)).err());
    /// # }
    /// ```
    pub fn set_number(&self, index: usize, value: Number) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Number(value))
    }

    /// Sets an object in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::object::Object;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some("test".to_string()),
    ///     array
    ///       .set_object(0, Object::new().add_string("test", "test"))
    ///       .ok()
    ///       .and_then(|a| a.get_object(0).ok().flatten().and_then(|o| o.get_string("test"))));
    /// assert_eq!(
    ///     Some(JsonIndexError { index: 1, len: 1, }),
    ///     array.set_object(1, Object::new()).err());
    /// # }
    /// ```
    pub fn set_object(&self, index: usize, value: Object) -> Result<Self, JsonIndexError> {
        self.set(index, Value::Object(value))
    }

    /// Sets a string in an array at a given index, which is positive and less than the length
    /// of the array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::error::JsonIndexError;
    /// # fn main() {
    /// let array = Array::new().add_bool(true);
    ///
    /// assert_eq!(
    ///     Some("test".to_string()),
    ///     array
    ///       .set_string(0, "test")
    ///       .ok()
    ///       .and_then(|a| a.get_string(0).ok().flatten()));
    /// assert_eq!(Some(JsonIndexError { index: 1, len: 1, }), array.set_string(1, "test").err());
    /// # }
    /// ```
    pub fn set_string(&self, index: usize, value: &str) -> Result<Self, JsonIndexError> {
        self.set(index, Value::String(value.to_string()))
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
