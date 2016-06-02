#[derive(RustcEncodable, RustcDecodable, PartialEq, Clone)]
pub struct RecordCell {
    pub domain: String,
    pub username: String,
    pub password: String
}