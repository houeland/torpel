#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs;

pub mod generated_parser;
pub mod process_parse_tree;

fn main() {
    let source = fs::read_to_string("spec/test-example-structures.torpel")
        .expect("Could not open program");

    let mut input: Vec<&str> = source.split_whitespace().collect();
    input.push("<<EOF>>");

    let program = generated_parser::parse_start(&mut input);

    if input != ["<<EOF>>"] {
        println!("== ERROR! REMAINING PROGRAM TOKENS ==\n{:?}", input);
        panic!("Could not parse program");
    }

    process_parse_tree::process_parse_tree(program);
}
