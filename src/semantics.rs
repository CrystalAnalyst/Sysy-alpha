use std::{collections::HashMap, usize};

use crate::{parser::Node, BasicType};

static mut FILEPATH: String = String::new();

#[derive(Clone)]
pub struct Var {
    vartype: BasicType,
    node: Node,
}
impl Var {
    pub fn new(vartype: BasicType, node: Node) -> Self {
        Var { vartype, node }
    }
}

pub struct Runtime {
    global: HashMap<String, Var>,
    local: Vec<HashMap<String, Var>>,
    loop_count: usize,
    cur_func_name: String,
    cur_func_type: BasicType,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            global: HashMap::new(),
            local: vec![],
            loop_count: 0,
            cur_func_name: String::new(),
            cur_func_type: BasicType::Nil,
        }
    }

    fn enter_scope(&mut self) {
        self.local.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.local.pop();
    }

    //todo: fn insert()

    //todo: fn find()

    fn start_loop(&mut self) {
        self.loop_count += 1;
    }

    fn end_loop(&mut self) {
        self.loop_count -= 1;
    }

    fn is_in_loop(&mut self) -> bool {
        if self.loop_count == 0 {
            return false;
        } else if self.loop_count > 0 {
            return true;
        } else {
            unreachable!()
        }
    }

    fn set_cur_func(&mut self, func_name: &String, func_type: &BasicType) {
        self.cur_func_name = func_name.clone();
        self.cur_func_type = func_type.clone();
    }

    fn get_cur_func(&mut self) -> (String, BasicType) {
        return (self.cur_func_name.clone(), self.cur_func_type.clone());
    }
}
