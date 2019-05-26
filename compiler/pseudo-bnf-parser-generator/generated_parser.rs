use regex::Regex;

fn torpel_consume_token(token: &str, input: &mut Vec<&str>) {
  if input[0] == token {
    input.remove(0).to_string();
  } else {
    panic!("unexpected token: expected \"{}\" at {:?}", token, input)
  }
}

fn torpel_is_valid_user_specified_name(token: &str) -> bool {
  lazy_static! {
    static ref RE: Regex = Regex::new("^[A-Z][a-zA-Z-]*$").unwrap();
  }
  return RE.is_match(token);
}

fn torpel_read_user_specified_name(input: &mut Vec<&str>) -> String {
  if torpel_is_valid_user_specified_name(input[0]) {
    return input.remove(0).to_string();
  } else {
    panic!("unexpected token: expected user specified name at {:?}", input)
  }
}




#[derive(Debug)]
pub struct Start {
  new_type: Vec<NewType>,
}

pub type EnumerationAlternativeName = String;

#[derive(Debug)]
pub struct Enumeration {
  enumeration_alternative_name: Vec<EnumerationAlternativeName>,
}

#[derive(Debug)]
pub struct NewType {
  type_name: TypeName,
  type_definition: TypeDefinition,
}

pub type StructureFieldName = String;

#[derive(Debug)]
pub struct Structure {
  structure_field_name: Vec<StructureFieldName>,
}

#[derive(Debug)]
pub enum TypeDefinition {
  Structure(Structure),
  Enumeration(Enumeration),
}

pub type TypeName = String;




pub fn parse_start(input: &mut Vec<&str>) -> Start {
  let mut new_type = vec![];
  new_type.push(parse_new_type(input));
  while input[0] == ";" {
    torpel_consume_token(";", input);
    if input[0] == "new-type" {
      new_type.push(parse_new_type(input));
    } else {
      break;
    }
  }
  return Start { new_type };
}

pub fn parse_enumeration_alternative_name(input: &mut Vec<&str>) -> EnumerationAlternativeName {
  return torpel_read_user_specified_name(input);
}

pub fn parse_enumeration(input: &mut Vec<&str>) -> Enumeration {
  torpel_consume_token("enumeration", input);
  torpel_consume_token("[", input);
  let mut enumeration_alternative_name = vec![];
  enumeration_alternative_name.push(parse_enumeration_alternative_name(input));
  while input[0] == "|" {
    torpel_consume_token("|", input);
    if torpel_is_valid_user_specified_name(input[0]) {
      enumeration_alternative_name.push(parse_enumeration_alternative_name(input));
    } else {
      break;
    }
  }
  torpel_consume_token("]", input);
  return Enumeration { enumeration_alternative_name };
}

pub fn parse_new_type(input: &mut Vec<&str>) -> NewType {
  torpel_consume_token("new-type", input);
  let type_name = parse_type_name(input);
  let type_definition = parse_type_definition(input);
  return NewType { type_name, type_definition };
}

pub fn parse_structure_field_name(input: &mut Vec<&str>) -> StructureFieldName {
  return torpel_read_user_specified_name(input);
}

pub fn parse_structure(input: &mut Vec<&str>) -> Structure {
  torpel_consume_token("structure", input);
  torpel_consume_token("{", input);
  let mut structure_field_name = vec![];
  structure_field_name.push(parse_structure_field_name(input));
  while input[0] == "," {
    torpel_consume_token(",", input);
    if torpel_is_valid_user_specified_name(input[0]) {
      structure_field_name.push(parse_structure_field_name(input));
    } else {
      break;
    }
  }
  torpel_consume_token("}", input);
  return Structure { structure_field_name };
}

pub fn parse_type_definition(input: &mut Vec<&str>) -> TypeDefinition {
  if input[0] == "structure" {
    return TypeDefinition::Structure(parse_structure(input));
  }
  if input[0] == "enumeration" {
    return TypeDefinition::Enumeration(parse_enumeration(input));
  }
  panic!("unexpected token: expected a valid sub-rule at {:?}", input)
}

pub fn parse_type_name(input: &mut Vec<&str>) -> TypeName {
  return torpel_read_user_specified_name(input);
}
