#[macro_use] extern crate clap;
extern crate bincode;
extern crate rustc_serialize;

pub mod domain;
pub mod store;

use clap::{Arg, App, SubCommand};
use store::Store;
use domain::RecordCell;

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
    
    let store = Store { path: "store.bin".to_owned() };
   
    let matches = app.clone().get_matches(); 
        
    match matches.subcommand_name() {
        Some("add") => {
            let sm = matches.subcommand_matches("add").unwrap();
            let domain = value_t!(sm, "domain", String).expect("Domain");
            let username = value_t!(sm, "username", String).expect("User name");
            let password = value_t!(sm, "password", String).expect("Password");
            let entry = RecordCell { domain: domain, username: username, password: password };
            store.persist(&entry);
            ()
        },
        Some("show") =>
            println!("{}", store.read()),
        _ => {
            let _ = app.clone().print_help();
            ()
        }
    }
}
