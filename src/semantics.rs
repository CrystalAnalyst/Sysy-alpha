use std::{collections::HashMap, fs::File, io::Read, path::Path, usize};

use colored::Colorize;

use crate::{builtin_funcs, parse::Node, BasicType, NodeType, Scope, TokenType};

static mut FILEPATH: String = String::new();

#[derive(clone)]
pub struct Var {
    varType: BasicType,
    node: Node,
}
impl Var {
    pub fn new(varType: BasicType, node: Node) -> Self {
        Var { varType, node }
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
}
