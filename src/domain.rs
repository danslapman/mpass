#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct RecordCell {
    pub domain: String,
    pub username: String,
    pub password: String
}