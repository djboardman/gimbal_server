
extern crate pest;
use std::env;

#[macro_use]
extern crate pest_derive;

mod lang;
mod database;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ast = lang::ast_builder::build_from_main_file(&args[1]);
    println!("{:?}", ast);

}
