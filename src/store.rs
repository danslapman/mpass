use std::fs::File;
use std::io::{Read, Write};

use domain::RecordCell;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use crypter::{encrypt, decrypt};
use rand::{ Rng, OsRng };

pub struct Store {
    pub path: String,
    pub key: Vec<u8>
}

impl Store {
    fn read_all(&self) -> Vec<RecordCell> {
        let mut contents = Vec::<u8>::new();
        match File::open(self.path.clone()) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut contents);
                let (iv, data) = contents.split_at(16);
                let decrypted_data = decrypt(data, self.key.as_slice(), iv).expect("Error while decrypting");
                let entries: Vec<RecordCell> = decode(decrypted_data.as_slice()).expect("Error while decoding");
                entries
            },
            Err(_) => Vec::<RecordCell>::new()
        }
        
    }
    
    fn write_all(&self, entries: Vec<RecordCell>) -> () {
        let encoded_entries = encode(&entries, SizeLimit::Infinite).expect("Error while encoding");
        
        let mut iv: [u8; 16] = [0; 16];
        let mut rng = OsRng::new().ok().unwrap();
        rng.fill_bytes(&mut iv);
        
        let mut encrypted_entries = encrypt(encoded_entries.as_slice(), self.key.as_slice(), &iv).expect("Error while encrypting");
        let mut data = Vec::from(&iv[..]);
        data.append(&mut encrypted_entries);
        
        let _ = File::create(self.path.clone()).expect("Error while creating file")
            .write(data.as_slice()).expect("Error while writing file");
        ()
    }
    
    pub fn persist(&self, entry: RecordCell) -> () {
        let mut entries = self.read_all();
        entries.push(entry);
        self.write_all(entries);
    }
    
    pub fn read(&self, domain: String) -> Option<RecordCell> {
        self.read_all().into_iter().fold(None, move |acc, el| if el.domain == domain { Some(el) } else { acc })
    }
    
    pub fn remove(&self, domain: String) -> bool {
        let mut entries = self.read_all();
        let contains = entries.clone().into_iter().any(|el| el.domain == domain);
        entries.retain(move |el| el.domain != domain);
        self.write_all(entries);
        contains
    }
} 