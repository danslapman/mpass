#[derive(RustcEncodable, RustcDecodable, PartialEq, Clone)]
pub enum Record {
    Credentials { domain: String, username: String, password: String },
    Command { name: String, command_line: String }
}

impl Record {
    pub fn get_name(&self) -> String {
        match *self {
            Record::Credentials { domain: ref d, ..} => d.clone(),
            Record::Command { name: ref n, ..} => n.clone()
        }
    }

    pub fn is_same_case(&self, other: &Record) -> bool {
        match (self.clone(), other.clone()) {
            (Record::Credentials {..}, Record::Credentials {..}) => true,
            (Record::Command { ..}, Record::Command { ..}) => true,
            _ => false
        }
    }

    pub fn is_credentials(&self) -> bool {
        match *self {
            Record::Credentials {..} => true, _ => false
        }
    }
}