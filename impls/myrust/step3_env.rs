use std::io::{self, Write};
use std::rc::Rc;

mod builtins;
mod env;
mod printer;
mod reader;
mod types;

use env::Env;
use types::{Callable, MalError, MalResult, MalType};

fn main() -> io::Result<()> {
    let builtin_functions = [
        ("+", Rc::new(builtins::add) as Callable),
        ("-", Rc::new(builtins::sub)),
        ("*", Rc::new(builtins::mult)),
        ("/", Rc::new(builtins::div)),
    ];

    let mut repl_env = Env::new(None);

    for (op, func) in builtin_functions {
        repl_env.set(op.to_string(), MalType::Function(func));
    }

    loop {
        print!("user> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.is_empty() {
            println!();
            return Ok(());
        } else if let Some(output) = rep(&input, &mut repl_env) {
            println!("{output}");
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: &str) -> MalResult {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(mal: MalType, env: &mut Env) -> MalResult {
    if is_debug_eval_set(env) {
        println!("EVAL: {}", printer::pr_str(mal.clone(), true));
    }

    match mal {
        MalType::Symbol(sym) => env.get(&sym),

        MalType::List(list) => match list.first() {
            None => Ok(MalType::List(list)),

            Some(MalType::Symbol(sym)) if sym == "def!" => {
                if list.len() != 3 {
                    return Err(MalError::EvalError(format!(
                        "'def!' expected 2 args, got {}",
                        list.len() - 1
                    )));
                }

                let mut it = list.into_iter();
                let _def_perse = it.next().unwrap();

                let MalType::Symbol(key) = it.next().unwrap() else {
                    return Err(MalError::EvalError(
                        "1st arg to 'def!' must be symbol".to_string(),
                    ));
                };

                let value = EVAL(it.next().unwrap(), env)?;
                env.set(key.clone(), value);
                env.get(&key)
            }

            Some(MalType::Symbol(sym)) if sym == "let*" => {
                if list.len() != 3 {
                    return Err(MalError::EvalError(format!(
                        "'let*' expected 2 args, got {}",
                        list.len() - 1
                    )));
                }

                let mut it = list.into_iter();
                let _let_perse = it.next().unwrap();

                let new_env_list = match it.next().unwrap() {
                    MalType::List(list) => list,
                    MalType::Vector(vec) => vec,
                    _ => {
                        return Err(MalError::EvalError(
                            "1st arg to 'let*' must be list".to_string(),
                        ));
                    }
                };

                if new_env_list.len() % 2 == 1 {
                    return Err(MalError::EvalError(
                        "odd number of args in 'let*' binding".to_string(),
                    ));
                }

                let to_eval = it.next().unwrap();

                let mut it = new_env_list.into_iter();

                let mut new_env = Env::new(Some(env.clone()));

                while let (Some(raw_key), Some(raw_value)) = (it.next(), it.next()) {
                    let MalType::Symbol(key) = raw_key else {
                        return Err(MalError::EvalError(
                            "key in 'let*' is not symbol".to_string(),
                        ));
                    };

                    let value = EVAL(raw_value, &mut new_env)?;

                    new_env.set(key, value);
                }

                EVAL(to_eval, &mut new_env)
            }

            Some(_) => {
                let mut it = list.into_iter().map(|m| EVAL(m, env));

                let Some(func) = it.next().unwrap()?.into_callable() else {
                    return Err(MalError::EvalError(
                        "1st lsit item not callable".to_string(),
                    ));
                };
                let args = it.collect::<Result<_, _>>()?;

                func(args)
            }
        },

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
        Err(_) | Ok(MalType::Nil | MalType::Bool(false))
    )
}

#[allow(non_snake_case)]
fn PRINT(mal: MalResult) -> Option<String> {
    match mal {
        Ok(mal) => Some(printer::pr_str(mal, true)),
        Err(MalError::EmptyInput) => None,
        Err(MalError::ParseError(msg)) => Some(format!("mal: parse error: {msg}")),
        Err(MalError::EvalError(msg)) => Some(format!("mal: eval error: {msg}")),
    }
}

fn rep(input: &str, env: &mut Env) -> Option<String> {
    PRINT(READ(input).and_then(|mal| EVAL(mal, env)))
}
