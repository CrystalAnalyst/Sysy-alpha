#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::{parser::Node, BasicType, NodeType};
use colored::Colorize;
use std::{collections::HashMap, fs::File, path::Path, usize};

static mut FILEPATH: String = String::new();

#[derive(Clone)]
pub struct Var {
    btype: BasicType,
    node: Node,
}

impl Var {
    pub fn new(btype: BasicType, node: Node) -> Self {
        Var { btype, node }
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

    ///将node节点(代表变量或者函数)新增到全局表或者当前作用域中。
    fn insert(&mut self, name: String, btype: BasicType, node: Node) {
        // step1.检查定义过的变量
        if matches!(node.node_type, NodeType::Decl(..)) {
            if self.local.is_empty() {
                if let Some(val) = self.global.get(&name) {
                    if matches!(val.node.node_type, NodeType::Decl(..)) {
                        //错误处理：该变量/函数已经全局定义过.
                    }
                }
            } else {
                if self.local.last().unwrap().contains_key(&name) {
                    // 错误处理: 该变量/函数已经局部定义过.
                }
            }
        }
        // step2.插入全局或者当前作用域
        if self.local.is_empty() || matches!(node.node_type, NodeType::Func(..)) {
            self.global.insert(name, Var::new(btype, node));
        } else {
            self.local
                .last_mut()
                .unwrap()
                .insert(name, Var::new(btype, node));
        }
    }

    //todo: fn find()
    fn find(&self, name: &String, node: &Node) -> Var {
        // step1. 从当前局部作用域往回查找
        for map in self.local.iter().rev() {
            if let Some(var) = map.get(name) {
                return var.clone();
            }
        }
        // step2. 在全局作用域中查找
        if let Some(var) = self.global.get(name) {
            return var.clone();
        } else {
            //处理错误: 该函数/变量尚未定义过
            unreachable!()
        }
    }
}

impl Node {
    fn error_spot(&self, msg: String) {
        let code = String::new();

        let code_chars: Vec<char> = code.chars().collect();
        let mut line_start = self.startpos;
        while line_start != 0 && code_chars[line_start] != '\n' {
            line_start -= 1;
        }
        let mut line_end = self.endpos;
        while line_end != code.len() && code_chars[line_end] != '\n' {
            line_end += 1;
        }

        let mut start_line = 1;
        let mut index = 0;
        while index != line_start {
            if code_chars[index] == '\n' {
                start_line += 1;
            }
            index += 1;
        }

        let code_lines = code[line_start..line_end].to_string();
        let mut sign_lines = String::new();
        for i in line_start..line_end {
            if code_chars[i] == '\n' {
                sign_lines.push('\n');
                continue;
            }
            if self.startpos <= i && i < self.endpos {
                sign_lines.push('^');
            } else {
                sign_lines.push(' ');
            }
        }
        //Error message
        println!("{}: {}", "sementic error".red().bold(), msg.bold());
        println!(
            "  {} {}:{}",
            "-->".blue().bold(),
            start_line + 1,
            self.startpos - line_start
        );
        for (i, (code_line, sign_line)) in code_lines
            .split('\n')
            .into_iter()
            .zip(sign_lines.split('\n').into_iter())
            .enumerate()
        {
            if code_line.trim().is_empty() {
                continue;
            }
            println!("     {}", "|".blue().bold());
            println!(
                "  {3:3}{2} {}\n     {2} {}\n",
                code_line,
                sign_line.red().bold(),
                "|".blue().bold(),
                (start_line + i).to_string().blue().bold()
            );
        }
        //panic!("{}", msg);
    }
}
