use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct StringError {
    data: String,
}

impl StringError {
    pub fn new(s: &str) -> StringError {
        StringError {
            data: s.to_owned(),
        }
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.data
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
