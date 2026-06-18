use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::types::MalError::EvalError;
use crate::types::{MalError, MalResult, MalType, new_list};

#[derive(Clone)]
pub struct Env(Rc<RefCell<EnvInner>>);

impl Env {
    pub fn new(
        outer: Option<Self>,
        binds: Vec<MalType>,
        exprs: Vec<MalType>,
    ) -> Result<Self, MalError> {
        let new_env = Self(Rc::new(RefCell::new(EnvInner {
            data: HashMap::new(),
            outer,
        })));

        let mut it_binds = binds.into_iter();
        let mut it_exprs = exprs.into_iter();

        while let Some(bind) = it_binds.next() {
            match bind {
                MalType::Symbol(sym) if sym == "&" => {
                    let Some(MalType::Symbol(rest_sym)) = it_binds.next() else {
                        return Err(EvalError("no symbol after &".to_string()));
                    };

                    new_env.set(rest_sym, new_list(it_exprs.collect()));
                    break;
                }
                MalType::Symbol(sym) => {
                    let Some(expr) = it_exprs.next() else {
                        return Err(EvalError("more args expected".to_string()));
                    };

                    new_env.set(sym, expr);
                }
                _ => return Err(EvalError("bind is not a symbol".to_string())),
            }
        }

        Ok(new_env)
    }

    pub fn set(&self, key: String, value: MalType) {
        self.0.borrow_mut().data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> MalResult {
        self.0.borrow().data.get(key).map_or_else(
            || {
                self.0.borrow().outer.as_ref().map_or_else(
                    || Err(EvalError(format!("'{key}' not found"))),
                    |outer| outer.get(key),
                )
            },
            |mal| Ok(mal.clone()),
        )
    }
}

struct EnvInner {
    data: HashMap<String, MalType>,
    outer: Option<Env>,
}
