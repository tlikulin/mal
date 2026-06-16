use std::collections::HashMap;

use crate::types::new_list;

use super::types::MalType;

pub fn pr_str(mal: MalType, readably: bool) -> String {
    match mal {
        MalType::Number(num) => num.to_string(),
        MalType::Symbol(sym) => sym,
        MalType::List(list, ..) => print_list(list, ("(", ")"), readably),
        MalType::Nil => "nil".to_owned(),
        MalType::Bool(bool) => bool.to_string(),
        MalType::String(string) => {
            if readably {
                print_string_readably(&string)
            } else {
                string
            }
        }
        MalType::Keyword(mut key) => {
            assert_eq!(key.pop(), Some('\u{29E}'));
            key
        }
        MalType::Vector(vec, ..) => print_list(vec, ("[", "]"), readably),
        MalType::HashMap(map, ..) => print_map(map, readably),
        MalType::BuiltinFunc(..) => "#<builtin>".to_string(),
        MalType::Lambda { params, body, .. } => format!(
            "(fn* {} {})",
            pr_str(new_list(params), readably),
            pr_str(*body, readably)
        ),
        MalType::Atom(inner) => format!("(atom {})", pr_str(inner.borrow().clone(), readably)),
    }
}

/// For printing `MalType::String`
pub fn print_string_readably(string: &str) -> String {
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
fn print_list(list: Vec<MalType>, delims: (&str, &str), readably: bool) -> String {
    format!(
        "{}{}{}",
        delims.0,
        list.into_iter()
            .map(|el| pr_str(el, readably))
            .collect::<Vec<String>>()
            .join(" "),
        delims.1
    )
}

/// For printing `MalType::HashMap`
fn print_map(map: HashMap<String, MalType>, readably: bool) -> String {
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
                pr_str(v, readably)
            )
        })
        .collect::<Vec<String>>()
        .join(" ");
    format!("{{{inner}}}")
}
