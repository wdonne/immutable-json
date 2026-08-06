//! With this crate, JSON objects and arrays can be transformed in an immutable way, which means
//! that every "change" leads to a new value. Persistent collections are used internally to make
//! changes and cloning cheap. Conversions from and to values in the
//! [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) crate are provided.
//!
//! The crate feature `jaq` enables the transformation of JSON with the
//! [jq language](https://jqlang.org). It is powered by the
//! [jaq engine](https://github.com/01mf02/jaq).
//!
//! The crate feature `patch` enables working with JSON patches. It uses the `json-patch` crate.
//! 
//! The crate feature `bson` provides conversions from and to the BSON format.
pub mod api;
pub mod array;
#[cfg(feature = "bson")]
pub mod bson;
pub mod error;
#[cfg(feature = "jaq")]
pub mod jaq;
pub mod object;
#[cfg(feature = "patch")]
pub mod patch;
pub mod pointer;
pub mod serde;
