extern crate inflector;

mod grammar;

use inflector::Inflector;
use std::collections::HashSet;
use std::fs;

fn to_type_name(n: &str) -> String {
    format!("{}", n).to_snake_case().to_class_case()
}

fn to_field_name(n: &str) -> String {
    format!("{}", n).to_snake_case()
}

fn to_parser_name(n: &str) -> String {
    format!("parse {}", n).to_snake_case()
}

fn rust_prelude() {
    println!("use regex::Regex;");
    println!();
    println!("fn torpel_consume_token(token: &str, input: &mut Vec<&str>) {{");
    println!("  if input[0] == token {{");
    println!("    input.remove(0).to_string();");
    println!("  }} else {{");
    let panic_message = format!(r#"unexpected token: expected "{{}}" at {{:?}}"#);
    println!("    panic!({:?}, token, input)", panic_message);
    println!("  }}");
    println!("}}");
    println!();
    println!("fn torpel_is_valid_user_specified_name(token: &str) -> bool {{");
    println!("  lazy_static! {{");
    println!(r#"    static ref RE: Regex = Regex::new("^[A-Z][a-zA-Z-]*$").unwrap();"#);
    println!("  }}");
    println!("  return RE.is_match(token);");
    println!("}}");
    println!();
    println!("fn torpel_read_user_specified_name(input: &mut Vec<&str>) -> String {{");
    println!("  if torpel_is_valid_user_specified_name(input[0]) {{");
    println!("    return input.remove(0).to_string();");
    println!("  }} else {{");
    let panic_message = format!(r#"unexpected token: expected user specified name at {{:?}}"#);
    println!("    panic!({:?}, input)", panic_message);
    println!("  }}");
    println!("}}");
}

fn grammar_to_rust_types(grammar: &grammar::Grammar) {
    let mut keys: Vec<&String> = grammar.rules.keys().collect();
    keys.sort();
    // println!("== RULE LIST ==\n{:#?}\n", keys);
    for k in keys {
        let rule = &grammar.rules[k];
        let type_name = to_type_name(&rule.rule_name);
        match &rule.rule_type {
            grammar::RuleType::UserSpecifiedName => {
                println!();
                println!("pub type {} = String;", type_name);
            }
            grammar::RuleType::Actions(actions) => {
                println!();
                println!("#[derive(Debug)]");
                println!("pub struct {} {{", type_name);
                for a in actions {
                    match a {
                        grammar::RuleAction::ConsumeToken(_) => {}
                        grammar::RuleAction::RepeatedRuleNameWithSeparator {
                            rule_name,
                            separator: _,
                        } => {
                            println!(
                                "  {}: Vec<{}>,",
                                to_field_name(rule_name),
                                to_type_name(rule_name)
                            );
                        }
                        grammar::RuleAction::RuleName(rule_name) => {
                            println!(
                                "  {}: {},",
                                to_field_name(rule_name),
                                to_type_name(rule_name)
                            );
                        }
                        grammar::RuleAction::Unknown(_) => panic!("unknown RuleAction"),
                    }
                }
                println!("}}");
            }
            grammar::RuleType::RuleChoice(sub_rule_names) => {
                println!();
                println!("#[derive(Debug)]");
                println!("pub enum {} {{", type_name);
                for r in sub_rule_names {
                    let name = r.to_snake_case().to_class_case();
                    println!("  {}({}),", name, name);
                }
                println!("}}");
            }
        }
    }
}

fn generate_check_if_can_start_rule(grammar: &grammar::Grammar, rule_name: &str) -> String {
    let rule = grammar.rules.get(rule_name).unwrap();
    match &rule.rule_type {
        grammar::RuleType::Actions(actions) => {
            let first_action = &actions[0];
            match first_action {
                grammar::RuleAction::ConsumeToken(consume_token) => {
                    format!("input[0] == {:?}", consume_token)
                }
                _ => panic!("generate_check_if_can_start_rule first_action must be consume_token"),
            }
        }
        grammar::RuleType::UserSpecifiedName => {
            "torpel_is_valid_user_specified_name(input[0])".to_string()
        }
        _ => panic!("generate_check_if_can_start_rule must be Actions or UserSpecifiedName"),
    }
}

fn grammar_to_rust_parsers(grammar: &grammar::Grammar) {
    let mut keys: Vec<&String> = grammar.rules.keys().collect();
    keys.sort();
    // println!("== RULE LIST ==\n{:#?}\n", keys);
    for k in keys {
        let rule = &grammar.rules[k];
        let type_name = to_type_name(&rule.rule_name);
        let parse_function_name = to_parser_name(&rule.rule_name);
        println!();
        println!(
            "pub fn {}(input: &mut Vec<&str>) -> {} {{",
            parse_function_name, type_name
        );
        match &rule.rule_type {
            grammar::RuleType::UserSpecifiedName => {
                println!("  return torpel_read_user_specified_name(input);");
            }
            grammar::RuleType::Actions(actions) => {
                let mut fields = vec![];
                for a in actions {
                    match a {
                        grammar::RuleAction::ConsumeToken(token) => {
                            println!("  torpel_consume_token({:?}, input);", token);
                        }
                        grammar::RuleAction::RepeatedRuleNameWithSeparator {
                            rule_name,
                            separator,
                        } => {
                            let field_name = to_field_name(rule_name);
                            let parser_name = to_parser_name(rule_name);
                            let condition = generate_check_if_can_start_rule(grammar, rule_name);
                            println!("  let mut {} = vec![];", field_name);
                            println!("  {}.push({}(input));", field_name, parser_name);
                            println!("  while input[0] == {:?} {{", separator);
                            println!("    torpel_consume_token({:?}, input);", separator);
                            println!("    if {} {{", condition);
                            println!("      {}.push({}(input));", field_name, parser_name);
                            println!("    }} else {{");
                            println!("      break;");
                            println!("    }}");
                            println!("  }}");
                            fields.push(field_name);
                        }
                        grammar::RuleAction::RuleName(rule_name) => {
                            let field_name = to_field_name(rule_name);
                            let parser_name = to_parser_name(rule_name);
                            println!("  let {} = {}(input);", field_name, parser_name);
                            fields.push(field_name);
                        }
                        grammar::RuleAction::Unknown(_) => panic!("unknown RuleAction"),
                    }
                }
                println!("  return {} {{ {} }};", type_name, fields.join(", "));
            }
            grammar::RuleType::RuleChoice(sub_rule_names) => {
                let mut checks = HashSet::new();
                for rn in sub_rule_names {
                    let condition = generate_check_if_can_start_rule(grammar, rn);
                    if checks.contains(&condition) {
                        panic!("ambiguous RuleChoice");
                    }
                    checks.insert(condition);
                }
                for rn in sub_rule_names {
                    let condition = generate_check_if_can_start_rule(grammar, rn);
                    let rn_type_name = to_type_name(rn);
                    let rn_parser_name = to_parser_name(rn);
                    println!("  if {} {{", condition);
                    println!("    return {}::{}({}(input));", type_name, rn_type_name, rn_parser_name);
                    println!("  }}");
                }
                let panic_message = format!(r#"unexpected token: expected a valid sub-rule at {{:?}}"#);
                println!("  panic!({:?}, input)", panic_message);
            }
        }
        println!("}}");
    }
}

fn main() {
    let grammar = grammar::read_grammar_from_file("spec/torpel-grammar.pseudo-bnf");
    // println!("== GRAMMAR ==\n{}", grammar);
    rust_prelude();
    println!("\n\n");
    grammar_to_rust_types(&grammar);
    println!("\n\n");
    grammar_to_rust_parsers(&grammar);

    if false {
        let program = fs::read_to_string("spec/test-example-structures.torpel")
            .expect("Could not open program");

        grammar::parse_program(&grammar, &program);
    }
}
