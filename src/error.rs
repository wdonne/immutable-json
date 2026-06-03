#[cfg(feature = "patch")]
use crate::api::Value;
#[cfg(feature = "patch")]
use crate::array::Array;
#[cfg(feature = "jaq")]
use jaq_core::compile::Errors;
#[cfg(feature = "jaq")]
use jaq_core::load::File;
use std::fmt;
use std::fmt::{Display, Formatter};
#[cfg(feature = "patch")]
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    /// Cannot convert from another JSON representation to an immutable JSON value.
    ConvertFrom,
    /// Cannot convert from an immutable JSON value to another JSON representation.
    ConvertTo,
    #[cfg(feature = "jaq")]
    /// A jaq compilation error. It requires the `jaq` crate feature.
    JaqCompile(String),
    #[cfg(feature = "jaq")]
    /// An exception while running a jaq filter. It requires the `jaq` crate feature.
    JaqException(String),
    #[cfg(feature = "jaq")]
    /// A jaq transformation didn't produce any result. It requires the `jaq` crate feature.
    JaqNoResult,
    /// Wrong array index.
    JsonIndex(JsonIndexError),
    #[cfg(feature = "patch")]
    /// Invalid JSON patch.
    JsonPatch(JsonPatchError),
    /// Invalid JSON pointer.
    JsonPointer(String),
    /// A serde_json error.
    SerdeJson(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConvertFrom => write!(f, "cannot convert from serde_json or jaq_json"),
            Error::ConvertTo => write!(f, "cannot convert to serde_json or jaq_json"),
            #[cfg(feature = "jaq")]
            Error::JaqCompile(s) => write!(f, "jaq compilation error: {}", s),
            #[cfg(feature = "jaq")]
            Error::JaqException(s) => write!(f, "jaq exception: {}", s),
            #[cfg(feature = "jaq")]
            Error::JaqNoResult => write!(f, "the jq expression did not yield a result"),
            Error::JsonIndex(i) => write!(f, "{}", i),
            #[cfg(feature = "patch")]
            Error::JsonPatch(p) => write!(f, "{}", p),
            Error::JsonPointer(p) => write!(f, "malformed JSON pointer {}", p),
            Error::SerdeJson(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<JsonIndexError> for Error {
    fn from(value: JsonIndexError) -> Self {
        Error::JsonIndex(value)
    }
}

#[cfg(feature = "patch")]
impl From<JsonPatchError> for Error {
    fn from(value: JsonPatchError) -> Self {
        Error::JsonPatch(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

#[cfg(feature = "jaq")]
impl From<Vec<(File<&str, PathBuf>, jaq_core::load::Error<&str>)>> for Error {
    fn from(errors: Vec<(File<&str, PathBuf>, jaq_core::load::Error<&str>)>) -> Self {
        Error::JaqCompile(
            errors
                .iter()
                .map(|(file, error)| {
                    pathbuf_to_string(file.path.clone())
                        + ": code: "
                        + file.code
                        + ": error: "
                        + &jaq_load_errors(error)
                })
                .fold(String::new(), |s, el| s + "\n" + &el),
        )
    }
}

#[cfg(feature = "jaq")]
impl<'a> From<Errors<&'a str, PathBuf>> for Error {
    fn from(errors: Errors<&'a str, PathBuf>) -> Self {
        Error::JaqCompile(
            errors
                .iter()
                .map(|(file, error)| {
                    pathbuf_to_string(file.path.clone())
                        + ": code: "
                        + file.code
                        + ":error: "
                        + &jaq_compile_errors(error)
                })
                .fold(String::new(), |s, el| s + "\n" + &el),
        )
    }
}

/// Denotes a wrong JSON array index.
#[derive(Debug, PartialEq)]
pub struct JsonIndexError {
    /// The index.
    pub index: usize,
    /// The length of the array.
    pub len: usize,
}

impl Display for JsonIndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the index {} is higher than {}",
            self.index,
            self.len - 1
        )
    }
}

#[derive(Debug)]
#[cfg(feature = "patch")]
pub struct JsonPatchError {
    pub error: String,
    pub patch: Box<Array>,
    pub source: Box<Value>,
}

#[cfg(feature = "patch")]
impl Display for JsonPatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot apply path '{}' to source '{}' because of '{}",
            self.patch, self.source, self.error
        )
    }
}

#[cfg(feature = "jaq")]
fn jaq_compile_errors(errors: &Vec<jaq_core::compile::Error<&str>>) -> String {
    errors
        .iter()
        .map(|(name, undefined)| undefined.as_str().to_string() + " " + name + " is undefined")
        .fold(String::new(), |s, el| s + "\n" + &el)
}

#[cfg(feature = "jaq")]
fn jaq_load_errors(error: &jaq_core::load::Error<&str>) -> String {
    match error {
        jaq_core::load::Error::Io(v) => v
            .iter()
            .map(|(p, e)| p.to_string() + ": " + e)
            .fold(String::new(), |s, el| s + "\n" + &el),
        jaq_core::load::Error::Lex(v) => jaq_read_error(v.iter().map(|(e, g)| (e.as_str(), *g))),
        jaq_core::load::Error::Parse(v) => jaq_read_error(v.iter().map(|(e, g)| (e.as_str(), *g))),
    }
}

#[cfg(feature = "jaq")]
fn jaq_read_error<'a>(errors: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    errors
        .map(|(e, g)| "expected ".to_string() + e + ", got " + g)
        .fold(String::new(), |s, el| s + "\n" + &el)
}

#[cfg(feature = "jaq")]
fn pathbuf_to_string(path_buf: PathBuf) -> String {
    path_buf.into_os_string().to_str().unwrap_or("").to_string()
}
