use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

        let (mut it_binds, mut it_exprs) = (binds.into_iter(), exprs.into_iter());

        while let Some(bind) = it_binds.next() {
            match bind {
                MalType::Symbol(sym) if sym == "&" => {
                    let Some(rest_bind) = it_binds.next() else {
                        return Err(MalError::EvalError("no bind after '&'".to_string()));
                    };
                    let MalType::Symbol(rest_sym) = rest_bind else {
                        return Err(MalError::EvalError("bind is not a symbol".to_string()));
                    };

                    new_env.set(rest_sym, new_list(it_exprs.collect()));
                    break;
                }
                MalType::Symbol(sym) => {
                    let Some(expr) = it_exprs.next() else {
                        return Err(MalError::EvalError("more args expected".to_string()));
                    };

                    new_env.set(sym, expr);
                }
                _ => return Err(MalError::EvalError("bind is not a symbol".to_string())),
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
                    || Err(MalError::EvalError(format!("'{key}' not found"))),
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
