#[macro_use] extern crate clap;
extern crate bincode;
extern crate rustc_serialize;
extern crate yaml_rust;
extern crate crypto;
extern crate rand;

pub mod domain;
pub mod store;
pub mod crypter;

use clap::{Arg, App, SubCommand};
use store::Store;
use domain::Record;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use yaml_rust::YamlLoader;
use rand::{ Rng, OsRng };

fn main() {
    let app = Box::new(App::new("mpass")
        .version("0.3")
        .about("Console password keeper")
        .author("Daniel Slapman <danslapman@gmail.com>")
        .subcommand(
            SubCommand::with_name("add")
            .arg(Arg::with_name("domain")
                .long("domain")
                .takes_value(true)
                .required(true)
                .index(1))
            .arg(Arg::with_name("username")
                .long("username")
                .takes_value(true)
                .required(true)
                .index(2))
            .arg(Arg::with_name("password")
                .long("password")
                .takes_value(true)
                .required(true)
                .index(3))
            .about("Add an entry to storage")
            .help("Creates a new entry in the storage with given data")
        )
        .subcommand(
            SubCommand::with_name("store")
            .arg(Arg::with_name("name")
                .long("name")
                .takes_value(true)
                .required(true)
                .index(1))
            .arg(Arg::with_name("command")
                .long("command")
                .takes_value(true)
                .required(true)
                .index(2))
            .about("Stores shell command into storage")
            .help("Creates new named command with given command line")
        )
        .subcommand(
            SubCommand::with_name("show")
            .arg(Arg::with_name("domain")
                .long("domain")
                .takes_value(true)
                .required(true)
                .index(1))
            .about("Display an entry by domain")
            .help("Displays an entry associated with given domain (if such entry exists)")
        )
        .subcommand(
            SubCommand::with_name("run")
            .arg(Arg::with_name("name")
                .long("name")
                .takes_value(true)
                .required(true)
                .index(1))
            .about("Runs command")
            .help("Runs command by given name")
        )
        .subcommand(
            SubCommand::with_name("drop")
            .arg(Arg::with_name("name")
                .long("name")
                .takes_value(true)
                .required(true)
                .index(1))
            .about("Remove an entry by domain/name")
            .help("Removes an entry associated with given domain/name (if such entry exists)")
        )
        .subcommand(
            SubCommand::with_name("domains")
            .about("Show domain list")
            .help("Shows domain list for all credentials in store")
        )
        .subcommand(
            SubCommand::with_name("commands")
            .about("Show stored commands")
            .help("Shows list of stored shell commands")
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
    
    let mut config_file_contents = String::new();
    let _ = File::open(mpass_dir.join("config.yml"))
        .map(|mut f| f.read_to_string(&mut config_file_contents));
   
    let store_file_path = YamlLoader::load_from_str(&config_file_contents)
        .map(|cfg| cfg[0]["store_location"].as_str().expect("Config has incorrect format").to_owned())
        .unwrap_or(mpass_dir.join("store.bin").to_str().unwrap_or("store.bin").to_owned());
        
    let store = Store { path: store_file_path, key: bin_key };
   
    let matches = app.clone().get_matches();

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
        _ => {
            let _ = app.clone().print_help();
            println!("");
        }
    }
}
