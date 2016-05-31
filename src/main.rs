#[macro_use] extern crate clap;
extern crate bincode;
extern crate rustc_serialize;
extern crate yaml_rust;

pub mod domain;
pub mod store;

use clap::{Arg, App, SubCommand};
use store::Store;
use domain::RecordCell;
use std::fs::File;
use std::io::Read;
use yaml_rust::YamlLoader;

fn main() {
    let app = Box::new(App::new("mpass")
        .version("0.1")
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
            SubCommand::with_name("show")
            .arg(Arg::with_name("domain")
                .long("domain")
                .takes_value(true)
                .required(true)
                .index(1))
            .about("Display an entry by domain")
            .help("Displays an entry associated with given domain (if such entry exists)")
    ));
    
    let home_dir = std::env::home_dir().expect("Impossible to get your home dir!");
    let mpass_dir = home_dir.join(".mpass");
    
    let mut config_file_contents = String::new();
    let _ = File::open(mpass_dir.join("config.yml"))
        .map(|mut f| f.read_to_string(&mut config_file_contents))
        .expect("Configuration file is unreadable or does not exist");
    
    let config = &YamlLoader::load_from_str(&config_file_contents).expect("Config file has invalid format")[0];
    
    let store = Store { path: config["store_location"].as_str().expect("store_location has invalid format").to_owned() };
   
    let matches = app.clone().get_matches(); 
        
    match matches.subcommand_name() {
        Some("add") => {
            let sm = matches.subcommand_matches("add").unwrap();
            let domain = value_t!(sm, "domain", String).expect("Domain");
            let username = value_t!(sm, "username", String).expect("User name");
            let password = value_t!(sm, "password", String).expect("Password");
            let entry = RecordCell { domain: domain, username: username, password: password };
            store.persist(entry);
            ()
        },
        Some("show") => {
            let sm = matches.subcommand_matches("show").unwrap();
            let domain = value_t!(sm, "domain", String).expect("Domain");
            match store.read(domain) {
                None => println!("There is no credentials associated with this domain"),
                Some(cr) => {
                    println!("Credentials for {}:", cr.domain);
                    println!("Username: {}", cr.username);
                    println!("Password: {}", cr.password);
                }
            }
        },
        _ => {
            let _ = app.clone().print_help();
            ()
        }
    }
}
