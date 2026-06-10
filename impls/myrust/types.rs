use std::collections::HashMap;

#[derive(Debug)]
pub enum MalType {
    Number(i32),
    Symbol(String),
    List(Vec<Self>),
    Nil,
    Bool(bool),
    String(String),
    Keyword(String),
    Vector(Vec<Self>),
    HashMap(HashMap<String, Self>),
}

pub enum MalError {
    EmptyInput,
    ParseError(String),
}

pub type MalResult = Result<MalType, MalError>;
