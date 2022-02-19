
extern crate pest;
extern crate yaml_rust;
extern crate postgres;
use std::env;
use std::fs;
use std::path::Path;
use std::io::prelude::*;

//use yaml_rust::{YamlLoader, YamlEmitter};
//use postgres::{Client, NoTls, Error};

#[macro_use]
extern crate pest_derive;

mod lang;
mod database;


fn main() {
    /* arg[1] is command
       compile
       migrate
    */
    let args: Vec<String> = env::args().collect();
    if args[1] == "compile" {
        println!("{}", compile(&args));
    } else if args[1] == "migrate" {
        println!("{}", migrate(&args));
    } else {
        println!("Error in command");
    }
}

fn compile(args: &Vec<String>) -> String {
    let ast = lang::ast_builder::build_from_main_file(&args[2]).unwrap();
    let config = database::meta::DatabaseConfig::Postgres("".to_string());
    let diffs = database::integration::diagnose_db_diffs(&ast, &config);
    let script = database::integration::diffs_to_script(&diffs, &config);
    let path = Path::new("changes.sql");
    let mut file = match fs::File::create(&path) {
      Err(m) => panic!("I couldn't save database migration script because: {}", m)
    , Ok(f) => f
    };

    match file.write_all(script.to_string().as_bytes()) {
      Err(m) => format!("I couldn't save database migration script because: {}", m)
    , Ok(_) => "Migration saved".to_string()
    }
}

fn migrate(args: &Vec<String>) -> String {
    let config = database::meta::DatabaseConfig::Postgres("".to_string());
    //let script = database::meta::DatabaseChange::SqlDb(vec![String::from("CREATE TABLE test__Person (first_name varchar(255), last_name varchar(255))")]);
    let script_str: Vec<String>= fs::read_to_string("changes.sql").unwrap().split("\n").map(|s| s.to_string()).collect();
    let script = database::meta::DatabaseChange::SqlDb(script_str);
    let errors = database::integration::migrate_db(&script, &config);
    match errors {
      Ok(_) => {
        "Migration completed".to_string()
      }
    , Err(s) => s
    }  
}