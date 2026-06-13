use std::{cell::RefCell, collections::HashMap, fmt::Debug, io, rc::Rc};

use crate::env::Env;

#[derive(Clone, Default)]
pub enum MalType {
    Number(i64),
    Symbol(String),
    List(Vec<Self>),
    #[default]
    Nil,
    Bool(bool),
    String(String),
    Keyword(String),
    Vector(Vec<Self>),
    HashMap(HashMap<String, Self>),
    BuiltinFunc(Rc<dyn Fn(Vec<Self>) -> MalResult>),
    Lambda {
        params: Vec<Self>,
        body: Box<Self>,
        capt_env: Env,
    },
    Atom(Rc<RefCell<Self>>),
}

impl MalType {
    pub const fn to_bool(&self) -> bool {
        !matches!(&self, Self::Nil | Self::Bool(false))
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0))
            | (Self::String(l0), Self::String(r0))
            | (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::List(l0) | Self::Vector(l0), Self::List(r0) | Self::Vector(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::HashMap(l0), Self::HashMap(r0)) => l0 == r0,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl Debug for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{mal}}")
    }
}

#[derive(Debug)]
pub enum MalError {
    EmptyInput,
    ParseError(String),
    EvalError(String),
    IOError(io::Error),
}

impl From<io::Error> for MalError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

pub type MalResult = Result<MalType, MalError>;
