use std::collections::HashMap;

use crate::ast::Expression;
use crate::{ast::Statement, errors::error, token::Token};

#[derive(Debug)]
pub enum Object<'a> {
    Integer(isize),
    Float(f64),
    Boolean(bool),
    String(String),
    Underscore,
    Null,
    Array(Vec<&'a Object<'a>>),
    Function {
        parameters: Vec<Token<'a>>,
        body: Statement<'a>,
        context: &'a mut Context<'a>,
    },
    Reference {
        value: &'a Object<'a>,
        to: Box<Expression<'a>>,
    },
    ReturnValue {
        value: &'a Object<'a>,
    },
}
#[derive(Debug)]
pub struct Module<'a> {
    pub functions: HashMap<String, &'a Object<'a>>,
}

#[derive(Debug)]
pub struct Context<'a> {
    pub store: HashMap<String, &'a Object<'a>>,
    pub outer: Option<&'a mut Context<'a>>,
    pub modules: HashMap<String, Module<'a>>,
}

impl<'a> Context<'a> {
    pub fn new(outer: Option<&'a mut Context<'a>>) -> Context {
        let store: HashMap<String, &'a Object<'a>> = HashMap::new();
        let modules: HashMap<String, Module<'a>> = HashMap::new();
        return Context {
            store,
            outer,
            modules,
        };
    }

    pub fn get(&self, key: &str) -> Option<&Object<'a>> {
        let x = self.store.get(key);
        if let Some(obj) = x {
            return Some(obj);
        }

        if let Some(outer) = &self.outer {
            return outer.get(key);
        }

        return None;
    }

    pub fn set(&mut self, key: &str, value: &'a Object<'a>) -> &'a Object<'a> {
        self.store.insert(key.to_owned(), value);
        return value;
    }

    pub fn update(&mut self, key: &str, value: &'a Object<'a>) -> &'a Object<'a> {
        let x = self.store.get(key);

        if let Some(object) = x {
            self.store.insert(key.to_owned(), value);
        }

        if let Some(outer) = &mut self.outer {
            outer.update(key, value);
        } else {
            error(format!(
                "Could not find identifier with name '{}' in current context.",
                key
            ));
            panic!();
        }

        return value;
    }
}
