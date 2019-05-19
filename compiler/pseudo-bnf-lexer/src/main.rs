use std::fs;

fn main() {
    let filename = "spec/torpel-grammar.pseudo-bnf";

    println!("Reading file {}", filename);

    let contents = fs::read_to_string(filename).expect("Could not read file");

    println!("File contents:\n{}", contents);
}
