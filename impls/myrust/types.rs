use std::{collections::HashMap, fmt::Debug, rc::Rc};

pub type Callable = Rc<dyn Fn(Vec<MalType>) -> MalResult>;

#[derive(Clone, Default)]
pub enum MalType {
    Number(i32),
    Symbol(String),
    List(Vec<Self>),
    #[default]
    Nil,
    Bool(bool),
    String(String),
    Keyword(String),
    Vector(Vec<Self>),
    HashMap(HashMap<String, Self>),
    Function(Callable),
}

impl MalType {
    pub fn into_callable(self) -> Option<Callable> {
        match self {
            Self::Function(func) => Some(func),
            _ => None,
        }
    }

    pub fn into_number(self) -> Option<i32> {
        match self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }
}

impl Debug for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Nil => write!(f, "Nil"),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::HashMap(arg0) => f.debug_tuple("HashMap").field(arg0).finish(),
            Self::Function(_) => f.debug_tuple("BuiltinFunc").field(&"<function>").finish(),
        }
    }
}

pub enum MalError {
    EmptyInput,
    ParseError(String),
    EvalError(String),
}

pub type MalResult = Result<MalType, MalError>;
