use std::{collections::HashMap, rc::Rc};

use crate::types::{MalError, MalResult, MalType};

pub struct Env(Rc<EnvInner>);

impl Env {
    pub fn new(outer: Option<Self>) -> Self {
        Self(Rc::new(EnvInner {
            data: HashMap::new(),
            outer,
        }))
    }

    pub fn set(&mut self, key: String, value: MalType) {
        Rc::get_mut(&mut self.0).unwrap().data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> MalResult {
        self.0.data.get(key).map_or_else(
            || {
                self.0.outer.as_ref().map_or_else(
                    || Err(MalError::EvalError(format!("'{key}' not found"))),
                    |outer| outer.get(key),
                )
            },
            |mal| Ok(mal.clone()),
        )
    }

    pub fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

struct EnvInner {
    data: HashMap<String, MalType>,
    outer: Option<Env>,
}
