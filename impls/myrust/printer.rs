use std::collections::HashMap;

use super::types::MalType;

pub fn pr_str(mal: MalType) -> String {
    match mal {
        MalType::Number(num) => num.to_string(),
        MalType::Symbol(sym) => sym,
        MalType::List(list) => print_list_readably(list, ("(", ")")),
        MalType::Nil => "nil".to_owned(),
        MalType::Bool(bool) => bool.to_string(),
        MalType::String(string) => print_string_readably(&string),
        MalType::Keyword(mut key) => {
            assert_eq!(key.pop(), Some('\u{29E}'));
            key
        }
        MalType::Vector(vec) => print_list_readably(vec, ("[", "]")),
        MalType::HashMap(map) => print_map_readably(map),
        MalType::BuiltinFunc(_) => "<builtin>".to_string(),
    }
}

/// For printing `MalType::String`
fn print_string_readably(string: &str) -> String {
    let mut output = String::from('"');

    for c in string.chars() {
        match c {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            _ => output.push(c),
        }
    }

    output.push('"');
    output
}

/// For printing `MalType::{List, Vec}`
fn print_list_readably(list: Vec<MalType>, delims: (&str, &str)) -> String {
    format!(
        "{}{}{}",
        delims.0,
        list.into_iter()
            .map(pr_str)
            .collect::<Vec<String>>()
            .join(" "),
        delims.1
    )
}

/// For printing `MalType::HashMap`
fn print_map_readably(map: HashMap<String, MalType>) -> String {
    let inner = map
        .into_iter()
        .map(|(mut k, v)| {
            format!(
                "{} {}",
                if k.ends_with('\u{29E}') {
                    k.pop();
                    k
                } else {
                    print_string_readably(&k)
                },
                pr_str(v)
            )
        })
        .collect::<Vec<String>>()
        .join(" ");
    format!("{{{inner}}}")
}
