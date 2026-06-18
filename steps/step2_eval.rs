use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

mod builtins;
mod printer;
mod reader;
mod types;

use types::{Callable, MalError, MalResult, MalType};

type Env = HashMap<String, MalType>;

fn main() -> io::Result<()> {
    let builtins = [
        ("+", Rc::new(builtins::add) as Callable),
        ("-", Rc::new(builtins::sub)),
        ("*", Rc::new(builtins::mult)),
        ("/", Rc::new(builtins::div)),
    ];

    let repl_env: Env = builtins
        .into_iter()
        .map(|(op, func)| (op.to_string(), MalType::BuiltinFunc(func)))
        .collect();

    // repl_env.insert("DEBUG-EVAL".to_string(), MalType::Bool(true));

    loop {
        print!("user> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.is_empty() {
            println!();
            return Ok(());
        } else if let Some(output) = rep(&input, &repl_env) {
            println!("{output}");
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: &str) -> MalResult {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(mal: MalType, env: &Env) -> MalResult {
    if is_debug_eval_set(env) {
        println!("EVAL: {}", printer::pr_str(mal.clone()));
    }

    match mal {
        MalType::Symbol(sym) => env.get(&sym).map_or_else(
            || Err(MalError::EvalError(format!("'{sym}' not found"))),
            |val| Ok(val.clone()),
        ),
        MalType::List(list) if !list.is_empty() => {
            let mut it = list.into_iter().map(|m| EVAL(m, env));

            let Some(func) = it.next().unwrap()?.into_callable() else {
                return Err(MalError::EvalError(
                    "1st lsit item not callable".to_string(),
                ));
            };
            let args = it.collect::<Result<_, _>>()?;

            func(args)
        }
        MalType::Vector(vec) => Ok(MalType::Vector(
            vec.into_iter()
                .map(|m| EVAL(m, env))
                .collect::<Result<_, _>>()?,
        )),
        MalType::HashMap(map) => Ok(MalType::HashMap(
            map.into_iter()
                .map(|(k, v)| EVAL(v, env).map(|m| (k, m)))
                .collect::<Result<_, _>>()?,
        )),
        _ => Ok(mal),
    }
}

fn is_debug_eval_set(env: &Env) -> bool {
    !matches!(
        env.get("DEBUG-EVAL"),
        None | Some(MalType::Nil | MalType::Bool(false))
    )
}

#[allow(non_snake_case)]
fn PRINT(mal: MalResult) -> Option<String> {
    match mal {
        Ok(mal) => Some(printer::pr_str(mal)),
        Err(MalError::EmptyInput) => None,
        Err(MalError::ParseError(msg)) => Some(format!("mal: parse error: {msg}")),
        Err(MalError::EvalError(msg)) => Some(format!("mal: eval error: {msg}")),
    }
}

fn rep(input: &str, env: &Env) -> Option<String> {
    PRINT(READ(input).and_then(|mal| EVAL(mal, env)))
}
