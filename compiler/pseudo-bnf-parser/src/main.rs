extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
enum RuleActionType {
    ConsumeToken,
    RuleName,
    RepeatedRuleNameWithSeparator,
    Unknown,
}

#[derive(Debug)]
struct RuleAction {
    action_type: RuleActionType,
    consume_token: String,
    rule_name: String,
    unknown_token: String,
}

impl fmt::Display for RuleAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.action_type {
            RuleActionType::ConsumeToken => write!(f, r#"consume: "{}""#, self.consume_token),
            RuleActionType::RuleName => write!(f, "sub-rule: {}", self.rule_name),
            RuleActionType::RepeatedRuleNameWithSeparator => write!(
                f,
                r#"repeated "{}"-separated sub-rule: {}"#,
                self.consume_token, self.rule_name
            ),
            RuleActionType::Unknown => write!(f, "unknown: {}", self.unknown_token),
        }
    }
}

#[derive(Debug)]
enum RuleType {
    UserSpecifiedName,
    RuleChoice,
    Actions,
}

#[derive(Debug)]
struct Rule {
    name: String,
    rule_type: RuleType,
    actions: Vec<RuleAction>,
    sub_rule_names: Vec<String>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rule: {}", self.name)?;
        match self.rule_type {
            RuleType::UserSpecifiedName => write!(f, " read user-specified-name\n"),
            RuleType::RuleChoice => {
                write!(f, " one of the following sub-rules:\n")?;
                for r in self.sub_rule_names.iter() {
                    write!(f, "  {}\n", r)?;
                }
                Ok(())
            },
            RuleType::Actions => {
                write!(f, " action sequence:\n")?;
                for action in self.actions.iter() {
                    write!(f, "  {}\n", action)?;
                }
                Ok(())
            },
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
            rule_type: RuleType::UserSpecifiedName,
            actions: vec![],
            sub_rule_names: vec![],
        };
    } else if right_hand.iter().any(|&x| x == "|") {
        return Rule {
            name,
            rule_type: RuleType::RuleChoice,
            actions: vec![],
            sub_rule_names: right_hand
                .iter()
                .cloned()
                .filter(|&x| x != "|")
                .map(|x| x.to_string())
                .collect(),
        };
    }

    let mut actions = Vec::new();
    for &token in right_hand.iter() {
        if string_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: RuleActionType::ConsumeToken,
                consume_token: string_pattern.captures(token).unwrap()[1].to_string(),
                rule_name: "".to_string(),
                unknown_token: "".to_string(),
            });
        } else if rule_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: RuleActionType::RuleName,
                consume_token: "".to_string(),
                rule_name: rule_pattern.captures(token).unwrap()[1].to_string(),
                unknown_token: "".to_string(),
            });
        } else if repeated_rule_pattern.is_match(token) {
            actions.push(RuleAction {
                action_type: RuleActionType::RepeatedRuleNameWithSeparator,
                consume_token: repeated_rule_pattern.captures(token).unwrap()[2].to_string(),
                rule_name: repeated_rule_pattern.captures(token).unwrap()[1].to_string(),
                unknown_token: "".to_string(),
            });
        } else {
            actions.push(RuleAction {
                action_type: RuleActionType::Unknown,
                consume_token: "".to_string(),
                rule_name: "".to_string(),
                unknown_token: token.to_string(),
            });
        }
    }
    let r = Rule {
        name,
        rule_type: RuleType::Actions,
        actions,
        sub_rule_names: vec![],
    };
    return r;
}

#[derive(Debug)]
struct Grammar {
    rules: HashMap<String, Rule>,
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut keys: Vec<&String> = self.rules.keys().collect();
        keys.sort();
        for k in keys.iter() {
            write!(f, "{}", self.rules.get(*k).unwrap())?;
        }
        Ok(())
    }
}

fn read_grammar_from_file(filename: &str) -> Grammar {
    println!("Reading grammar from file {}", filename);
    let file = File::open(filename).expect("Could not read grammar");
    let reader = BufReader::new(&file);
    let mut rules = HashMap::new();
    for line in reader.lines() {
        let s = line.unwrap();
        let rule = parse_rule(&s);
        rules.insert(rule.name.clone(), rule);
    }
    Grammar { rules }
}

fn consume(expected: &str, tokens: &mut Vec<&str>, indent: &str) {
    if tokens[0] == expected {
        // println!(r#"{}consumed "{}"!"#, indent, tokens[0]);
        tokens.remove(0);
    // println!(r#"{}next is "{}"!"#, indent, tokens[0]);
    } else {
        println!(
            "{}SYNTAX ERROR: expected {} but found {}!",
            indent, expected, tokens[0]
        );
    }
}

fn check_if_can_start_rule(grammar: &Grammar, rule_name: &str, token: &str, indent: &str) -> bool {
    let rule = grammar.rules.get(rule_name).unwrap();
    match rule.rule_type {
        RuleType::Actions => {
            let first_action = &rule.actions[0];
            // println!(r#"{}try to match {}"#, indent, first_action);
            match first_action.action_type {
                RuleActionType::ConsumeToken => first_action.consume_token == token,
                _ => {
                    println!(
                        r#"{}SYNTAX ERROR: check_if_can_start_rule at "{}""#,
                        indent, token
                    );
                    println!("{}reason: first action must be consuming a token", indent);
                    false
                }
            }
        },
        RuleType::UserSpecifiedName => {
            let user_specified_name_pattern = Regex::new(r#"^[A-Z][a-zA-Z-]*$"#).unwrap();
            user_specified_name_pattern.is_match(token)
        },
        _ => {
            println!(
                r#"{}SYNTAX ERROR: check_if_can_start_rule at "{}""#,
                indent, token
            );
            println!(
                "{}reason: only action sequences and user-specified-names are supported",
                indent
            );
            false
        }
    }
}

fn run_action(grammar: &Grammar, action: &RuleAction, tokens: &mut Vec<&str>, indent: &str) {
    let subindent = indent.to_owned() + "  ";
    match action.action_type {
        RuleActionType::RepeatedRuleNameWithSeparator => {
            println!("{}run_action {}", indent, action);
            run_rule(grammar, &action.rule_name, tokens, &subindent);
            while tokens[0] == action.consume_token {
                consume(&action.consume_token, tokens, &subindent);
                let matches =
                    check_if_can_start_rule(grammar, &action.rule_name, tokens[0], &subindent);
                // println!("{} - {} - {}", subindent, action.rule_name, matches);
                if matches {
                    run_rule(grammar, &action.rule_name, tokens, &subindent);
                } else {
                    break;
                }
            }
            // println!(r#"{}finished loop, next token is "{}""#, indent, tokens[0]);
        },
        RuleActionType::RuleName => {
            println!("{}run_action {}", indent, action);
            run_rule(grammar, &action.rule_name, tokens, &subindent);
        },
        RuleActionType::ConsumeToken => {
            println!("{}run_action {}", indent, action);
            consume(&action.consume_token, tokens, &subindent);
        },
        RuleActionType::Unknown => {
            println!("{}UNKNOWN ACTION! {}", indent, action);
        },
    }
}

fn run_rule(grammar: &Grammar, rule_name: &str, tokens: &mut Vec<&str>, indent: &str) {
    let subindent = indent.to_owned() + "  ";
    let rule = grammar.rules.get(rule_name).unwrap();
    match rule.rule_type {
        RuleType::Actions => {
            println!("{}Running rule {} action sequence!", indent, rule.name);
            for action in rule.actions.iter() {
                run_action(grammar, action, tokens, &subindent);
            }
            println!("{}Rule {} done", indent, rule.name);
        },
        RuleType::UserSpecifiedName => {
            println!(
                r#"{}Running rule {} read user specified name: "{}""#,
                indent, rule.name, tokens[0]
            );
            let user_specified_name_pattern = Regex::new(r#"^[A-Z][a-zA-Z-]*$"#).unwrap();
            if user_specified_name_pattern.is_match(tokens[0]) {
                tokens.remove(0);
            } else {
                println!(
                    r#"{}SYNTAX ERROR: invalid user-specified-name at "{}""#,
                    indent, tokens[0]
                );
            }
            // println!(r#"{}read name: "{}""#, subindent, tokens[0]);
            // println!(r#"{}next is: "{}""#, subindent, tokens[0]);
        },
        RuleType::RuleChoice => {
            println!(
                "{}Running rule {} rule-choice options {:?}:",
                indent, rule.name, rule.sub_rule_names
            );
            let mut count_matches = 0;
            let mut match_rule_name = "";
            for rn in rule.sub_rule_names.iter() {
                let matches = check_if_can_start_rule(grammar, rn, tokens[0], &subindent);
                // println!("{} - {} - {}", subindent, rn, matches);
                if matches {
                    count_matches += 1;
                    match_rule_name = rn;
                }
            }
            if count_matches == 1 {
                println!("{}matched rule {}!", indent, match_rule_name);
                run_rule(grammar, match_rule_name, tokens, &subindent);
            } else {
                println!("{}rule-choice syntax error", indent);
                println!(
                    "{}reason: must match exactly 1 sub-rule, but matched {}",
                    indent, count_matches
                );
            }
        },
    }
}

fn parse_program(grammar: &Grammar, source: &str) {
    // let mut keys: Vec<&String> = grammar.rules.keys().collect();
    // keys.sort();
    // println!("== RULE LIST ==\n{:#?}\n", keys);
    let mut tokens: Vec<&str> = source.split_whitespace().collect();
    tokens.push("<<EOF>>");
    println!("== READING PROGRAM ==");
    run_rule(grammar, "<<START>>", &mut tokens, "");
    if tokens == ["<<EOF>>"] {
        println!("== DONE! WELL-FORMED PROGRAM! ==");
    } else {
        println!("== ERROR! REMAINING PROGRAM TOKENS ==\n{:?}", tokens);
    }
}

fn main() {
    let grammar = read_grammar_from_file("spec/torpel-grammar.pseudo-bnf");
    println!("== GRAMMAR ==\n{}", grammar);

    let program =
        fs::read_to_string("spec/test-example-structures.torpel").expect("Could not open program");

    parse_program(&grammar, &program);
}
