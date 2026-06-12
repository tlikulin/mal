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

fn eval(mal: MalType, env: &Env) -> MalResult {
    if env.get("DEBUG-EVAL").is_ok_and(|dbg| dbg.to_bool()) {
        println!("EVAL: {}", printer::pr_str(mal.clone(), true));
    }

    match mal {
        MalType::Symbol(sym) => env.get(&sym),

        MalType::List(list) => match list.first() {
            None => Ok(MalType::List(list)),
            Some(MalType::Symbol(sym)) if sym == "def!" => eval_def(list, env),
            Some(MalType::Symbol(sym)) if sym == "let*" => eval_let(list, env),
            Some(MalType::Symbol(sym)) if sym == "do" => eval_do(list, env),
            Some(MalType::Symbol(sym)) if sym == "if" => eval_if(list, env),
            Some(MalType::Symbol(sym)) if sym == "fn*" => eval_fn(list, env),
            Some(_) => {
                let mut it = list.into_iter().map(|m| eval(m, env));

                let func = it.next().unwrap()?;

                let args = it.collect::<Result<_, _>>()?;

                func.call(args)
            }
        },

        MalType::Vector(vec) => Ok(MalType::Vector(
            vec.into_iter()
                .map(|m| eval(m, env))
                .collect::<Result<_, _>>()?,
        )),

        MalType::HashMap(map) => Ok(MalType::HashMap(
            map.into_iter()
                .map(|(k, v)| eval(v, env).map(|m| (k, m)))
                .collect::<Result<_, _>>()?,
        )),

        _ => Ok(mal),
    }
}

fn eval_def(list: Vec<MalType>, env: &Env) -> MalResult {
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

    let value = eval(it.next().unwrap(), env)?;
    env.set(key.clone(), value);
    env.get(&key)
}

fn eval_let(list: Vec<MalType>, env: &Env) -> MalResult {
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
    let new_env = Env::new(Some(env.clone()), vec![], vec![]).unwrap();

    while let (Some(raw_key), Some(raw_value)) = (it.next(), it.next()) {
        let MalType::Symbol(key) = raw_key else {
            return Err(MalError::EvalError(
                "key in 'let*' is not symbol".to_string(),
            ));
        };

        let value = eval(raw_value, &new_env)?;

        new_env.set(key, value);
    }

    eval(to_eval, &new_env)
}

fn eval_do(list: Vec<MalType>, env: &Env) -> MalResult {
    let mut it = list
        .into_iter()
        .skip(1)
        .map(|m| eval(m, env))
        .collect::<Result<Vec<_>, _>>()?;

    it.pop().map_or_else(
        || Err(MalError::EvalError("'do' expects some args".to_string())),
        Ok,
    )
}

fn eval_if(list: Vec<MalType>, env: &Env) -> MalResult {
    if list.len() < 3 {
        return Err(MalError::EvalError("too few args to 'if'".to_string()));
    }
    let mut it = list.into_iter();
    let _if_perse = it.next().unwrap();

    let cond = eval(it.next().unwrap(), env)?;

    if cond.to_bool() {
        let true_branch = it.next().unwrap();
        eval(true_branch, env)
    } else {
        let _true_branch = it.next().unwrap();
        let false_branch = it.next().unwrap_or(MalType::Nil);
        eval(false_branch, env)
    }
}

fn eval_fn(list: Vec<MalType>, env: &Env) -> MalResult {
    if list.len() != 3 {
        return Err(MalError::EvalError("'fn*' must have 2 args".to_string()));
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
        binds,
        body,
        env: env.clone(),
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
