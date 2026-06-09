use std::collections::HashMap;

#[derive(Debug)]
pub enum MalType {
    Number(i32),
    Symbol(String),
    List(Vec<MalType>),
    Nil,
    Bool(bool),
    String(String),
    Keyword(String),
    Vector(Vec<MalType>),
    HashMap(HashMap<String, MalType>),
}
