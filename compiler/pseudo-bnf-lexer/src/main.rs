extern crate regex;

use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
struct RuleAction {
    action_type: String,
    consume_token: String,
    rule_name: String,
    unknown_token: String,
}

impl fmt::Display for RuleAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.action_type == "consume_token" {
            write!(f, r#"consume: "{}""#, self.consume_token)
        } else if self.action_type == "unimplemented" {
            write!(f, "unimplemented: {}", self.unknown_token)
        } else if self.action_type == "rule_name" {
            write!(f, "sub-rule: {}", self.rule_name)
        } else if self.action_type == "repeated rule_name with separator" {
            write!(f, r#"repeated "{}"-separated sub-rule: {}"#, self.consume_token, self.rule_name)
        } else {
            write!(f, "unknown: {}", self.unknown_token)
        }
    }
}

#[derive(Debug)]
struct Rule {
    name: String,
    rule_type: String,
    actions: Vec<RuleAction>,
    sub_rules: Vec<String>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rule: {}", self.name)?;
        if self.rule_type == "user-specified-token" {
            write!(f, " read user-specified-token\n")
        } else if self.rule_type == "rule-choice" {
            write!(f, " one of the following sub-rules:\n")?;
            for r in self.sub_rules.iter() {
                write!(f, "  {}\n", r)?;
            }
            Ok(())
        } else if self.rule_type == "actions" {
            write!(f, " action sequence:\n")?;
            for action in self.actions.iter() {
                write!(f, "  {}\n", action)?;
            }
            Ok(())
        } else {
            write!(f, " not recognized!\n")
        }
    }
}

fn parse_rule(line: &str) -> Rule {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let name = tokens[0].to_string();
    let right_hand = &tokens[2..];
    let string_pattern = Regex::new(r#"^"([^"]+)"$"#).unwrap();
    let rule_pattern = Regex::new(r#"^(<[A-Z-]+>)$"#).unwrap();
    let repeated_rule_pattern = Regex::new(r#"^(<[A-Z-]+>)\*"(.)"$"#).unwrap();

    if right_hand == ["<<USER-SPECIFIED-NAME>>"] {
        return Rule {
            name,
            rule_type: "user-specified-token".to_string(),
            actions: vec![],
            sub_rules: vec![],
        };
    } else if right_hand.iter().any(|&x| x == "|") {
        return Rule {
            name,
            rule_type: "rule-choice".to_string(),
            actions: vec![],
            sub_rules: right_hand.iter().cloned().filter(|&x| x != "|").map(|x| x.to_string()).collect(),
        };
    }

    let mut actions = Vec::new();
    for &token in right_hand.iter() {
        if string_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: "consume_token".to_string(),
                consume_token: string_pattern.captures(token).unwrap()[1].to_string(),
                rule_name: "".to_string(),
                unknown_token: "".to_string(),
            });
        } else if rule_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: "rule_name".to_string(),
                consume_token: "".to_string(),
                rule_name: rule_pattern.captures(token).unwrap()[1].to_string(),
                unknown_token: "".to_string(),
            });
        } else if repeated_rule_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: "repeated rule_name with separator".to_string(),
                consume_token: repeated_rule_pattern.captures(token).unwrap()[2].to_string(),
                rule_name: repeated_rule_pattern.captures(token).unwrap()[1].to_string(),
                unknown_token: "".to_string(),
            });
        } else {
            actions.push(RuleAction {
                action_type: "unknown".to_string(),
                consume_token: "".to_string(),
                rule_name: "".to_string(),
                unknown_token: token.to_string(),
            });
        }
    }
    let r = Rule {
        name,
        rule_type: "actions".to_string(),
        actions,
        sub_rules: vec![],
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
        println!("{}", rule);
    }
}

fn main() {
    let filename = "spec/torpel-grammar.pseudo-bnf";

    read_grammar_from_file(filename);
}
