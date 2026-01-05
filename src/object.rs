use crate::api::{Number, Value};
use crate::array::Array;
use imbl::HashMap;
use imbl::hashmap::Iter;
use imbl::shared_ptr::DefaultSharedPtr;
use crate::api::Number::{Decimal, Integer};

#[derive(Clone, Debug)]
pub struct Object {
    map: HashMap<String, Value>,
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<(String, Value)> for Object {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        let mut result = Object::new();

        for i in iter {
            result = result.add(&i.0, i.1);
        }

        result
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (String, Value);
    type IntoIter = ObjectIter<'a>;

    fn into_iter(self) -> ObjectIter<'a> {
        self.iter()
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl Object {
    /// Adds a field to an object.
    /// ```rust
    /// # use immutable_json::api::Value::String;
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some("test".to_string()),
    ///     Object::new().add("test", String("test".to_string())).get_string("test"));
    /// # }
    /// ```
    pub fn add(&self, key: &str, value: Value) -> Self {
        let mut new_map = self.map.clone();

        new_map.insert(key.to_string(), value);
        Object { map: new_map }
    }

    /// Adds a field to an object as an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some(1),
    ///     Object::new()
    ///         .add_array("test", Array::new().add_integer(1))
    ///         .get_array("test").and_then(|a| a.get_integer(0).ok()?));
    /// # }
    /// ```
    pub fn add_array(&self, key: &str, value: Array) -> Self {
        self.add(key, Value::Array(value))
    }

    /// Adds a field to an object as a bool.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some(true), Object::new().add_bool("test", true).get_bool("test"));
    /// # }
    /// ```
    pub fn add_bool(&self, key: &str, value: bool) -> Self {
        self.add(key, Value::Bool(value))
    }

    /// Adds a field to an object as a decimal.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some(3.0), Object::new().add_decimal("test", 3.0).get_decimal("test"));
    /// # }
    /// ```
    pub fn add_decimal(&self, key: &str, value: f64) -> Self {
        self.add(key, Value::Number(Decimal(value)))
    }

    /// Adds a field to an object as an integer.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some(3), Object::new().add_integer("test", 3).get_integer("test"));
    /// # }
    /// ```
    pub fn add_integer(&self, key: &str, value: i128) -> Self {
        self.add(key, Value::Number(Integer(value)))
    }

    /// Adds a field to an object as a number.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # use immutable_json::api::Number::Integer;
    /// # fn main() {
    /// assert_eq!(Some(3),
    ///     Object::new()
    ///         .add_number("test", Integer(3))
    ///         .get_number("test").and_then(|n| n.as_integer()));
    /// # }
    /// ```
    pub fn add_number(&self, key: &str, value: Number) -> Self {
        self.add(key, Value::Number(value))
    }

    /// Adds a field to an object as an object.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # use immutable_json::api::Number::Integer;
    /// # fn main() {
    /// assert_eq!(Some(1),
    ///     Object::new()
    ///         .add_object("test", Object::new().add_integer("test", 1))
    ///         .get_object("test").and_then(|o| o.get_integer("test")));
    /// # }
    /// ```
    pub fn add_object(&self, key: &str, value: Object) -> Self {
        self.add(key, Value::Object(value))
    }

    /// Adds a field to an object as an integer.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// assert_eq!(Some("test".to_string()),
    ///     Object::new().add_string("test", "test").get_string("test"));
    /// # }
    /// ```
    pub fn add_string(&self, key: &str, value: &str) -> Self {
        self.add(key, Value::String(value.to_string()))
    }

    /// Returns a value from the object if it exists at the given key.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new().add_string("test1", "test");
    ///
    /// assert_eq!(Some("test".to_string()), object.get_string("test1"));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(&key.to_string())
    }

    /// Returns a value from the object if it exists at the given key and if it is an array.
    /// ```rust
    /// # use immutable_json::array::Array;
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_array("test1", Array::new().add_integer(3))
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(3), object.get_array("test1").and_then(|a| a.get_integer(0).ok()?));
    /// assert_eq!(None, object.get_array("test2"));
    /// # }
    /// ```
    pub fn get_array(&self, key: &str) -> Option<Array> {
        self.get(key).and_then(|v| v.as_array())
    }

    /// Returns a value from the object if it exists at the given key and if it is a bool.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_bool("test1", true)
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(true), object.get_bool("test1"));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// Returns a value from the object if it exists at the given key and if it is a decimal.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_decimal("test1", 3.0)
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(3.0), object.get_decimal("test1"));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get_decimal(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_decimal())
    }

    /// Returns a value from the object if it exists at the given key and if it is an integer.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_decimal("test1", 3.0)
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(3.0), object.get_decimal("test1"));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get_integer(&self, key: &str) -> Option<i128> {
        self.get(key).and_then(|v| v.as_integer())
    }

    /// Returns a value from the object if it exists at the given key and if it is a number.
    /// ```rust
    /// # use immutable_json::api::Number;
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_number("test1", Number::Decimal(3.0))
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(3.0), object.get_number("test1").and_then(|n| n.as_decimal()));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get_number(&self, key: &str) -> Option<Number> {
        self.get(key).and_then(|v| v.as_number())
    }

    /// Returns a value from the object if it exists at the given key and if it is an object.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_object("test1", Object::new().add_integer("test", 3))
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some(3), object.get_object("test1").and_then(|a| a.get_integer("test")));
    /// assert_eq!(None, object.get_array("test2"));
    /// # }
    /// ```
    pub fn get_object(&self, key: &str) -> Option<Object> {
        self.get(key).and_then(|v| v.as_object())
    }

    /// Returns a value from the object if it exists at the given key and if it is a string.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new()
    ///     .add_string("test1", "test")
    ///     .add_integer("test2", 3);
    ///
    /// assert_eq!(Some("test".to_string()), object.get_string("test1"));
    /// assert_eq!(None, object.get_string("test2"));
    /// # }
    /// ```
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_string())
    }

    /// Returns `true` if the object has a field with the given key.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new().add_string("test1", "test");
    ///
    /// assert_eq!(true, object.has_key("test1"));
    /// assert_eq!(false, object.has_key("test2"));
    /// # }
    /// ```
    pub fn has_key(&self, key: &str) -> bool {
        self.map.contains_key(&key.to_string())
    }

    /// Returns an iterator over the key/value tuples of the object.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new().add_string("test1", "test1").add_integer("test2", 1);
    ///
    /// assert_eq!(object, Object::from_iter(object.iter()));
    /// # }
    /// ```
    pub fn iter(&'_ self) -> ObjectIter<'_> {
        ObjectIter {
            iter: self.map.iter(),
        }
    }

    pub fn new() -> Self {
        Object {
            map: HashMap::new(),
        }
    }

    /// Removes the field with the given key from the object.
    /// ```rust
    /// # use immutable_json::object::Object;
    /// # fn main() {
    /// let object = Object::new().add_string("test", "test");
    ///
    /// assert_eq!(true, object.has_key("test"));
    /// assert_eq!(false, object.remove("test").has_key("test"));
    /// # }
    /// ```
    pub fn remove(&self, key: &str) -> Self {
        let mut new_map = self.map.clone();

        new_map.remove(&key.to_string());
        Object { map: new_map }
    }
}

#[derive(Clone)]
pub struct ObjectIter<'a> {
    iter: Iter<'a, String, Value, DefaultSharedPtr>,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|i| (i.0.clone(), i.1.clone()))
    }
}
