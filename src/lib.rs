//! With this crate, JSON objects and arrays can be transformed in an immutable way, which means
//! that every "change" leads to a new value. Persistent collections are used internally to make
//! changes and cloning cheap. Conversions from and to values in the
//! [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) crate are provided.
pub mod api;
pub mod array;
pub mod error;
pub mod object;
pub mod serde;
