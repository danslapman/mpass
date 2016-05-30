#[macro_use] extern crate clap;
extern crate bincode;
extern crate rustc_serialize;

pub mod domain;

use clap::{Arg, App, SubCommand};

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
   
    let matches = app.clone().get_matches(); 
        
    match matches.subcommand_name() {
        Some("add") =>
            println!("Adding entry..."),
        Some("show") =>
            println!("Showing entry..."),
        _ => {
            let _ = app.clone().print_help();
            ()
        }
    }
}
