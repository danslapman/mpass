use std::fs::File;
use std::io::{Read, Write};

use domain::RecordCell;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

#[derive(Clone)]
pub struct Store {
    pub path: String
}

impl Store {
    fn read_all(&self) -> Vec<RecordCell> {
        let mut contents = Vec::<u8>::new();
        let _ = File::open(self.clone().path).map(|mut f| f.read_to_end(&mut contents));
        let entries: Vec<RecordCell> = decode(&contents[..]).unwrap_or(Vec::<RecordCell>::new());
        entries
    }
    
    pub fn persist(&self, entry: RecordCell) -> () {
        let mut entries = self.clone().read_all();
        entries.push(entry);
        
        let _ = File::create(self.clone().path).unwrap()
            .write(encode(&entries, SizeLimit::Infinite).unwrap().as_slice()).unwrap();
        ()
    }
    
    pub fn read(&self, domain: String) -> Option<RecordCell> {
        let mut contents = Vec::<u8>::new();
        let _ = File::open(self.clone().path).unwrap().read_to_end(&mut contents);
        let entries: Vec<RecordCell> = decode(&contents[..]).unwrap();
        entries.into_iter().fold(None, move |acc, el| if el.domain == domain { Some(el) } else { acc })
    }
} 