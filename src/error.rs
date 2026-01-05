use std::fmt;

#[derive(Debug)]
pub struct JsonIndexError {
    pub index: usize,
    pub len: usize,
}

impl fmt::Display for JsonIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the index {} is higher than {}",
            self.index,
            self.len - 1
        )
    }
}

impl PartialEq for JsonIndexError {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.len == other.len
    }
}
