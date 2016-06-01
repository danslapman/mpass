use std::fs::File;
use std::io::{Read, Write};

use domain::RecordCell;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

pub struct Store {
    pub path: String,
    pub key: Vec<u8>
}

impl Store {
    fn read_all(&self) -> Vec<RecordCell> {
        let mut contents = Vec::<u8>::new();
        let _ = File::open(self.path.clone()).map(|mut f| f.read_to_end(&mut contents));
        let entries: Vec<RecordCell> = decode(&contents[..]).unwrap_or(Vec::<RecordCell>::new());
        entries
    }
    
    pub fn persist(&self, entry: RecordCell) -> () {
        let mut entries = self.read_all();
        entries.push(entry);
        
        let _ = File::create(self.path.clone()).unwrap()
            .write(encode(&entries, SizeLimit::Infinite).unwrap().as_slice()).unwrap();
        ()
    }
    
    pub fn read(&self, domain: String) -> Option<RecordCell> {
        let mut contents = Vec::<u8>::new();
        let _ = File::open(self.path.clone()).unwrap().read_to_end(&mut contents);
        let entries: Vec<RecordCell> = decode(&contents[..]).unwrap();
        entries.into_iter().fold(None, move |acc, el| if el.domain == domain { Some(el) } else { acc })
    }
} 