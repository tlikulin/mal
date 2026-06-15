#![allow(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    reason = "must conform to same signature"
)]
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use crate::env::Env;
use crate::types::{MalError, MalResult, MalType};
use crate::{printer, reader};

pub fn ns() -> Vec<(&'static str, MalType)> {
    vec![
        ("*ARGV*", MalType::List(vec![])),
        ("+", builtin_fn(add)),
        ("-", builtin_fn(sub)),
        ("*", builtin_fn(mult)),
        ("/", builtin_fn(div)),
        ("prn", builtin_fn(prn)),
        ("list", builtin_fn(list)),
        ("list?", builtin_fn(is_list)),
        ("empty?", builtin_fn(is_empty)),
        ("count", builtin_fn(count)),
        ("=", builtin_fn(eq)),
        ("<", builtin_fn(lt)),
        ("<=", builtin_fn(le)),
        (">", builtin_fn(gt)),
        (">=", builtin_fn(ge)),
        ("pr-str", builtin_fn(prstr)),
        ("str", builtin_fn(str)),
        ("println", builtin_fn(println)),
        ("read-string", builtin_fn(read_string)),
        ("slurp", builtin_fn(slurp)),
        ("atom", builtin_fn(atom)),
        ("atom?", builtin_fn(is_atom)),
        ("deref", builtin_fn(deref)),
        ("reset!", builtin_fn(reset)),
        ("swap!", builtin_fn(swap)),
        ("cons", builtin_fn(cons)),
        ("concat", builtin_fn(concat)),
        ("vec", builtin_fn(vec)),
        ("nth", builtin_fn(nth)),
        ("first", builtin_fn(first)),
        ("rest", builtin_fn(rest)),
        ("macro?", builtin_fn(is_macro)),
    ]
}

fn builtin_fn<F>(func: F) -> MalType
where
    F: Fn(Vec<MalType>) -> MalResult + 'static,
{
    MalType::BuiltinFunc(Rc::new(func))
}

fn add(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '+'".to_string()))
    } else if let (MalType::Number(b), MalType::Number(a)) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        Ok(MalType::Number(a + b))
    } else {
        Err(MalError::EvalError("'+' expects numbers".to_string()))
    }
}

fn sub(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '-'".to_string()))
    } else if let (MalType::Number(b), MalType::Number(a)) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        Ok(MalType::Number(a - b))
    } else {
        Err(MalError::EvalError("'-' expects numbers".to_string()))
    }
}

fn mult(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '*'".to_string()))
    } else if let (MalType::Number(b), MalType::Number(a)) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        Ok(MalType::Number(a * b))
    } else {
        Err(MalError::EvalError("'*' expects numbers".to_string()))
    }
}

fn div(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '/'".to_string()))
    } else if let (MalType::Number(b), MalType::Number(a)) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        Ok(MalType::Number(a / b))
    } else {
        Err(MalError::EvalError("'/' expects numbers".to_string()))
    }
}

fn prn(args: Vec<MalType>) -> MalResult {
    let Ok(MalType::String(s)) = prstr(args) else {
        unreachable!("prstr() should not fail")
    };
    println!("{s}");
    Ok(MalType::Nil)
}

const fn list(args: Vec<MalType>) -> MalResult {
    Ok(MalType::List(args))
}

fn is_list(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("too few args to 'list?'".to_string()))
    } else {
        Ok(MalType::Bool(matches!(args[0], MalType::List(_))))
    }
}

fn is_empty(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("too few args to 'empty?'".to_string()))
    } else if let MalType::List(inner) | MalType::Vector(inner) = &args[0] {
        Ok(MalType::Bool(inner.is_empty()))
    } else {
        Err(MalError::EvalError(
            "'empty?' expects list/vector".to_string(),
        ))
    }
}

fn count(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("too few args to 'count'".to_string()))
    } else if let MalType::List(inner) | MalType::Vector(inner) = &args[0] {
        Ok(MalType::Number(inner.len().try_into().unwrap()))
    } else if matches!(&args[0], MalType::Nil) {
        Ok(MalType::Number(0))
    } else {
        Err(MalError::EvalError(
            "'count' expects list/vector/nil".to_string(),
        ))
    }
}

fn eq(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '='".to_string()))
    } else {
        Ok(MalType::Bool(args[0] == args[1]))
    }
}

fn lt(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '<'".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 < n2))
    } else {
        Err(MalError::EvalError("'<' expects numbers".to_string()))
    }
}

fn le(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '<='".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 <= n2))
    } else {
        Err(MalError::EvalError("'<=' expects numbers".to_string()))
    }
}

fn gt(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '>'".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 > n2))
    } else {
        Err(MalError::EvalError("'>' expects numbers".to_string()))
    }
}

fn ge(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '>='".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 >= n2))
    } else {
        Err(MalError::EvalError("'>=' expects numbers".to_string()))
    }
}

fn prstr(args: Vec<MalType>) -> MalResult {
    Ok(MalType::String(
        args.into_iter()
            .map(|mal| printer::pr_str(mal, true))
            .collect::<Vec<_>>()
            .join(" "),
    ))
}

fn str(args: Vec<MalType>) -> MalResult {
    Ok(MalType::String(
        args.into_iter()
            .map(|mal| printer::pr_str(mal, false))
            .collect::<String>(),
    ))
}

fn println(args: Vec<MalType>) -> MalResult {
    println!(
        "{}",
        args.into_iter()
            .map(|mal| printer::pr_str(mal, false))
            .collect::<Vec<_>>()
            .join(" ")
    );
    Ok(MalType::Nil)
}

fn read_string(args: Vec<MalType>) -> MalResult {
    match args.first() {
        Some(MalType::String(string)) => reader::read_str(string),
        Some(_) => Err(MalError::EvalError(
            "'read_string' expects string".to_string(),
        )),
        None => Err(MalError::EmptyInput),
    }
}

fn slurp(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::String(filename)) = args.first() {
        let contents = fs::read_to_string(filename)?;
        Ok(MalType::String(contents))
    } else {
        Err(MalError::EvalError("'slurp' expects string".to_string()))
    }
}

fn atom(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("'atom' expects an arg".to_string()))
    } else {
        Ok(MalType::Atom(Rc::new(RefCell::new(args.swap_remove(0)))))
    }
}

fn is_atom(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("'atom?' expects an arg".to_string()))
    } else {
        Ok(MalType::Bool(matches!(args[0], MalType::Atom(_))))
    }
}

fn deref(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::Atom(inner)) = args.first() {
        Ok(inner.borrow().clone())
    } else {
        Err(MalError::EvalError("'deref' expects an atom".to_string()))
    }
}

fn reset(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(MalType::Atom(inner)), Some(mal)) = (it.next(), it.next()) {
        inner.replace(mal);
        Ok(inner.borrow().clone())
    } else {
        Err(MalError::EvalError(
            "'reset!' expects an atom and a value".to_string(),
        ))
    }
}

fn swap(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(MalType::Atom(inner)), Some(func)) = (it.next(), it.next()) {
        let mut ast = vec![func, inner.borrow().clone()];
        ast.extend(it);

        let blank_env = Env::new(None, vec![], vec![]).unwrap();
        let new_value = crate::eval(MalType::List(ast), &blank_env)?;

        inner.replace(new_value);
        Ok(inner.borrow().clone())
    } else {
        Err(MalError::EvalError(
            "'swap!' expects an atom, a func (and args)".to_string(),
        ))
    }
}

fn cons(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(first), Some(MalType::List(mut list) | MalType::Vector(mut list))) =
        (it.next(), it.next())
    {
        list.insert(0, first);
        Ok(MalType::List(list))
    } else {
        Err(MalError::EvalError(
            "'cons' expects a value and a list/vector".to_string(),
        ))
    }
}

fn concat(args: Vec<MalType>) -> MalResult {
    let mut result = Vec::new();
    for arg in args {
        let (MalType::List(list) | MalType::Vector(list)) = arg else {
            return Err(MalError::EvalError(
                "'concat' expects lists/vectors".to_string(),
            ));
        };

        result.extend(list);
    }
    Ok(MalType::List(result))
}

fn vec(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        return Err(MalError::EvalError(
            "'vec' expects a list/vector".to_string(),
        ));
    }

    match args.swap_remove(0) {
        MalType::List(list) => Ok(MalType::Vector(list)),
        vector @ MalType::Vector(_) => Ok(vector),
        _ => Err(MalError::EvalError(
            "'vec' expects a list/vector".to_string(),
        )),
    }
}

fn nth(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(MalType::List(mut list) | MalType::Vector(mut list)), Some(MalType::Number(ind))) =
        (it.next(), it.next())
    {
        match ind.try_into() {
            Ok(ind) if ind < list.len() => Ok(list.swap_remove(ind)),
            _ => Err(MalError::EvalError("out of bounds".to_string())),
        }
    } else {
        Err(MalError::EvalError(
            "'nth' expects list/vector and index".to_string(),
        ))
    }
}

fn first(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError(
            "'first' expects list/vector/nil".to_string(),
        ))
    } else {
        args.swap_remove(0).get_first().map_or_else(
            || {
                Err(MalError::EvalError(
                    "'first' expects list/vector/nil".to_string(),
                ))
            },
            Ok,
        )
    }
}

fn rest(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError(
            "'rest' expects list/vector/nil".to_string(),
        ))
    } else {
        match args.swap_remove(0) {
            MalType::Nil => Ok(MalType::List(vec![])),
            MalType::List(list) | MalType::Vector(list) if list.is_empty() => {
                Ok(MalType::List(vec![]))
            }
            MalType::List(mut list) | MalType::Vector(mut list) => {
                list.remove(0);
                Ok(MalType::List(list))
            }
            _ => Err(MalError::EvalError(
                "'rest' expects list/vector/nil".to_string(),
            )),
        }
    }
}

fn is_macro(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Lambda { is_macro: true, .. })
    )))
}
