use std::fmt::{Display, Formatter, Result};

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct RecordCell {
    pub domain: String,
    pub username: String,
    pub password: String
}

impl Display for RecordCell {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}, {}, {}", self.domain, self.username, self.password)
    }
}