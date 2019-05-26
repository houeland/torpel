extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
pub enum RuleAction {
    ConsumeToken(String),
    RuleName(String),
    RepeatedRuleNameWithSeparator { rule_name: String, separator: String },
    Unknown(String),
}

impl fmt::Display for RuleAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuleAction::ConsumeToken(token) => write!(f, r#"ConsumeToken: "{}""#, token),
            RuleAction::RuleName(rule_name) => write!(f, "RuleName: {}", rule_name),
            RuleAction::RepeatedRuleNameWithSeparator { rule_name, separator }=> write!(
                f,
                r#"RepeatedRuleNameWithSeparator "{}"-separated sub-rule: {}"#,
                separator, rule_name
            ),
            RuleAction::Unknown(token) => write!(f, "Unknown: {}", token),
        }
    }
}

#[derive(Debug)]
pub enum RuleType {
    UserSpecifiedName,
    RuleChoice(Vec<String>),
    Actions(Vec<RuleAction>),
}

#[derive(Debug)]
pub struct Rule {
    pub rule_name: String,
    pub rule_type: RuleType,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rule: {}", self.rule_name)?;
        match &self.rule_type {
            RuleType::UserSpecifiedName => write!(f, " read user-specified-name\n"),
            RuleType::RuleChoice(sub_rule_names) => {
                write!(f, " one of the following sub-rules:\n")?;
                for r in sub_rule_names.iter() {
                    write!(f, "  {}\n", r)?;
                }
                Ok(())
            }
            RuleType::Actions(actions) => {
                write!(f, " action sequence:\n")?;
                for action in actions.iter() {
                    write!(f, "  {}\n", action)?;
                }
                Ok(())
            }
        }
    }
}

fn parse_grammar_rule(line: &str) -> Rule {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let rule_name = tokens[0].to_string();
    let right_hand = &tokens[2..];
    let string_pattern = Regex::new(r#"^"([^"]+)"$"#).unwrap();
    let rule_pattern = Regex::new(r#"^(<[A-Z-]+>)$"#).unwrap();
    let repeated_rule_pattern = Regex::new(r#"^(<[A-Z-]+>)\*"(.)"$"#).unwrap();

    if right_hand == ["<<USER-SPECIFIED-NAME>>"] {
        return Rule {
            rule_name,
            rule_type: RuleType::UserSpecifiedName,
        };
    } else if right_hand.iter().any(|&x| x == "|") {
        let sub_rule_names = right_hand
            .iter()
            .cloned()
            .filter(|&x| x != "|")
            .map(|x| x.to_string())
            .collect();
        return Rule {
            rule_name,
            rule_type: RuleType::RuleChoice(sub_rule_names),
        };
    }

    let mut actions = Vec::new();
    for &token in right_hand.iter() {
        if string_pattern.is_match(token) {
            let consume_token = string_pattern.captures(token).unwrap()[1].to_string();
            actions.push(RuleAction::ConsumeToken(consume_token));
        } else if rule_pattern.is_match(token) {
            let rule_name = rule_pattern.captures(token).unwrap()[1].to_string();
            actions.push(RuleAction::RuleName(rule_name));
        } else if repeated_rule_pattern.is_match(token) {
            let rule_name = repeated_rule_pattern.captures(token).unwrap()[1].to_string();
            let separator = repeated_rule_pattern.captures(token).unwrap()[2].to_string();
            actions.push(RuleAction::RepeatedRuleNameWithSeparator { rule_name, separator });
        } else {
            actions.push(RuleAction::Unknown(token.to_string()));
        }
    }
    return Rule {
        rule_name,
        rule_type: RuleType::Actions(actions),
    };
}

#[derive(Debug)]
pub struct Grammar {
    pub rules: HashMap<String, Rule>,
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

pub fn read_grammar_from_file(filename: &str) -> Grammar {
    // println!("Reading grammar from file {}", filename);
    let file = File::open(filename).expect("Could not read grammar");
    let reader = BufReader::new(&file);
    let mut rules = HashMap::new();
    for line in reader.lines() {
        let s = line.unwrap();
        let rule = parse_grammar_rule(&s);
        rules.insert(rule.rule_name.clone(), rule);
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
    match &rule.rule_type {
        RuleType::Actions(actions) => {
            let first_action = &actions[0];
            // println!(r#"{}try to match {}"#, indent, first_action);
            match first_action {
                RuleAction::ConsumeToken(consume_token) => consume_token == token,
                _ => {
                    println!(
                        r#"{}SYNTAX ERROR: check_if_can_start_rule at "{}""#,
                        indent, token
                    );
                    println!("{}reason: first action must be consuming a token", indent);
                    false
                }
            }
        }
        RuleType::UserSpecifiedName => {
            let user_specified_name_pattern = Regex::new(r#"^[A-Z][a-zA-Z-]*$"#).unwrap();
            user_specified_name_pattern.is_match(token)
        }
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

#[derive(Debug)]
enum DetailedActionProduction {
    Rules(Vec<DetailedRuleProduction>),
    Consume(String),
    Unknown,
}

#[derive(Debug)]
enum DetailedRuleProduction {
    RuleActionSequence {
        rule_name: String,
        actions: Vec<DetailedActionProduction>,
    },
    UserSpecifiedName {
        rule_name: String,
        user_specified_name: String,
    },
    Error,
}

fn run_action(
    grammar: &Grammar,
    action: &RuleAction,
    tokens: &mut Vec<&str>,
    indent: &str,
) -> DetailedActionProduction {
    let subindent = indent.to_owned() + "  ";
    match action {
        RuleAction::RepeatedRuleNameWithSeparator {rule_name, separator}=> {
            // println!("{}run_action {}", indent, action);
            let mut rules = vec![];
            let rp = run_rule(grammar, rule_name, tokens, &subindent);
            rules.push(rp);
            while tokens[0] == separator {
                consume(separator, tokens, &subindent);
                let matches =
                    check_if_can_start_rule(grammar, rule_name, tokens[0], &subindent);
                // println!("{} - {} - {}", subindent, action.rule_name, matches);
                if matches {
                    let rp = run_rule(grammar, rule_name, tokens, &subindent);
                    rules.push(rp);
                } else {
                    break;
                }
            }
            DetailedActionProduction::Rules(rules)
            // println!(r#"{}finished loop, next token is "{}""#, indent, tokens[0]);
        }
        RuleAction::RuleName(rule_name) => {
            // println!("{}run_action {}", indent, action);
            let rp = run_rule(grammar, rule_name, tokens, &subindent);
            DetailedActionProduction::Rules(vec![rp])
        }
        RuleAction::ConsumeToken(consume_token) => {
            // println!("{}run_action {}", indent, action);
            consume(consume_token, tokens, &subindent);
            DetailedActionProduction::Consume(consume_token.to_owned())
        }
        RuleAction::Unknown(_) => {
            println!("{}UNKNOWN ACTION! {}", indent, action);
            DetailedActionProduction::Unknown
        }
    }
}

fn run_rule(
    grammar: &Grammar,
    rule_name: &str,
    tokens: &mut Vec<&str>,
    indent: &str,
) -> DetailedRuleProduction {
    let subindent = indent.to_owned() + "  ";
    let rule = grammar.rules.get(rule_name).unwrap();
    match &rule.rule_type {
        RuleType::Actions(rule_actions) => {
            let mut actions = vec![];
            // println!("{}Running rule {} action sequence!", indent, rule.name);
            for action in rule_actions.iter() {
                let ap = run_action(grammar, action, tokens, &subindent);
                actions.push(ap);
            }
            // println!("{}Rule {} done", indent, rule.name);
            DetailedRuleProduction::RuleActionSequence {
                rule_name: rule.rule_name.to_owned(),
                actions,
            }
        }
        RuleType::UserSpecifiedName => {
            // println!(
            //     r#"{}Running rule {} read user specified name: "{}""#,
            //     indent, rule.name, tokens[0]
            // );
            let user_specified_name_pattern = Regex::new(r#"^[A-Z][a-zA-Z-]*$"#).unwrap();
            if user_specified_name_pattern.is_match(tokens[0]) {
                let name = tokens.remove(0);
                DetailedRuleProduction::UserSpecifiedName {
                    rule_name: rule.rule_name.to_owned(),
                    user_specified_name: name.to_owned(),
                }
            } else {
                println!(
                    r#"{}SYNTAX ERROR: invalid user-specified-name at "{}""#,
                    indent, tokens[0]
                );
                DetailedRuleProduction::Error
            }
            // println!(r#"{}read name: "{}""#, subindent, tokens[0]);
            // println!(r#"{}next is: "{}""#, subindent, tokens[0]);
        }
        RuleType::RuleChoice(sub_rule_names) => {
            // println!(
            //     "{}Running rule {} rule-choice options {:?}:",
            //     indent, rule.name, rule.sub_rule_names
            // );
            let mut count_matches = 0;
            let mut match_rule_name = "";
            for rn in sub_rule_names.iter() {
                let matches = check_if_can_start_rule(grammar, rn, tokens[0], &subindent);
                // println!("{} - {} - {}", subindent, rn, matches);
                if matches {
                    count_matches += 1;
                    match_rule_name = rn;
                }
            }
            if count_matches == 1 {
                // println!("{}matched rule {}!", indent, match_rule_name);
                run_rule(grammar, match_rule_name, tokens, &subindent)
            } else {
                println!("{}rule-choice syntax error", indent);
                println!(
                    "{}reason: must match exactly 1 sub-rule, but matched {}",
                    indent, count_matches
                );
                DetailedRuleProduction::Error
            }
        }
    }
}

pub enum RuleProduction {
    RuleActionSequence {
        rule_name: String,
        actions: Vec<RuleProduction>,
    },
    UserSpecifiedName {
        rule_name: String,
        user_specified_name: String,
    },
    Error,
}

impl fmt::Debug for RuleProduction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuleProduction::RuleActionSequence { rule_name, actions } => {
                write!(f, r#"RuleActionSequence {} "#, rule_name)?;
                f.debug_list().entries(actions.iter()).finish()?;
                Ok(())
            }
            RuleProduction::UserSpecifiedName {
                rule_name,
                user_specified_name,
            } => write!(
                f,
                "UserSpecifiedName {} = {:?}",
                rule_name, user_specified_name
            ),
            RuleProduction::Error => write!(f, "{:?}", self),
        }
    }
}

fn from_detailed(rule: DetailedRuleProduction) -> RuleProduction {
    match rule {
        DetailedRuleProduction::RuleActionSequence { rule_name, actions } => {
            let mut high_level_actions = vec![];
            for a in actions {
                match a {
                    DetailedActionProduction::Rules(drp) => {
                        for p in drp {
                            high_level_actions.push(from_detailed(p));
                        }
                    }
                    DetailedActionProduction::Consume(_) => {}
                    DetailedActionProduction::Unknown => {}
                }
            }
            RuleProduction::RuleActionSequence {
                rule_name,
                actions: high_level_actions,
            }
        }
        DetailedRuleProduction::UserSpecifiedName {
            rule_name,
            user_specified_name,
        } => RuleProduction::UserSpecifiedName {
            rule_name,
            user_specified_name,
        },
        DetailedRuleProduction::Error => RuleProduction::Error,
    }
}

pub fn run_grammar(grammar: &Grammar, tokens: &mut Vec<&str>) -> RuleProduction {
    let details = run_rule(grammar, "<<START>>", tokens, "");
    from_detailed(details)
}

pub fn parse_program(grammar: &Grammar, source: &str) {
    // let mut keys: Vec<&String> = grammar.rules.keys().collect();
    // keys.sort();
    // println!("== RULE LIST ==\n{:#?}\n", keys);
    let mut tokens: Vec<&str> = source.split_whitespace().collect();
    tokens.push("<<EOF>>");
    println!("== READING PROGRAM ==");
    let r = run_grammar(grammar, &mut tokens);
    if tokens == ["<<EOF>>"] {
        println!("== DONE! WELL-FORMED PROGRAM! ==");
        println!("{:#?}", r);
    } else {
        println!("== ERROR! REMAINING PROGRAM TOKENS ==\n{:?}", tokens);
    }
}
