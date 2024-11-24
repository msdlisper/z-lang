use std::cell::RefCell;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::convert::Into;
use std::fmt::Debug;
use std::{collections::HashMap, rc::Rc};

use super::parse_ast::AstNodeType;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum ValType {
    String(String),
    Number(i32),
    Boolean(bool),
    Closure {
        // 定义函数时, 当时的作用域
        scope: Rc<RefCell<Scope>>,
        block: Rc<AstNodeType>,
        name: String,
        // AstNodeType里面的Declaration
        args: Vec<AstNodeType>,
    },
}

impl Into<i32> for ValType {
    fn into(self) -> i32 {
        match self {
            ValType::Number(num) => num,
            _ => panic!("Cannot convert ValType to the target type"),
        }
    }
}

impl PartialEq for ValType {
    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s != o,
            _ => false,
        }
    }
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s == o,
            _ => false,
        }
    }
}

impl PartialOrd for ValType {
    fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s > o,
            _ => false,
        }
    }

    fn ge(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s >= o,
            _ => false,
        }
    }

    fn le(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s <= o,
            _ => false,
        }
    }

    fn lt(&self, other: &Self) -> bool {
        match (self, other) {
            (ValType::Number(s), ValType::Number(o)) => s < o,
            _ => false,
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

#[derive(Debug)]
pub struct Frame {
    pub parent_frame: Option<Rc<RefCell<Frame>>>,
    pub scope: Rc<RefCell<Scope>>,
}


impl Frame {
    pub fn contains_key(&self, q: &String) -> bool {
        self.scope.borrow().contains_key(q)
    }

    pub fn get(&self, q: &String) -> Option<Rc<ValType>> {
        self.scope.borrow().get(q)
    }

    pub fn set(&mut self, q: String, val: Rc<ValType>, is_declare: bool) -> bool {
        self.scope.borrow_mut().set(q, val, is_declare);
        true
    }

    pub fn create_scope(&mut self, new_scope: Rc<RefCell<Scope>>) -> bool {
        new_scope.borrow_mut().set_parent(self.scope.clone());
        self.scope = new_scope;
        true
    }

    pub fn drop_scope(&mut self) -> bool {
        let parent = self.scope.borrow().get_parent_scope();
        match parent {
            Some(p) => {
                self.scope = p;
            },
            None => {
                // 没有上一级了
                return false
            }
        }

        
        true
    }
}

/**
 * 作用域
 */
#[derive(Debug)]
pub enum Scope {
    Closure(ScopeStruct),
    Block(ScopeStruct),
}

/**
 * 块级作用域
 */
#[derive(Debug)]
pub struct ScopeStruct {
    pub play_object: HashMap<String, Rc<ValType>>,
    pub parent_scope: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    fn get(&self, q: &String) -> Option<Rc<ValType>> {
        match self {
            Scope::Closure(scope_struct) | Scope::Block(scope_struct) => scope_struct.get(q),
        }
    }
    fn contains_key(&self, q: &String) -> bool {
        match self {
            Scope::Closure(scope_struct) | Scope::Block(scope_struct) => {
                scope_struct.contains_key(q)
            }
        }
    }
    fn get_parent_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        match self {
            Scope::Closure(scope_struct) | Scope::Block(scope_struct) => {
                scope_struct.get_parent_scope()
            }
        }
    }
    fn set(&mut self, q: String, val: Rc<ValType>, is_declare: bool) -> bool {
        match self {
            Scope::Closure(scope_struct) | Scope::Block(scope_struct) => scope_struct.set(q, val, is_declare),
        }
    }
    fn set_parent(&mut self, parent: Rc<RefCell<Scope>>) -> bool {
        match self {
            Scope::Closure(scope_struct) | Scope::Block(scope_struct) => {
                scope_struct.set_parent(parent)
            }
        }
    }

    // fn drop(&mut self) -> bool {
    //     match self {
    //         Scope::Closure(scope_struct) | Scope::Block(scope_struct) => {
    //             scope_struct.drop()
    //         }
    //     }
    // }
}

impl ScopeStruct {
    fn new(parent_scope: Option<Rc<RefCell<Scope>>>) -> Self
    where
        Self: Sized,
    {
        ScopeStruct {
            play_object: HashMap::new(),
            parent_scope,
        }
    }

    fn get(&self, q: &String) -> Option<Rc<ValType>> {
        match self.play_object.get(q) {
            Some(val) => return Some(val.clone()),
            None => {
                // 没有的话就从父级找
                let val = match self.get_parent_scope() {
                    Some(parent) => {
                        return parent.borrow().get(q);
                    },
                    None => None,
                };
                val
            },
        }

    }

    fn get_parent_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent_scope.clone()
    }

    fn set(&mut self, q: String, val: Rc<ValType>, is_declare: bool) -> bool {
        if is_declare {
            self.play_object.insert(q, val);
            return true;
        }
        match self.play_object.contains_key(&q) {
            true => {
                self.play_object.insert(q, val);
                return true
            },
            false => {
                // 没有的话就从父级找
                 match self.get_parent_scope() {
                    Some(parent) => {
                        return parent.borrow_mut().set(q, val, false);
                    },
                    None => return false,
                }
            },
        }
    }

    fn contains_key(&self, q: &String) -> bool {
        match self.play_object.contains_key(q) {
            true => return true,
            false => {
                // 没有的话就从父级找
                 match self.get_parent_scope() {
                    Some(parent) => {
                        return parent.borrow().contains_key(q);
                    },
                    None => return false,
                }
            },
        }
    }

    fn set_parent(&mut self, parent: Rc<RefCell<Scope>>) -> bool {
        self.parent_scope = Some(parent);
        true
    }

}

