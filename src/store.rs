use std::fs::File;
use std::io::{Read, Write};

use domain::Record;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use crypter::{encrypt, decrypt};
use rand::{ Rng, OsRng };

pub struct Store {
    pub path: String,
    pub key: Vec<u8>
}

impl Store {
    fn read_all(&self) -> Vec<Record> {
        let mut contents = Vec::<u8>::new();
        match File::open(self.path.clone()) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut contents);
                let (iv, data) = contents.split_at(16);
                let decrypted_data = decrypt(data, self.key.as_slice(), iv).expect("Error while decrypting");
                let entries: Vec<Record> = decode(decrypted_data.as_slice()).expect("Error while decoding");
                entries
            },
            Err(_) => Vec::<Record>::new()
        }
        
    }
    
    fn write_all(&self, entries: Vec<Record>) -> () {
        let encoded_entries = encode(&entries, SizeLimit::Infinite).expect("Error while encoding");
        
        let mut iv: [u8; 16] = [0; 16];
        let mut rng = OsRng::new().ok().unwrap();
        rng.fill_bytes(&mut iv);
        
        let mut encrypted_entries = encrypt(encoded_entries.as_slice(), self.key.as_slice(), &iv)
            .expect("Error while encrypting");
        let mut data = Vec::from(&iv[..]);
        data.append(&mut encrypted_entries);
        
        let _ = File::create(self.path.clone()).expect("Error while creating file")
            .write(data.as_slice()).expect("Error while writing file");
        ()
    }

    pub fn list_domains(&self) -> Vec<String> {
        let entries = self.read_all();
        entries.into_iter().filter_map(|e| match e {
            Record::Credentials {domain: d, ..} => Some(d),
            _ => None
        }).collect::<Vec<_>>()
    }

    pub fn list_commands(&self) -> Vec<String> {
        let entries = self.read_all();
        entries.into_iter().filter_map(|e| match e {
            Record::Command { name: n, ..} => Some(n),
            _ => None
        }).collect::<Vec<_>>()
    }
    
    pub fn persist(&self, entry: Record) -> bool {
        let mut entries = self.read_all();
        let contains = entries.clone()
            .into_iter()
            .filter(|e| e.is_same_case(&entry))
            .map(|e| e.get_name()).any(|el| el == entry.get_name());
        if contains {
            false
        } else {
            entries.push(entry);
            self.write_all(entries);
            true
        }
    }

    pub fn read_credentials(&self, domain: String) -> Option<Record> {
        self.read_all().into_iter().fold(None, move |acc, el| {
            match el {
                Record::Credentials { domain: ref dm, ..} if dm.clone() == domain => Some(el.clone()),
                _ => acc
            }
        })
    }

    pub fn read_cmd(&self, name: String) -> Option<Record> {
        self.read_all().into_iter().fold(None, move |acc, el| {
            match el {
                Record::Command { name: ref nm, ..} if nm.clone() == name => Some(el.clone()),
                _ => acc
            }
        })
    }

    pub fn remove(&self, name: String) -> bool {
        let mut entries = self.read_all();
        let contains = entries.clone()
            .into_iter()
            .map(|e| e.get_name()).any(|el| el == name);
        entries.retain(move |el| el.get_name() != name);
        self.write_all(entries);
        contains
    }

    pub fn export_all_items(&self) -> Vec<Record> {
        self.read_all()
    }
} 