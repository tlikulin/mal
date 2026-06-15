use std::io::{self, Write};
use std::rc::Rc;

mod core;
mod env;
mod printer;
mod reader;
mod types;

use env::Env;
use types::{MalError, MalResult, MalType};

thread_local! {
    static REPL_ENV: Env = Env::new(None, vec![], vec![]).unwrap();
}

fn main() -> io::Result<()> {
    REPL_ENV.with(|repl_env| {
        for (key, val) in core::ns() {
            repl_env.set(key.to_string(), val);
        }

        repl_env.set(
            "eval".to_string(),
            MalType::BuiltinFunc(Rc::new(|mut args| {
                if args.is_empty() {
                    Err(MalError::EvalError("'eval' expects an arg".to_string()))
                } else {
                    REPL_ENV.with(|repl_env| eval(args.swap_remove(0), repl_env))
                }
            })),
        );

        let _not_func = rep("(def! not (fn* (a) (if a false true)))", repl_env);
        let _load_file_func = rep(r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#, repl_env);
        let _cond_macro = rep("(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond (rest (rest xs)))))))", repl_env);

        let mut cmd_args = std::env::args();
        let _arv0 = cmd_args.next().unwrap();

        // non-interactive
        if let Some(argv1) = cmd_args.next() {
            let input = format!("(load-file {})", printer::print_string_readably(&argv1));

            let argv = cmd_args.map(MalType::String).collect();
            repl_env.set("*ARGV*".to_string(), MalType::List(argv));

            if let Some(output) = rep(&input, repl_env)
                && output != "nil"
            {
                println!("{output}");
            }

            return Ok(());
        }

        // interactive
        loop {
            print!("user> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.is_empty() {
                println!();
                return Ok(());
            } else if let Some(output) = rep(&input, repl_env) {
                println!("{output}");
            }
        }
    })
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

                Some(MalType::Symbol(sym)) if sym == "def!" => return eval_def(list, env, false),

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

                Some(MalType::Symbol(sym)) if sym == "quote" => {
                    if list.len() < 2 {
                        return Err(MalError::EvalError("too few args to 'quote'".to_string()));
                    }
                    return Ok(list.swap_remove(1));
                }

                Some(MalType::Symbol(sym)) if sym == "quasiquote" => {
                    if list.len() < 2 {
                        return Err(MalError::EvalError(
                            "too few args to 'quasiquote'".to_string(),
                        ));
                    }

                    ast = quasiquote(list.swap_remove(1))?;
                }

                Some(MalType::Symbol(sym)) if sym == "defmacro!" => {
                    return eval_def(list, env, true);
                }

                Some(MalType::Symbol(sym)) if sym == "try*" => {
                    let mut it = list.into_iter();
                    let _try_perse = it.next().unwrap();

                    let Some(expr) = it.next() else {
                        return Err(MalError::EvalError("too few args to 'try*'".to_string()));
                    };

                    match eval(expr, env) {
                        Err(exc) => {
                            if let Some(catch_block) = it.next()
                                && catch_block.is_list_with_sym("catch*")
                            {
                                let MalType::List(mut catch_list) = catch_block else {
                                    unreachable!()
                                };
                                if catch_list.len() < 3 {
                                    return Err(MalError::EvalError(
                                        "too few args to 'catch*'".to_string(),
                                    ));
                                }
                                if let (to_eval, MalType::Symbol(sym)) =
                                    (catch_list.swap_remove(2), catch_list.swap_remove(1))
                                {
                                    live_env = Env::new(Some(env.clone()), vec![], vec![]).unwrap();
                                    live_env.set(sym, exc.into_mal());
                                    env = &live_env;
                                    ast = to_eval;
                                } else {
                                    return Err(MalError::EvalError(
                                        "'catch*' expects symbol to bind".to_string(),
                                    ));
                                }
                            } else {
                                return Err(exc);
                            }
                        }
                        success => return success,
                    }
                }

                // apply case
                _ => {
                    let mut it = list.into_iter();

                    let func = eval(it.next().unwrap(), env)?;

                    match func {
                        MalType::BuiltinFunc(builtin) => {
                            let args = it.map(|m| eval(m, env)).collect::<Result<Vec<_>, _>>()?;
                            return builtin(args);
                        }
                        MalType::Lambda {
                            params,
                            body,
                            capt_env,
                            is_macro: false,
                        } => {
                            let args = it.map(|m| eval(m, env)).collect::<Result<Vec<_>, _>>()?;
                            live_env = Env::new(Some(capt_env), params, args)?;
                            env = &live_env;
                            ast = *body;
                        }
                        MalType::Lambda {
                            params,
                            body,
                            capt_env,
                            is_macro: true,
                        } => {
                            let raw_args = it.collect();
                            let macro_env = Env::new(Some(capt_env), params, raw_args)?;
                            ast = eval(*body, &macro_env)?;
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

fn eval_def(list: Vec<MalType>, env: &Env, is_macro: bool) -> MalResult {
    if list.len() < 3 {
        return Err(MalError::EvalError(
            "too few args to 'def!'('defmacro!')".to_string(),
        ));
    }

    let mut it = list.into_iter();
    let _def_perse = it.next().unwrap();

    let MalType::Symbol(key) = it.next().unwrap() else {
        return Err(MalError::EvalError(
            "1st arg to 'def!'('defmacro!') must be symbol".to_string(),
        ));
    };

    let mut value = eval(it.next().unwrap(), env)?;

    if is_macro {
        if !matches!(value, MalType::Lambda { .. }) {
            return Err(MalError::EvalError(
                "'defmacro!' expects a function".to_string(),
            ));
        }
        value.set_macro();
    }

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
        capt_env: env.clone(),
        is_macro: false,
    })
}

fn quasiquote(ast: MalType) -> MalResult {
    let was_vector = ast.is_vector();
    match ast {
        MalType::List(mut list) if ast.is_list_with_sym("unquote") => {
            if list.len() < 2 {
                Err(MalError::EvalError(
                    "too few args too 'unquote'".to_string(),
                ))
            } else {
                Ok(list.swap_remove(1))
            }
        }

        MalType::List(list) | MalType::Vector(list) => {
            let mut result = Vec::new();

            for elt in list.into_iter().rev() {
                if elt.is_list_with_sym("splice-unquote") {
                    let MalType::List(mut elt_list) = elt else {
                        unreachable!()
                    };
                    if elt_list.len() < 2 {
                        return Err(MalError::EvalError(
                            "too few args too 'splice-unquote'".to_string(),
                        ));
                    }

                    result = vec![
                        MalType::Symbol("concat".to_string()),
                        elt_list.swap_remove(1),
                        MalType::List(result),
                    ];
                } else {
                    result = vec![
                        MalType::Symbol("cons".to_string()),
                        quasiquote(elt)?,
                        MalType::List(result),
                    ];
                }
            }

            let result = MalType::List(result);
            if was_vector {
                Ok(MalType::List(vec![
                    MalType::Symbol("vec".to_string()),
                    result,
                ]))
            } else {
                Ok(result)
            }
        }

        MalType::HashMap(_) | MalType::Symbol(_) => Ok(MalType::List(vec![
            MalType::Symbol("quote".to_string()),
            ast,
        ])),

        _ => Ok(ast),
    }
}

fn print(mal: MalResult) -> Option<String> {
    match mal {
        Ok(mal) => Some(printer::pr_str(mal, true)),
        Err(MalError::EmptyInput) => None,
        Err(MalError::ParseError(msg)) => Some(format!("mal: parse error: {msg}")),
        Err(MalError::EvalError(msg)) => Some(format!("mal: eval error: {msg}")),
        Err(MalError::IOError(e)) => Some(format!("mal: IO error: {e}")),
        Err(MalError::Exception(exc)) => Some(format!(
            "mal: uncaught exception: {}",
            printer::pr_str(exc, true)
        )),
    }
}

fn rep(input: &str, env: &Env) -> Option<String> {
    print(read(input).and_then(|mal| eval(mal, env)))
}
