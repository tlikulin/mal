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
        is_macro: bool,
    },
    Atom(Rc<RefCell<Self>>),
}

impl MalType {
    pub const fn to_bool(&self) -> bool {
        !matches!(&self, Self::Nil | Self::Bool(false))
    }

    pub fn is_list_with_sym(&self, sym: &str) -> bool {
        if let Self::List(list) = self {
            matches!(list.first(), Some(Self::Symbol(symbol)) if symbol == sym)
        } else {
            false
        }
    }

    pub const fn is_vector(&self) -> bool {
        matches!(self, Self::Vector(..))
    }

    pub const fn set_macro(&mut self) {
        if let Self::Lambda { is_macro, .. } = self {
            *is_macro = true;
        }
    }

    pub fn get_first(self) -> Option<Self> {
        match self {
            Self::Nil => Some(Self::Nil),
            Self::List(list) | Self::Vector(list) if list.is_empty() => Some(Self::Nil),
            Self::List(mut list) | Self::Vector(mut list) => Some(list.swap_remove(0)),
            _ => None,
        }
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
            (Self::Atom(l0), Self::Atom(r0)) => Rc::ptr_eq(l0, r0),
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
    Exception(MalType),
}

impl From<io::Error> for MalError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl MalError {
    pub fn into_mal(self) -> MalType {
        match self {
            Self::EmptyInput => MalType::String("no input".to_string()),
            Self::ParseError(msg) | Self::EvalError(msg) => MalType::String(msg),
            Self::IOError(e) => MalType::String(e.to_string()),
            Self::Exception(mal) => mal,
        }
    }
}

pub type MalResult = Result<MalType, MalError>;
