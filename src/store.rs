use std::fs::File;
use std::io::{Read, Write};

use domain::RecordCell;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

pub struct Store {
    pub path: String
}

impl Store {
    pub fn persist(self, entry: &RecordCell) -> () {
        let _ = File::create(self.path).unwrap().write(encode(entry, SizeLimit::Infinite).unwrap().as_slice());
        ()
    }
    
    pub fn read(self) -> RecordCell {
        let mut contents = Vec::<u8>::new();
        let _ = File::open(self.path).unwrap().read_to_end(&mut contents);
        let entry: RecordCell = decode(&contents[..]).unwrap();
        entry
    }
}