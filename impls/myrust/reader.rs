use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

use regex::Regex;

use super::types::{MalError, MalResult, MalType};
use MalError::{EmptyInput, ParseError};

pub struct Reader {
    tokens: VecDeque<String>,
}

impl Reader {
    const fn new(tokens: VecDeque<String>) -> Self {
        Self { tokens }
    }

    fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    fn next(&mut self) -> String {
        self.tokens.pop_front().unwrap()
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.front()
    }
}

pub fn read_str(input: &str) -> MalResult {
    let tokens = tokenize(input);
    let mut reader = Reader::new(tokens);
    if reader.is_empty() {
        Err(EmptyInput)
    } else {
        read_form(&mut reader)
    }
}

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap()
});

fn tokenize(input: &str) -> VecDeque<String> {
    TOKEN_REGEX
        .captures_iter(input)
        .map(|mat| String::from(&mat[1]))
        .filter(|t| !t.starts_with(';'))
        .take_while(|s| !s.is_empty())
        .collect()
}

fn read_form(reader: &mut Reader) -> MalResult {
    match reader.peek() {
        Some(t) if t == "(" => Ok(MalType::List(read_list(reader, ("(", ")"))?)),
        Some(t) if t == "[" => Ok(MalType::Vector(read_list(reader, ("[", "]"))?)),
        Some(t) if t == "{" => read_map(read_list(reader, ("{", "}"))?),
        Some(t) if is_macro(t) => read_macro(reader),
        Some(_) => read_atom(reader),
        None => Err(ParseError("underflow".to_string())),
    }
}

fn read_list(reader: &mut Reader, delims: (&str, &str)) -> Result<Vec<MalType>, MalError> {
    let mut list = Vec::new();
    assert_eq!(reader.next(), delims.0);

    while !reader.is_empty() {
        if let Some(t) = reader.peek()
            && t == delims.1
        {
            assert_eq!(reader.next(), delims.1);
            return Ok(list);
        }

        list.push(read_form(reader)?);
    }

    Err(ParseError(format!("unbalanced '{}'", delims.0)))
}

fn read_atom(reader: &mut Reader) -> MalResult {
    let token = reader.next();

    token.parse().map_or_else(
        |_| match token.as_str() {
            "nil" => Ok(MalType::Nil),
            "true" => Ok(MalType::Bool(true)),
            "false" => Ok(MalType::Bool(false)),
            t if t.starts_with('"') => read_atom_string(&token),
            t if t.starts_with(':') => Ok(MalType::Keyword(token)),
            ")" | "]" | "}" => Err(ParseError(format!("unbalanced '{token}'"))),
            _ => Ok(MalType::Symbol(token)),
        },
        |num| Ok(MalType::Number(num)),
    )
}

fn read_atom_string(token: &str) -> MalResult {
    assert!(token.starts_with('"'));
    if token.len() >= 2 && token.ends_with('"') {
        let mut string = String::new();
        let mut escaped = false;

        for c in token[1..token.len() - 1].chars() {
            match c {
                '\\' | '"' if escaped => {
                    string.push(c);
                    escaped = false;
                }
                'n' if escaped => {
                    string.push('\n');
                    escaped = false;
                }
                _ if escaped => {
                    return Err(ParseError(format!("can't escape '{c}' in string")));
                }
                '\\' => escaped = true,
                _ => string.push(c),
            }
        }

        if escaped {
            Err(ParseError("invalid string literal".into()))
        } else {
            Ok(MalType::String(string))
        }
    } else {
        Err(ParseError("unbalanced '\"'".into()))
    }
}

fn read_map(mut list: Vec<MalType>) -> MalResult {
    if list.len() % 2 == 1 {
        Err(ParseError("hash-map can't have odd number of items".into()))
    } else {
        let mut map = HashMap::new();

        while !list.is_empty() {
            let (value, raw_key) = (list.pop().unwrap(), list.pop().unwrap());

            let key = match raw_key {
                MalType::String(string) => string,
                MalType::Keyword(keyword) => keyword,
                _ => return Err(ParseError("hash-map key not hashable".into())),
            };

            map.insert(key, value);
        }

        Ok(MalType::HashMap(map))
    }
}

fn is_macro(token: &str) -> bool {
    ["'", "`", "~", "~@", "@", "^"].contains(&token)
}

fn read_macro(reader: &mut Reader) -> MalResult {
    let macro_symbol = match reader.next().as_str() {
        "'" => String::from("quote"),
        "`" => String::from("quasiquote"),
        "~" => String::from("unquote"),
        "~@" => String::from("splice-unquote"),
        "@" => String::from("deref"),
        "^" => String::from("with-meta"),
        t => unreachable!("{t} is not a valid quote kind"),
    };

    if macro_symbol == "with-meta" {
        let target1 = read_form(reader)?;
        let target2 = read_form(reader)?;

        Ok(MalType::List(vec![
            MalType::Symbol(macro_symbol),
            target2,
            target1,
        ]))
    } else {
        let target = read_form(reader)?;

        Ok(MalType::List(vec![MalType::Symbol(macro_symbol), target]))
    }
}
