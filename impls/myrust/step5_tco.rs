use std::io::{self, Write};

mod core;
mod env;
mod printer;
mod reader;
mod types;

use env::Env;
use types::{MalError, MalResult, MalType};

fn main() -> io::Result<()> {
    let repl_env = Env::new(None, vec![], vec![]).unwrap();

    for (key, val) in core::ns() {
        repl_env.set(key.to_string(), val);
    }

    let _not_func_ = rep("(def! not (fn* (a) (if a false true)))", &repl_env).unwrap();

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

fn read(input: &str) -> MalResult {
    reader::read_str(input)
}

#[allow(clippy::too_many_lines)]
fn eval(mut ast: MalType, orig_env: &Env) -> MalResult {
    let mut env = orig_env;
    // to keep up borrowing rules
    let mut live_env;

    loop {
        if env.get("DEBUG-EVAL").is_ok_and(|dbg| dbg.to_bool()) {
            println!("EVAL: {}", printer::pr_str(ast.clone(), true));
        }

        match ast {
            MalType::Symbol(sym) => return env.get(&sym),

            MalType::List(mut list) => match list.first() {
                None => return Ok(MalType::List(list)),

                Some(MalType::Symbol(sym)) if sym == "def!" => return eval_def(list, env),

                Some(MalType::Symbol(sym)) if sym == "let*" => {
                    if list.len() < 3 {
                        return Err(MalError::EvalError("too few args to 'let*".to_string()));
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

                    ast = it.next().unwrap();

                    let mut it = new_env_list.into_iter();
                    live_env = Env::new(Some(env.clone()), vec![], vec![]).unwrap();

                    while let (Some(raw_key), Some(raw_value)) = (it.next(), it.next()) {
                        let MalType::Symbol(key) = raw_key else {
                            return Err(MalError::EvalError(
                                "key in 'let*' is not symbol".to_string(),
                            ));
                        };

                        let value = eval(raw_value, &live_env)?;
                        live_env.set(key, value);
                    }

                    env = &live_env;
                }

                Some(MalType::Symbol(sym)) if sym == "do" => {
                    if list.len() <= 1 {
                        return Ok(MalType::Nil);
                    }

                    ast = list.pop().unwrap();

                    for to_eval in list.into_iter().skip(1) {
                        let _evaled = eval(to_eval, env)?;
                    }
                }

                Some(MalType::Symbol(sym)) if sym == "if" => {
                    if list.len() < 3 {
                        return Err(MalError::EvalError("too few args to 'if'".to_string()));
                    }

                    let mut it = list.into_iter();
                    let _if_perse = it.next().unwrap();

                    let cond = eval(it.next().unwrap(), env)?;
                    let true_branch = it.next().unwrap();

                    ast = if cond.to_bool() {
                        true_branch
                    } else {
                        match it.next() {
                            Some(false_branch) => false_branch,
                            None => return Ok(MalType::Nil),
                        }
                    };
                }

                Some(MalType::Symbol(sym)) if sym == "fn*" => return eval_fn(list, env),

                // apply case
                Some(_) => {
                    let mut it = list.into_iter().map(|m| eval(m, env));

                    let func = it.next().unwrap()?;

                    let args = it.collect::<Result<_, _>>()?;

                    match func {
                        MalType::BuiltinFunc(builtin) => return builtin(args),
                        MalType::Lambda {
                            params,
                            body,
                            l_env,
                        } => {
                            ast = *body;
                            live_env = Env::new(Some(l_env), params, args)?;
                            env = &live_env;
                        }
                        _ => return Err(MalError::EvalError("not callable".to_string())),
                    }
                }
            },

            MalType::Vector(vec) => {
                return Ok(MalType::Vector(
                    vec.into_iter()
                        .map(|m| eval(m, env))
                        .collect::<Result<_, _>>()?,
                ));
            }

            MalType::HashMap(map) => {
                return Ok(MalType::HashMap(
                    map.into_iter()
                        .map(|(k, v)| eval(v, env).map(|m| (k, m)))
                        .collect::<Result<_, _>>()?,
                ));
            }

            _ => return Ok(ast),
        }
    }
}

fn eval_def(list: Vec<MalType>, env: &Env) -> MalResult {
    if list.len() < 3 {
        return Err(MalError::EvalError("too few args to 'def!".to_string()));
    }

    let mut it = list.into_iter();
    let _def_perse = it.next().unwrap();

    let MalType::Symbol(key) = it.next().unwrap() else {
        return Err(MalError::EvalError(
            "1st arg to 'def!' must be symbol".to_string(),
        ));
    };

    let value = eval(it.next().unwrap(), env)?;
    env.set(key.clone(), value);
    env.get(&key)
}

fn eval_fn(list: Vec<MalType>, env: &Env) -> MalResult {
    if list.len() < 3 {
        return Err(MalError::EvalError("too few args to 'fn*".to_string()));
    }

    let mut it = list.into_iter();
    let _fn_perse = it.next().unwrap();

    let binds = match it.next().unwrap() {
        MalType::List(l) => l,
        MalType::Vector(v) => v,
        _ => {
            return Err(MalError::EvalError(
                "1st arg to 'fn*' must be list/vector".to_string(),
            ));
        }
    };
    let body = Box::new(it.next().unwrap());

    Ok(MalType::Lambda {
        params: binds,
        body,
        l_env: env.clone(),
    })
}

fn print(mal: MalResult) -> Option<String> {
    match mal {
        Ok(mal) => Some(printer::pr_str(mal, true)),
        Err(MalError::EmptyInput) => None,
        Err(MalError::ParseError(msg)) => Some(format!("mal: parse error: {msg}")),
        Err(MalError::EvalError(msg)) => Some(format!("mal: eval error: {msg}")),
    }
}

fn rep(input: &str, env: &Env) -> Option<String> {
    print(read(input).and_then(|mal| eval(mal, env)))
}
