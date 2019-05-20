extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
struct Rule {
    name: String,
    rule_type: String,
    consume_token: String,
}

fn parse_rule(line: &str) -> Rule {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let name = tokens[0].to_string();
    let right_hand = &tokens[2..];
    if right_hand == ["<<USER-SPECIFIED-NAME>>"] {
        return Rule {
            name,
            rule_type: "user-specified-token".to_string(),
            consume_token: "".to_string(),
        };
    }
    let string_pattern = Regex::new(r#"^"([^"]+)"$"#).unwrap();
    let first = right_hand[0];
    if string_pattern.is_match(first) {
        return Rule {
            name,
            rule_type: "consume_token".to_string(),
            consume_token: string_pattern.captures(first).unwrap()[1].to_string(),
        };
    }
    // let rule_type = match start {
    //     start if start.is_whitespace() => {}
    // }
    let r = Rule {
        name,
        rule_type: "unknown".to_string(),
        consume_token: "".to_string(),
    };
    return r;
}

fn read_grammar_from_file(filename: &str) {
    println!("Reading file {}", filename);
    let file = File::open(filename).expect("Could not read file");
    let reader = BufReader::new(&file);
    for line in reader.lines() {
        let s = line.unwrap();
        let rule = parse_rule(&s);
        println!("rule: {:?}", rule);
    }
}

fn main() {
    let filename = "spec/torpel-grammar.pseudo-bnf";

    read_grammar_from_file(filename);
}
