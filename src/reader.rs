use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

use regex::Regex;

use crate::types::{MalError, MalResult, MalType};
use crate::types::{new_hashmap, new_list, new_vector};
use MalError::{EmptyInput, ParseError};

pub const KW_SUFFIX: char = '\u{29E}';

struct Reader {
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
    let mut reader = Reader::new(tokenize(input));

    if reader.is_empty() {
        Err(EmptyInput)
    } else {
        read_form(&mut reader)
    }
}

fn tokenize(input: &str) -> VecDeque<String> {
    static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
            .unwrap()
    });

    TOKEN_REGEX
        .captures_iter(input)
        .map(|mat| String::from(&mat[1]))
        .filter(|t| !t.starts_with(';'))
        .take_while(|s| !s.is_empty())
        .collect()
}

fn read_form(reader: &mut Reader) -> MalResult {
    match reader.peek() {
        Some(t) if t == "(" => Ok(new_list(read_list(reader, ("(", ")"))?)),
        Some(t) if t == "[" => Ok(new_vector(read_list(reader, ("[", "]"))?)),
        Some(t) if t == "{" => read_map(read_list(reader, ("{", "}"))?),
        Some(t) if is_macro(t) => read_macro(reader),
        Some(_) => read_atom(reader),
        None => Err(ParseError("underflow".to_string())),
    }
}

fn read_list(reader: &mut Reader, delims: (&str, &str)) -> Result<Vec<MalType>, MalError> {
    let mut list = Vec::new();
    let _open_delim = reader.next();

    while !reader.is_empty() {
        if let Some(t) = reader.peek()
            && t == delims.1
        {
            let _close_delim = reader.next();
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
            t if t.starts_with(':') => Ok(MalType::Keyword(format!("{token}{KW_SUFFIX}"))),
            ")" | "]" | "}" => Err(ParseError(format!("unbalanced '{token}'"))),
            _ => Ok(MalType::Symbol(token)),
        },
        |num| Ok(MalType::Number(num)),
    )
}

fn read_atom_string(token: &str) -> MalResult {
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
                    string.push(c);
                    escaped = false;
                }
                '\\' => escaped = true,
                _ => string.push(c),
            }
        }

        if escaped {
            Err(ParseError("unbalanced '\\'".to_string()))
        } else {
            Ok(MalType::String(string))
        }
    } else {
        Err(ParseError("unbalanced '\"'".to_string()))
    }
}

pub fn read_map(list: Vec<MalType>) -> MalResult {
    if list.len() % 2 == 1 {
        return Err(ParseError("odd number of items in {} literal".to_string()));
    }

    let mut map = HashMap::new();

    let mut it = list.into_iter();

    while let (Some(raw_key), Some(value)) = (it.next(), it.next()) {
        map.insert(raw_key.to_key()?.to_owned(), value);
    }

    Ok(new_hashmap(map))
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
        t => unreachable!("{t} is not a valid reader macro"),
    };

    if macro_symbol == "with-meta" {
        let target1 = read_form(reader)?;
        let target2 = read_form(reader)?;

        Ok(new_list(vec![
            MalType::Symbol(macro_symbol),
            target2,
            target1,
        ]))
    } else {
        let target = read_form(reader)?;

        Ok(new_list(vec![MalType::Symbol(macro_symbol), target]))
    }
}
