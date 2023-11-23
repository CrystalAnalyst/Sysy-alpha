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
