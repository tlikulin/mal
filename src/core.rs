#![allow(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    reason = "builtins must conform to same fn signature"
)]

use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, rc::Rc};

use crate::env::Env;
use crate::printer;
use crate::reader::{self, KW_SUFFIX};
use crate::types::MalError::{EvalError, Exception, ParseError};
use crate::types::{MalResult, MalType};
use crate::types::{new_hashmap, new_list, new_vector};

pub fn ns() -> Vec<(&'static str, MalType)> {
    vec![
        ("*host-language*", MalType::String("rust".to_string())),
        ("*ARGV*", new_list(vec![])),
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
        ("throw", builtin_fn(throw)),
        ("apply", builtin_fn(apply)),
        ("map", builtin_fn(map)),
        ("nil?", builtin_fn(is_nil)),
        ("true?", builtin_fn(is_true)),
        ("false?", builtin_fn(is_false)),
        ("symbol?", builtin_fn(is_symbol)),
        ("symbol", builtin_fn(symbol)),
        ("keyword", builtin_fn(keyword)),
        ("keyword?", builtin_fn(is_keyword)),
        ("vector", builtin_fn(vector)),
        ("vector?", builtin_fn(is_vector)),
        ("sequential?", builtin_fn(is_sequential)),
        ("hash-map", builtin_fn(hashmap)),
        ("map?", builtin_fn(is_map)),
        ("assoc", builtin_fn(assoc)),
        ("dissoc", builtin_fn(dissoc)),
        ("get", builtin_fn(get)),
        ("contains?", builtin_fn(contains)),
        ("keys", builtin_fn(keys)),
        ("vals", builtin_fn(vals)),
        ("readline", builtin_fn(readline)),
        ("time-ms", builtin_fn(timems)),
        ("meta", builtin_fn(meta)),
        ("with-meta", builtin_fn(withmeta)),
        ("fn?", builtin_fn(is_fn)),
        ("string?", builtin_fn(is_string)),
        ("number?", builtin_fn(is_number)),
        ("seq", builtin_fn(seq)),
        ("conj", builtin_fn(conj)),
    ]
}

// helpers

fn builtin_fn<F: Fn(Vec<MalType>) -> MalResult + 'static>(func: F) -> MalType {
    MalType::BuiltinFunc(Rc::new(func), None)
}

fn arithmetic(mut args: Vec<MalType>, calc: impl Fn(i64, i64) -> i64, op: char) -> MalResult {
    if args.len() < 2 {
        Err(EvalError(format!("too few args to '{op}'")))
    } else if let (MalType::Number(b), MalType::Number(a)) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        Ok(MalType::Number(calc(a, b)))
    } else {
        Err(EvalError(format!("'{op}' expects numbers")))
    }
}

fn numeric_cmp(args: Vec<MalType>, cmp: impl Fn(&i64, &i64) -> bool, op: &str) -> MalResult {
    if args.len() < 2 {
        Err(EvalError(format!("too few args to '{op}'")))
    } else if let (MalType::Number(a), MalType::Number(b)) = (&args[0], &args[1]) {
        Ok(MalType::Bool(cmp(a, b)))
    } else {
        Err(EvalError(format!("'{op}' expects 2 numbers")))
    }
}

// actual builtins

fn add(args: Vec<MalType>) -> MalResult {
    arithmetic(args, |a, b| a + b, '+')
}

fn sub(args: Vec<MalType>) -> MalResult {
    arithmetic(args, |a, b| a - b, '-')
}

fn mult(args: Vec<MalType>) -> MalResult {
    arithmetic(args, |a, b| a * b, '*')
}

fn div(args: Vec<MalType>) -> MalResult {
    arithmetic(args, |a, b| a / b, '/')
}

fn prn(args: Vec<MalType>) -> MalResult {
    let Ok(MalType::String(s)) = prstr(args) else {
        unreachable!()
    };
    println!("{s}");
    Ok(MalType::Nil)
}

const fn list(args: Vec<MalType>) -> MalResult {
    Ok(new_list(args))
}

fn is_list(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'list?'".to_string()))
    } else {
        Ok(MalType::Bool(matches!(args[0], MalType::List(..))))
    }
}

fn is_empty(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'empty?'".to_string()))
    } else if let MalType::List(seq, ..) | MalType::Vector(seq, ..) = &args[0] {
        Ok(MalType::Bool(seq.is_empty()))
    } else {
        Err(EvalError("'empty?' expects list/vector".to_string()))
    }
}

fn count(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'count'".to_string()))
    } else if let MalType::List(seq, ..) | MalType::Vector(seq, ..) = &args[0] {
        Ok(MalType::Number(seq.len().try_into().unwrap()))
    } else if matches!(&args[0], MalType::Nil) {
        Ok(MalType::Number(0))
    } else {
        Err(EvalError("'count' expects list/vector/nil".to_string()))
    }
}

fn eq(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(EvalError("too few args to '='".to_string()))
    } else {
        Ok(MalType::Bool(args[0] == args[1]))
    }
}

fn lt(args: Vec<MalType>) -> MalResult {
    numeric_cmp(args, |a, b| a < b, "<")
}

fn le(args: Vec<MalType>) -> MalResult {
    numeric_cmp(args, |a, b| a <= b, "<=")
}

fn gt(args: Vec<MalType>) -> MalResult {
    numeric_cmp(args, |a, b| a > b, ">")
}

fn ge(args: Vec<MalType>) -> MalResult {
    numeric_cmp(args, |a, b| a >= b, ">=")
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
            .collect::<Vec<String>>()
            .join(" ")
    );
    Ok(MalType::Nil)
}

fn read_string(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::String(string)) = args.first() {
        reader::read_str(string)
    } else {
        Err(EvalError("'read_string' expects string".to_string()))
    }
}

fn slurp(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::String(filename)) = args.first() {
        let contents = fs::read_to_string(filename)?;
        Ok(MalType::String(contents))
    } else {
        Err(EvalError("'slurp' expects string".to_string()))
    }
}

fn atom(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("'atom' expects value".to_string()))
    } else {
        Ok(MalType::Atom(Rc::new(RefCell::new(args.swap_remove(0)))))
    }
}

fn is_atom(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Atom(..))
    )))
}

fn deref(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::Atom(atom)) = args.first() {
        Ok(atom.borrow().clone())
    } else {
        Err(EvalError("'deref' expects an atom".to_string()))
    }
}

fn reset(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(MalType::Atom(atom)), Some(mal)) = (it.next(), it.next()) {
        atom.replace(mal.clone());
        Ok(mal)
    } else {
        Err(EvalError("'reset!' expects atom and value".to_string()))
    }
}

fn swap(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(MalType::Atom(atom)), Some(func)) = (it.next(), it.next()) {
        let mut fn_args = vec![atom.borrow().clone()];
        fn_args.extend(it);

        let new_value = match func {
            MalType::BuiltinFunc(builtin, ..) => builtin(fn_args),
            MalType::Lambda {
                params,
                body,
                capt_env,
                ..
            } => {
                let lambda_env = Env::new(Some(capt_env), params, fn_args)?;
                crate::eval(*body, &lambda_env)
            }
            _ => Err(EvalError("not callable".to_string())),
        }?;

        atom.replace(new_value.clone());
        Ok(new_value)
    } else {
        Err(EvalError(
            "'swap!' expects atom, func (and args)".to_string(),
        ))
    }
}

fn cons(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (Some(first), Some(MalType::List(mut seq, ..) | MalType::Vector(mut seq, ..))) =
        (it.next(), it.next())
    {
        seq.insert(0, first);
        Ok(new_list(seq))
    } else {
        Err(EvalError(
            "'cons' expects value and list/vector".to_string(),
        ))
    }
}

fn concat(args: Vec<MalType>) -> MalResult {
    let mut result = Vec::new();
    for arg in args {
        let (MalType::List(seq, ..) | MalType::Vector(seq, ..)) = arg else {
            return Err(EvalError("'concat' expects lists/vectors".to_string()));
        };

        result.extend(seq);
    }
    Ok(new_list(result))
}

fn vec(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("'vec' expects list/vector".to_string()))
    } else {
        match args.swap_remove(0) {
            MalType::List(list, ..) => Ok(new_vector(list)),
            vector @ MalType::Vector(..) => Ok(vector),
            _ => Err(EvalError("'vec' expects list/vector".to_string())),
        }
    }
}

fn nth(args: Vec<MalType>) -> MalResult {
    let mut it = args.into_iter();
    if let (
        Some(MalType::List(mut seq, ..) | MalType::Vector(mut seq, ..)),
        Some(MalType::Number(ind)),
    ) = (it.next(), it.next())
    {
        match ind.try_into() {
            Ok(ind) if ind < seq.len() => Ok(seq.swap_remove(ind)),
            _ => Err(EvalError("out of bounds".to_string())),
        }
    } else {
        Err(EvalError(
            "'nth' expects list/vector and number".to_string(),
        ))
    }
}

fn first(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("'first' expects list/vector/nil".to_string()))
    } else {
        match args.swap_remove(0) {
            MalType::Nil => Ok(MalType::Nil),
            MalType::List(seq, ..) | MalType::Vector(seq, ..) if seq.is_empty() => Ok(MalType::Nil),
            MalType::List(mut seq, ..) | MalType::Vector(mut seq, ..) => Ok(seq.swap_remove(0)),
            _ => Err(EvalError("'first' expects list/vector/nil".to_string())),
        }
    }
}

fn rest(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("'rest' expects list/vector/nil".to_string()))
    } else {
        match args.swap_remove(0) {
            MalType::Nil => Ok(new_list(vec![])),
            MalType::List(list, ..) | MalType::Vector(list, ..) if list.is_empty() => {
                Ok(new_list(vec![]))
            }
            MalType::List(mut list, ..) | MalType::Vector(mut list, ..) => {
                list.remove(0);
                Ok(new_list(list))
            }
            _ => Err(EvalError("'rest' expects list/vector/nil".to_string())),
        }
    }
}

fn is_macro(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Lambda { is_macro: true, .. })
    )))
}

fn throw(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("'throw' expects arg".to_string()))
    } else {
        Err(Exception(args.swap_remove(0)))
    }
}

fn apply(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(EvalError("too few args to 'apply'".to_string()))
    } else if let Some(MalType::List(seq, ..) | MalType::Vector(seq, ..)) = args.pop() {
        let func = args.remove(0);
        args.extend(seq);

        match func {
            MalType::BuiltinFunc(builtin, ..) => builtin(args),
            MalType::Lambda {
                params,
                body,
                capt_env,
                ..
            } => {
                let lambda_env = Env::new(Some(capt_env), params, args)?;
                crate::eval(*body, &lambda_env)
            }
            _ => Err(EvalError("not callable".to_string())),
        }
    } else {
        Err(EvalError(
            "'apply' expects list/vector as last arg".to_string(),
        ))
    }
}

fn map(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(EvalError("too few args to 'map'".to_string()))
    } else if let (MalType::List(seq, ..) | MalType::Vector(seq, ..), func) =
        (args.swap_remove(1), args.swap_remove(0))
    {
        match func {
            MalType::BuiltinFunc(builtin, ..) => {
                let mut result = Vec::new();
                for elt in seq {
                    result.push(builtin(vec![elt])?);
                }
                Ok(new_list(result))
            }
            MalType::Lambda {
                params,
                body,
                capt_env,
                ..
            } => {
                let mut result = Vec::new();
                for elt in seq {
                    let lambda_env = Env::new(Some(capt_env.clone()), params.clone(), vec![elt])?;
                    result.push(crate::eval(*body.clone(), &lambda_env)?);
                }
                Ok(new_list(result))
            }
            _ => Err(EvalError("not callable".to_string())),
        }
    } else {
        Err(EvalError(
            "'map' expects list/vector as 2nd arg".to_string(),
        ))
    }
}

fn is_nil(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(args.first(), Some(MalType::Nil))))
}

fn is_true(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Bool(true))
    )))
}

fn is_false(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Bool(false))
    )))
}

fn is_symbol(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Symbol(..))
    )))
}

fn symbol(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'symbol'".to_string()))
    } else if let MalType::String(sym) = args.swap_remove(0) {
        Ok(MalType::Symbol(sym))
    } else {
        Err(EvalError("'symbol' expects string".to_string()))
    }
}

fn keyword(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'keyword'".to_string()))
    } else {
        match args.swap_remove(0) {
            key @ MalType::Keyword(..) => Ok(key),
            MalType::String(key) => Ok(MalType::Keyword(format!(":{key}{KW_SUFFIX}"))),
            _ => Err(EvalError("'keyword' expects string".to_string())),
        }
    }
}

fn is_keyword(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Keyword(..))
    )))
}

const fn vector(args: Vec<MalType>) -> MalResult {
    Ok(new_vector(args))
}

fn is_vector(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Vector(..))
    )))
}

fn is_sequential(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Vector(..) | MalType::List(..))
    )))
}

fn hashmap(args: Vec<MalType>) -> MalResult {
    reader::read_map(args).map_err(|e| match e {
        ParseError(msg) => EvalError(msg),
        _ => e,
    })
}

fn is_map(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::HashMap(..))
    )))
}

fn assoc(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'assoc'".to_string()))
    } else if let MalType::HashMap(mut orig_map, ..) = args.remove(0) {
        let MalType::HashMap(extra_map, ..) = hashmap(args)? else {
            unreachable!()
        };
        orig_map.extend(extra_map);
        Ok(new_hashmap(orig_map))
    } else {
        Err(EvalError("'assoc' expects map (and args)".to_string()))
    }
}

fn dissoc(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'dissoc'".to_string()))
    } else if let MalType::HashMap(mut map, ..) = args.remove(0) {
        for elt in &args {
            map.remove(elt.to_key()?);
        }
        Ok(new_hashmap(map))
    } else {
        Err(EvalError("'dissoc' expects map (and args)".to_string()))
    }
}

fn get(mut args: Vec<MalType>) -> MalResult {
    if let [MalType::HashMap(map, ..), raw_key] = &mut args[..2] {
        Ok(map.remove(raw_key.to_key()?).unwrap_or_default())
    } else if matches!(args.first(), Some(MalType::Nil)) {
        Ok(MalType::Nil)
    } else {
        Err(EvalError("'get' expects map and key".to_string()))
    }
}

fn contains(args: Vec<MalType>) -> MalResult {
    if let [MalType::HashMap(map, ..), raw_key] = &args[..2] {
        Ok(MalType::Bool(map.contains_key(raw_key.to_key()?)))
    } else {
        Err(EvalError("'contains?' expects map and key".to_string()))
    }
}

fn keys(mut args: Vec<MalType>) -> MalResult {
    if !args.is_empty()
        && let MalType::HashMap(map, ..) = args.swap_remove(0)
    {
        let keys = map.into_keys().map(|key| {
            if key.ends_with(KW_SUFFIX) {
                MalType::Keyword(key)
            } else {
                MalType::String(key)
            }
        });
        Ok(new_list(keys.collect()))
    } else {
        Err(EvalError("'keys' expects map".to_string()))
    }
}

fn vals(mut args: Vec<MalType>) -> MalResult {
    if !args.is_empty()
        && let MalType::HashMap(map, ..) = args.swap_remove(0)
    {
        Ok(new_list(map.into_values().collect()))
    } else {
        Err(EvalError("'vals' expects map".to_string()))
    }
}

fn readline(args: Vec<MalType>) -> MalResult {
    if let Some(MalType::String(msg)) = args.first() {
        print!("{msg}");
        io::stdout().flush()?;
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.is_empty() {
        println!();
        Ok(MalType::Nil)
    } else {
        if input.ends_with('\n') {
            input.pop();
        }
        Ok(MalType::String(input))
    }
}

fn timems(_args: Vec<MalType>) -> MalResult {
    Ok(MalType::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap(),
    ))
}

fn meta(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'meta'".to_string()))
    } else {
        match args.swap_remove(0) {
            MalType::List(_, None)
            | MalType::Vector(_, None)
            | MalType::HashMap(_, None)
            | MalType::BuiltinFunc(_, None)
            | MalType::Lambda { meta: None, .. } => Ok(MalType::Nil),

            MalType::List(_, Some(meta))
            | MalType::Vector(_, Some(meta))
            | MalType::HashMap(_, Some(meta))
            | MalType::BuiltinFunc(_, Some(meta))
            | MalType::Lambda {
                meta: Some(meta), ..
            } => Ok(*meta),

            _ => Err(EvalError("'meta' not supported for given type".to_string())),
        }
    }
}

fn withmeta(mut args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        Err(EvalError("too few args to 'with-meta'".to_string()))
    } else {
        let meta = Some(Box::new(args.swap_remove(1)));
        match args.swap_remove(0) {
            MalType::List(list, ..) => Ok(MalType::List(list, meta)),
            MalType::Vector(vector, ..) => Ok(MalType::Vector(vector, meta)),
            MalType::HashMap(hashmap, ..) => Ok(MalType::HashMap(hashmap, meta)),
            MalType::BuiltinFunc(func, ..) => Ok(MalType::BuiltinFunc(func, meta)),
            MalType::Lambda {
                params,
                body,
                capt_env,
                is_macro,
                ..
            } => Ok(MalType::Lambda {
                params,
                body,
                capt_env,
                is_macro,
                meta,
            }),
            _ => Err(EvalError(
                "'with-meta' not supported for given type".to_string(),
            )),
        }
    }
}

fn is_fn(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(
            MalType::BuiltinFunc(..)
                | MalType::Lambda {
                    is_macro: false,
                    ..
                }
        )
    )))
}

fn is_string(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::String(..))
    )))
}

fn is_number(args: Vec<MalType>) -> MalResult {
    Ok(MalType::Bool(matches!(
        args.first(),
        Some(MalType::Number(..))
    )))
}

fn seq(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'seq'".to_string()))
    } else {
        match args.swap_remove(0) {
            MalType::Nil => Ok(MalType::Nil),
            MalType::List(seq, ..) | MalType::Vector(seq, ..) if seq.is_empty() => Ok(MalType::Nil),
            MalType::String(string) if string.is_empty() => Ok(MalType::Nil),

            list @ MalType::List(..) => Ok(list),
            MalType::Vector(vector, ..) => Ok(new_list(vector)),
            MalType::String(string) => Ok(new_list(
                string
                    .chars()
                    .map(|c| MalType::String(c.to_string()))
                    .collect(),
            )),

            _ => Err(EvalError(
                "'seq' expects list/vector/string/nil".to_string(),
            )),
        }
    }
}

fn conj(mut args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        Err(EvalError("too few args to 'seq'".to_string()))
    } else {
        match args.remove(0) {
            MalType::Vector(mut vector, ..) => {
                vector.extend(args);
                Ok(new_vector(vector))
            }
            MalType::List(list, ..) => {
                args.reverse();
                args.extend(list);
                Ok(new_list(args))
            }
            _ => Err(EvalError(
                "'conj' expects list/vector and values".to_string(),
            )),
        }
    }
}
