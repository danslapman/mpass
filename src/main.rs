#[macro_use] extern crate clap;
#[macro_use] extern crate mdo;
#[macro_use] extern crate serde_derive;
extern crate bincode;
extern crate rustc_serialize;
extern crate yaml_rust;
extern crate crypto;
extern crate rand;
extern crate serde_json;

pub mod domain;
pub mod store;
pub mod crypter;

use store::Store;
use domain::Record;
use mdo::result::{bind, ret};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{self, Read, Write};
use std::process::{Command, exit};
use yaml_rust::YamlLoader;
use rand::{ Rng, OsRng };
use rustc_serialize::base64::*;

fn main() {
    let mpass_app = clap_app!(mpass_app =>
        (version: "0.3")
        (author: "Daniel Slapman <danslapman@gmail.com>")
        (about: "Console password keeper")
        (@subcommand add =>
            (about: "Add an entry to storage")
            (help: "Creates a new entry in the storage with given data")
            (@arg domain: +required +takes_value)
            (@arg username: +required +takes_value)
            (@arg password: +required +takes_value)
        )
        (@subcommand store =>
            (about: "Stores shell command")
            (help: "Creates new named command with given command line")
            (@arg name: +required +takes_value)
            (@arg command: +required +takes_value)
        )
        (@subcommand show =>
            (about: "Display an entry by domain")
            (help: "Displays an entry associated with given domain (if such entry exists)")
            (@arg domain: +required +takes_value)
        )
        (@subcommand run =>
            (about: "Runs command")
            (help: "Runs command by given name")
            (@arg name: +required +takes_value)
        )
        (@subcommand drop =>
            (about: "Remove an entry by domain/name")
            (help: "Removes an entry associated with given domain/name (if such entry exists)")
            (@arg name: +required +takes_value)
        )
        (@subcommand domains =>
            (about: "Show domain list")
            (help: "Shows domain list for all credentials in store")
        )
        (@subcommand commands =>
            (about: "Show stored commands")
            (help: "Shows list of stored shell commands")
        )
        (@subcommand export =>
            (about: "Export data")
            (help: "Writes all the data into given file in JSON format")
            (@arg file: +required +takes_value conflicts_with[key])
            (@arg key: -k --key "Export encryption key")
        )
        (@subcommand import =>
            (about: "Import data")
            (help: "Import data from JSON file")
            (@arg file: +required +takes_value conflicts_with[key])
            (@arg key: -k --key "Import encryption key")
        )
    );

    let home_dir = std::env::home_dir().expect("Impossible to get your home dir!");
    let mpass_dir = home_dir.join(".mpass");

    let mut bin_key = Vec::<u8>::new();
    let _ = File::open(mpass_dir.join("key.bin")).map(|mut f| f.read_to_end(&mut bin_key));
    if bin_key.len() == 0 {
        let mut rnd_key: [u8; 32] = [0; 32];
        let mut rng = OsRng::new().ok().unwrap();
        rng.fill_bytes(&mut rnd_key);
        bin_key = Vec::from(&rnd_key[..]);
        let _ = File::create(mpass_dir.join("key.bin")).unwrap()
            .write(&rnd_key[..]).unwrap();
    }

    let config_path = mpass_dir.join("config.yml");

    if !Path::new(&config_path).exists() {
        println!("Please, type path to the store (default is {{home}}/.mpass/store.bin)");
        println!("(New store will be created if file does not exist):");
        let mut buffer = String::new();
        let creation_process = mdo! {
            _ =<< io::stdin().read_line(&mut buffer);
            let trimmed_buffer = buffer.trim();
            let store_path = if trimmed_buffer.is_empty() {
                    home_dir.join(".mpass").join("store.bin").to_string_lossy().into_owned()
                } else {
                    trimmed_buffer.to_owned()
                };
            let line = format!("store_location: {}", store_path);
            mut f =<< File::create(config_path);
            _ =<< f.write_all(line.as_bytes());
            ret ret(())
        };
        creation_process.expect("Fatal error during configuration file creation!");
    }
    
    let mut config_file_contents = String::new();
    let _ = File::open(mpass_dir.join("config.yml"))
        .map(|mut f| f.read_to_string(&mut config_file_contents));
   
    let store_file_path = YamlLoader::load_from_str(&config_file_contents)
        .map(|cfg| cfg[0]["store_location"].as_str().expect("Config has incorrect format").to_owned())
        .unwrap_or(mpass_dir.join("store.bin").to_str().unwrap_or("store.bin").to_owned());

    if Path::new(&store_file_path).is_dir() {
        println!("`store_location` must be file, not directory!");
        exit(1)
    }
        
    let store = Store { path: store_file_path, key: bin_key.clone() };
   
    let matches = mpass_app.clone().get_matches();

    match matches.subcommand_name() {
        Some("add") => {
            let sm = matches.subcommand_matches("add").unwrap();
            let domain = value_t!(sm, "domain", String).expect("Domain");
            let username = value_t!(sm, "username", String).expect("User name");
            let password = value_t!(sm, "password", String).expect("Password");
            let entry = Record::Credentials { domain: domain, username: username, password: password };
            match store.persist(entry) {
                false => println!("An item associated with this name already exist"),
                _ => ()
            }
        },
        Some("store") => {
            let sm = matches.subcommand_matches("store").unwrap();
            let name = value_t!(sm, "name", String).expect("Name");
            let cmd = value_t!(sm, "command", String).expect("Command");
            let entry = Record::Command { name: name, command_line: cmd };
            match store.persist(entry) {
                false => println!("An item associated with this name already exist"),
                _ => ()
            }
        },
        Some("show") => {
            let sm = matches.subcommand_matches("show").unwrap();
            let domain = value_t!(sm, "domain", String).expect("Domain");
            match store.read_credentials(domain) {
                Some(Record::Credentials {domain: d, username: u, password: p}) => {
                    println!("Credentials for {}:", d);
                    println!("Username: {}", u);
                    println!("Password: {}", p);
                },
                _ => println!("There is no credentials associated with this domain")
            }
        },
        Some("run") => {
            let sm = matches.subcommand_matches("run").unwrap();
            let name = value_t!(sm, "name", String).expect("Name");
            match store.read_cmd(name) {
                Some(Record::Command {name: n, command_line: cmd}) => {
                    println!("Executing {}", n);
                    let parts = cmd.split(" ").collect::<Vec<&str>>();

                    let mut command = Command::new(parts[0]);
                    for i in 1 .. parts.len() {
                        command.arg(parts[i]);
                    }

                    command.status().expect("command failed to start");
                },
                _ => println!("There is no command associated with given name")
            }
        },
        Some("drop") => {
            let sm = matches.subcommand_matches("drop").unwrap();
            let domain = value_t!(sm, "name", String).expect("Domain/name");
            match store.remove(domain.clone()) {
                true => println!("Item named '{}' deleted", domain),
                false => println!("There is no item associated with this name")
            }
        },
        Some("domains") => {
            for domain in store.list_domains() {
                println!("{}", domain);
            }
        },
        Some("commands") => {
            for command in store.list_commands() {
                println!("{}", command);
            }
        },
        Some("export") => {
            let sm = matches.subcommand_matches("export").unwrap();
            if sm.is_present("key") {
                println!("Your encryption key: {}", bin_key.to_base64(STANDARD));
            } else {
                let file = value_t!(sm, "file", String).expect("File");

                let json = serde_json::to_string_pretty(&store.export_all_items())
                    .expect("Fatal error during serialization");

                let write_res = mdo! {
                    mut f =<< File::create(file);
                    res =<< f.write_all(json.as_bytes());
                    ret ret(res)
                };
                write_res.expect("Unrecoverable error!");
            }
        },
        Some("import") => {
            let sm = matches.subcommand_matches("import").unwrap();
            match value_t!(sm, "key", String).ok() {
                Some(key) => {
                    if store.export_all_items().len() > 0 {
                        println!("Can't import key: store is not empty!");
                        exit(1)
                    }

                    let new_key = key.from_base64()
                        .expect("Provided key is not valid base64 string!");

                    if new_key.len() != 32 {
                        println!("You must provide 256-bit key");
                        exit(1)
                    }

                    let import_proc = mdo! {
                        mut f =<< OpenOptions::new().write(true).open(mpass_dir.join("key.bin"));
                        _ =<< f.write(&new_key[..]);
                        ret ret(())
                    };

                    import_proc.expect("Key import failed");
                },
                None => {
                    let file = value_t!(sm, "file", String).expect("File");

                    let mut read_buf = String::new();
                    let str_data = mdo! {
                        mut f =<< File::open(file);
                        _ =<< f.read_to_string(&mut read_buf);
                        ret ret(read_buf)
                    }.expect("Can't read provided file");

                    let import_data: Vec<Record> =
                    serde_json::from_str(&str_data).expect("Deserialization problem");

                    let store_res = import_data.into_iter()
                        .map(|record| store.persist(record))
                        .all(|flag| flag);

                    if store_res {
                        println!("Data imported sucessfully");
                    } else {
                        println!("Some records duplicate existing entries and were not imported");
                    }
                }
            }
        },
        _ => {
            let _ = mpass_app.clone().print_help();
            println!();
        }
    }
}
