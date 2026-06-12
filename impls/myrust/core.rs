use std::rc::Rc;

use crate::{
    printer,
    types::{MalError, MalResult, MalType},
};

pub fn ns() -> Vec<(&'static str, MalType)> {
    vec![
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

#[allow(clippy::unnecessary_wraps)]
fn prn(args: Vec<MalType>) -> MalResult {
    let Ok(MalType::String(s)) = prstr(args) else {
        unreachable!("prstr() should not fail")
    };
    println!("{s}");
    Ok(MalType::Nil)
}

#[allow(clippy::unnecessary_wraps)]
const fn list(args: Vec<MalType>) -> MalResult {
    Ok(MalType::List(args))
}

#[allow(clippy::needless_pass_by_value)]
fn is_list(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(MalError::EvalError("too few args to 'list?'".to_string()))
    } else {
        Ok(MalType::Bool(matches!(args[0], MalType::List(_))))
    }
}

#[allow(clippy::needless_pass_by_value)]
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

#[allow(clippy::needless_pass_by_value)]
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

#[allow(clippy::needless_pass_by_value)]
fn eq(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '='".to_string()))
    } else {
        Ok(MalType::Bool(args[0] == args[1]))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn lt(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '<'".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 < n2))
    } else {
        Err(MalError::EvalError("'<' expects numbers".to_string()))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn le(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '<='".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 <= n2))
    } else {
        Err(MalError::EvalError("'<=' expects numbers".to_string()))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn gt(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '>'".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 > n2))
    } else {
        Err(MalError::EvalError("'>' expects numbers".to_string()))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn ge(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(MalError::EvalError("too few args to '>='".to_string()))
    } else if let (MalType::Number(n1), MalType::Number(n2)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(n1 >= n2))
    } else {
        Err(MalError::EvalError("'>=' expects numbers".to_string()))
    }
}

#[allow(clippy::unnecessary_wraps)]
fn prstr(args: Vec<MalType>) -> MalResult {
    Ok(MalType::String(
        args.into_iter()
            .map(|mal| printer::pr_str(mal, true))
            .collect::<Vec<_>>()
            .join(" "),
    ))
}

#[allow(clippy::unnecessary_wraps)]
fn str(args: Vec<MalType>) -> MalResult {
    Ok(MalType::String(
        args.into_iter()
            .map(|mal| printer::pr_str(mal, false))
            .collect::<String>(),
    ))
}

#[allow(clippy::unnecessary_wraps)]
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
