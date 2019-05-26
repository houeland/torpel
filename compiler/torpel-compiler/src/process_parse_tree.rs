use super::generated_parser;

pub fn process_parse_tree(start: generated_parser::Start) {
    println!("{:#?}", start);
}
