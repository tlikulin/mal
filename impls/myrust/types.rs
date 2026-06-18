use std::collections::HashMap;
use std::io;
use std::{cell::RefCell, rc::Rc};

use crate::env::Env;

pub type MalResult = Result<MalType, MalError>;

#[derive(Clone, Default)]
pub enum MalType {
    #[default]
    Nil,
    Bool(bool),
    Number(i64),
    Symbol(String),
    String(String),
    Keyword(String),
    List(Vec<Self>, Option<Box<Self>>),
    Vector(Vec<Self>, Option<Box<Self>>),
    HashMap(HashMap<String, Self>, Option<Box<Self>>),
    BuiltinFunc(Rc<dyn Fn(Vec<Self>) -> MalResult>, Option<Box<Self>>),
    Lambda {
        params: Vec<Self>,
        body: Box<Self>,
        capt_env: Env,
        is_macro: bool,
        meta: Option<Box<Self>>,
    },
    Atom(Rc<RefCell<Self>>),
}

// helper functions for less typing
pub const fn new_list(list: Vec<MalType>) -> MalType {
    MalType::List(list, None)
}
pub const fn new_vector(vector: Vec<MalType>) -> MalType {
    MalType::Vector(vector, None)
}
pub const fn new_hashmap(hashmap: HashMap<String, MalType>) -> MalType {
    MalType::HashMap(hashmap, None)
}

impl MalType {
    pub const fn to_bool(&self) -> bool {
        !matches!(&self, Self::Nil | Self::Bool(false))
    }

    pub fn to_key(&self) -> Result<&str, MalError> {
        match self {
            Self::String(key) | Self::Keyword(key) => Ok(key),
            _ => Err(MalError::ParseError(
                "hash-map key not hashable".to_string(),
            )),
        }
    }

    pub fn is_list_with_sym(&self, sym: &str) -> bool {
        if let Self::List(list, ..) = self {
            matches!(list.first(), Some(Self::Symbol(symbol)) if symbol == sym)
        } else {
            false
        }
    }

    pub const fn set_macro(&mut self) {
        if let Self::Lambda { is_macro, .. } = self {
            *is_macro = true;
        }
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0))
            | (Self::String(l0), Self::String(r0))
            | (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (
                Self::List(l0, ..) | Self::Vector(l0, ..),
                Self::List(r0, ..) | Self::Vector(r0, ..),
            ) => l0 == r0,
            (Self::HashMap(l0, ..), Self::HashMap(r0, ..)) => l0 == r0,
            (Self::Atom(l0), Self::Atom(r0)) => Rc::ptr_eq(l0, r0),
            // all functions are incomparable
            _ => false,
        }
    }
}

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
