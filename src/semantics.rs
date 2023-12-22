#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::{parser::Node, BasicType, NodeType, Scope, TokenType};
use colored::Colorize;
use std::{collections::HashMap, fs::File, path::Path, usize};

static mut FILEPATH: String = String::new();

#[derive(Clone)]
pub struct Var {
    basic_type: BasicType,
    node: Node,
}

impl Var {
    pub fn new(basic_type: BasicType, node: Node) -> Self {
        Var { basic_type, node }
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

    fn startpos_loop(&mut self) {
        self.loop_count += 1;
    }

    fn endpos_loop(&mut self) {
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
    fn insert(&mut self, name: String, basic_type: BasicType, node: Node) {
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
            self.global.insert(name, Var::new(basic_type, node));
        } else {
            self.local
                .last_mut()
                .unwrap()
                .insert(name, Var::new(basic_type, node));
        }
    }

    //todo: fn find()
    fn find(&self, name: &String, node: &Node) -> (BasicType, Node) {
        // step1. 从当前局部作用域往回查找
        for map in self.local.iter().rev() {
            if let Some(var) = map.get(name) {
                return (var.basic_type.clone(), var.node.clone());
            }
        }
        // step2. 在全局作用域中查找
        if let Some(var) = self.global.get(name) {
            return (var.basic_type.clone(), var.node.clone());
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
        let mut line_startpos = self.startpos;
        while line_startpos != 0 && code_chars[line_startpos] != '\n' {
            line_startpos -= 1;
        }
        let mut line_endpos = self.endpos;
        while line_endpos != code.len() && code_chars[line_endpos] != '\n' {
            line_endpos += 1;
        }

        let mut startpos_line = 1;
        let mut index = 0;
        while index != line_startpos {
            if code_chars[index] == '\n' {
                startpos_line += 1;
            }
            index += 1;
        }

        let code_lines = code[line_startpos..line_endpos].to_string();
        let mut sign_lines = String::new();
        for i in line_startpos..line_endpos {
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
            startpos_line + 1,
            self.startpos - line_startpos
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
                (startpos_line + i).to_string().blue().bold()
            );
        }
        //panic!("{}", msg);
    }
}

fn traverse(node: &Node, ctx: &mut Runtime) -> Node {
    /* params: node代表当前节点, ctx代表runtime环境 */
    /* 1. 遍历parser生成的AST树, 对AST上的每个Node进行语义检查 */
    /* 2. 语义分析的结果是新的AST树, 就是你要的Anonated AST, 经过语义检查后的AST. */
    use NodeType::*;
    match &node.node_type {
        /* control flow */
        Break => {
            if !ctx.is_in_loop() {
                node.error_spot(format!("Break should in a loop"));
            }
            node.clone() //返回带Break语义的节点
        }
        Continue => {
            if !ctx.is_in_loop() {
                node.error_spot(format!("Continue should in a loop"));
            }
            node.clone() //返回带Continue语义的节点
        }
        /* literal */
        Number(_) => {
            let mut new_node = node.clone();
            new_node.basic_type = BasicType::Const;
            new_node //返回Const语义的节点
        }

        /* declStmt, List of Decl. */
        DeclStmt(decls) => {
            // 用Vec![]来存储声明语句的结果。
            let mut new_node = vec![];
            for decl in decls {
                // 将每一条声明语句的结果处理后都存入Vec![]中,
                new_node.push(traverse(&decl, ctx));
            }
            Node::new(DeclStmt(new_node)) //返回DeclStmt语义的节点
        }

        /* variable, eg: int a[3][3] = {{1,2,3},{4,5,6},{7,8,9}}, local */
        Decl(basic_type, name, dims, inits, scope) => {
            let mut ty = basic_type.clone();
            // step1. 处理维度
            let new_dims = if let Some(dim) = dims {
                let mut new = vec![];
                let mut n = vec![];
                for dim_node in dim {
                    let result = eval(&dim_node, ctx);
                    if result <= 0 && !matches!(dim_node.node_type, NodeType::Nil) {
                        dim_node.error_spot(format!("Dimension of {} should > 0", name));
                    }
                    new.push(Node {
                        startpos: dim_node.startpos,
                        endpos: dim_node.endpos,
                        node_type: Number(result),
                        basic_type: BasicType::Const, // 这里的basic_type是Const, 因为数组的大小是常量√, 不管你是啥数组。
                    });
                    n.push(result as usize);
                }
                if ty == BasicType::Int || matches!(ty, BasicType::IntArray(_)) {
                    ty = BasicType::IntArray(n);
                } else if ty == BasicType::Const || matches!(ty, BasicType::ConstArray(_)) {
                    ty = BasicType::ConstArray(n);
                }
                Some(new)
            } else {
                None
            };

            // step2. 处理初始化列表
            let mut new_inits = vec![];
            if let Some(init_nodes) = inits {
                // 如果是一维初始化列表, 处理:
                if new_dims.is_none() && init_nodes.len() == 1 {
                    let mut new_node;
                    new_node = traverse(&init_nodes[0], ctx);
                    if basic_type == &BasicType::Const || scope == &Scope::Global {
                        new_node = Node {
                            startpos: init_nodes[0].startpos,
                            endpos: init_nodes[0].endpos,
                            node_type: Number(eval(&init_nodes[0], ctx)),
                            basic_type: BasicType::Const,
                        };
                    }
                    new_inits.push(new_node);
                } else if let Some(ref n_dims) = new_dims {
                    // 如果是多维初始化列表, 处理.
                    if scope == &Scope::Global {
                        new_inits = expand_inits(&n_dims, &init_nodes, true, ctx, 0);
                    } else {
                        new_inits = expand_inits(&n_dims, &init_nodes, false, ctx, 0);
                    }
                } else {
                    node.error_spot(format!("error_spot initializer for {}", name));
                    unreachable!()
                }
            }
            let n_inits = if new_inits.is_empty() {
                None
            } else {
                Some(new_inits)
            };
            // step3. 新声明节点推入作用域
            let new_node = Node::new(Decl(
                ty.clone(),
                name.clone(),
                new_dims,
                n_inits,
                scope.clone(),
            ));
            ctx.insert(name.clone(), ty, new_node.clone());
            new_node
        }

        /* 根据给定的名称和索引, 在环境中查找相应的变量或数组, 并根据结果作不同的处理. */
        /* 具体来说, 如果是变量, 则根据其基本类型的不同进行处理, 然后返回一个新节点,
           相对应的, 如果是数组, 则根据索引和数组维度的长度进行判断, 处理完后返回一个新节点.
           这些节点都是经过语义分析后初步带有类型,语义信息的 "Anotated AST Node".
        */
        Access(name, indexes, _) => {
            let (basic_type, n) = ctx.find(name, node);
            if let NodeType::Decl(_, _, _, _, _) = n.node_type {
                match &basic_type {
                    BasicType::Const => {
                        let num = eval(node, ctx);
                        let mut new_node = Node {
                            startpos: node.startpos,
                            endpos: node.endpos,
                            node_type: Number(num),
                            basic_type: BasicType::Const,
                        };
                        new_node.basic_type = BasicType::Const;
                        return new_node;
                    }
                    BasicType::Int => {
                        let mut nn = n.clone();
                        nn.basic_type = basic_type.clone();
                        Node {
                            startpos: node.startpos,
                            endpos: node.endpos,
                            node_type: Access(name.clone(), indexes.clone(), Box::new(nn)),
                            basic_type: BasicType::Int,
                        }
                    }
                    BasicType::IntArray(dims) | BasicType::ConstArray(dims) => {
                        if indexes.is_none() {
                            let mut nn = n.clone();
                            nn.basic_type = basic_type.clone();
                            return Node {
                                startpos: node.startpos,
                                endpos: node.endpos,
                                node_type: Access(name.clone(), None, Box::new(nn)),
                                basic_type: basic_type.clone(),
                            };
                        }
                        let mut new_indexes = vec![];
                        for index in indexes.as_ref().unwrap() {
                            let new_index = traverse(&index, ctx);
                            if new_index.basic_type != BasicType::Int
                                && new_index.basic_type != BasicType::Const
                            {
                                node.error_spot(format!(
                                    "Index of {} should be int or const",
                                    name
                                ));
                            }
                            new_indexes.push(new_index);
                        }
                        let dim_len = dims.len();
                        let index_len = new_indexes.len();
                        let bty = if matches!(&basic_type, BasicType::IntArray(_)) {
                            if index_len == dim_len {
                                BasicType::Int
                            } else {
                                let arr = dims[index_len..dim_len].to_vec();
                                BasicType::IntArray(arr)
                            }
                        } else {
                            if index_len == dim_len {
                                BasicType::Const
                            } else {
                                let arr = dims[index_len..dim_len].to_vec();
                                BasicType::ConstArray(arr)
                            }
                        };
                        let mut nn = n.clone();
                        nn.basic_type = basic_type.clone();
                        Node {
                            startpos: node.startpos,
                            endpos: node.endpos,
                            node_type: Access(name.clone(), Some(new_indexes), Box::new(nn)),
                            basic_type: bty,
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                node.error_spot(format!(
                    "{} cannot be accessed since it is a function",
                    name
                ));
                unreachable!()
            }
        }

        BinOp(ttype, lhs, rhs) => {
            let new_lhs = traverse(&lhs, ctx);
            if new_lhs.basic_type != BasicType::Int && new_lhs.basic_type != BasicType::Const {
                lhs.error_spot(format!(
                    "Expression at the left of the operator should be int or const"
                ));
            }
            let new_rhs = traverse(&rhs, ctx);
            if new_rhs.basic_type != BasicType::Int && new_rhs.basic_type != BasicType::Const {
                rhs.error_spot(format!(
                    "Expression at the right of the operator should be int or const"
                ));
            }
            if new_lhs.basic_type == BasicType::Const && new_rhs.basic_type == BasicType::Const {
                return Node {
                    startpos: node.startpos,
                    endpos: node.endpos,
                    node_type: Number(eval(node, ctx)),
                    basic_type: BasicType::Const,
                };
            }
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: BinOp(ttype.clone(), Box::new(new_lhs), Box::new(new_rhs)),
                basic_type: BasicType::Int,
            }
        }
        Call(name, call_args, _) => {
            let (_, n) = ctx.find(&name, node);
            if let Func(ret, _, def_args, _) = &n.node_type {
                if call_args.len() != def_args.len() {
                    node.error_spot(format!(
                        "Argument length of {} should be {} instead of {}",
                        name,
                        def_args.len(),
                        call_args.len()
                    ));
                }
                let mut new_call_args = vec![];
                for (call_arg, def_arg) in call_args.iter().zip(def_args.iter()) {
                    let new_call_arg = traverse(&call_arg, ctx);
                    new_call_args.push(new_call_arg.clone());
                    //Both int/const
                    if let Decl(def_basic_type, _, _, _, _) = &def_arg.node_type {
                        if def_basic_type == &BasicType::Int
                            && (new_call_arg.basic_type == BasicType::Int
                                || new_call_arg.basic_type == BasicType::Const)
                        {
                            continue;
                        }
                    }
                    //Both array
                    if let Decl(def_basic_type, _, _, _, _) = &def_arg.node_type {
                        if let BasicType::IntArray(def_dims) = def_basic_type {
                            if let BasicType::IntArray(call_dims) = &new_call_arg.basic_type {
                                for (call_dim, def_dim) in
                                    call_dims.iter().zip(def_dims.iter()).skip(1)
                                {
                                    if call_dim != def_dim {
                                        call_arg.error_spot(format!(
                                            "error_spot dimension in function call {}",
                                            name
                                        ));
                                    }
                                }
                                continue;
                            }
                        }
                    }
                    //Others
                    call_arg.error_spot(format!("Unmatched type in function call {}", name));
                }
                Node {
                    startpos: node.startpos,
                    endpos: node.endpos,
                    node_type: Call(name.clone(), new_call_args, Box::new(n.clone())),
                    basic_type: ret.clone(),
                }
            } else {
                node.error_spot(format!("{} is not a function", name));
                unreachable!();
            }
        }
        Assign(name, indexes, expr, _) => {
            let (basic_type, n) = ctx.find(name, node);
            if let Decl(_, _, _, _, _) = n.node_type {
                match &basic_type {
                    BasicType::Const | BasicType::ConstArray(_) => {
                        node.error_spot(format!("Cannot assign to constant {}", name));
                        unreachable!()
                    }
                    BasicType::Int => {
                        if indexes.is_some() {
                            node.error_spot(format!(
                                "Integer {} should not have indexes in assign",
                                name
                            ));
                        }
                        let new_expr = traverse(expr, ctx);
                        if new_expr.basic_type != BasicType::Int
                            && new_expr.basic_type != BasicType::Const
                        {
                            node.error_spot(format!("Should assign int/const to int"))
                        }
                        Node {
                            startpos: node.startpos,
                            endpos: node.endpos,
                            node_type: Assign(
                                name.clone(),
                                None,
                                Box::new(new_expr),
                                Box::new(n.clone()),
                            ),
                            basic_type: BasicType::Nil,
                        }
                    }
                    BasicType::IntArray(dims) => {
                        if indexes.is_none() {
                            node.error_spot(format!(
                                "Integer array {} should have indexes in assign",
                                name
                            ));
                        }
                        let new_expr = traverse(expr, ctx);
                        if new_expr.basic_type != BasicType::Int
                            && new_expr.basic_type != BasicType::Const
                        {
                            node.error_spot(format!("Should assign int/const to int"));
                        }
                        if indexes.as_ref().unwrap().len() != dims.len() {
                            node.error_spot(format!(
                                "Indexes of {} should be {} instead of {}",
                                name,
                                dims.len(),
                                indexes.as_ref().unwrap().len()
                            ))
                        }
                        let mut new_indexes = vec![];
                        for index in indexes.as_ref().unwrap() {
                            let new_index = traverse(&index, ctx);
                            if new_index.basic_type != BasicType::Int
                                && new_index.basic_type != BasicType::Const
                            {
                                node.error_spot(format!(
                                    "Index of array {} should be int/const",
                                    name
                                ));
                            }
                            new_indexes.push(new_index);
                        }

                        let mut decl_node = n.clone();
                        decl_node.basic_type = basic_type;
                        Node {
                            startpos: node.startpos,
                            endpos: node.endpos,
                            node_type: Assign(
                                name.clone(),
                                Some(new_indexes),
                                Box::new(new_expr),
                                Box::new(decl_node),
                            ),
                            basic_type: BasicType::Nil,
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                node.error_spot(format!("Cannot assign to function {}", name));
                unreachable!()
            }
        }
        ExprStmt(expr) => Node {
            startpos: node.startpos,
            endpos: node.endpos,
            node_type: ExprStmt(Box::new(traverse(expr, ctx))),
            basic_type: BasicType::Nil,
        },
        Block(stmts) => {
            ctx.enter_scope();
            let mut new_stmts = vec![];
            for stmt in stmts {
                new_stmts.push(traverse(&stmt, ctx));
            }
            ctx.exit_scope();
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: Block(new_stmts),
                basic_type: BasicType::Nil,
            }
        }
        If(cond, on_true, on_false) => {
            let new_cond = traverse(cond, ctx);
            if new_cond.basic_type != BasicType::Int && new_cond.basic_type != BasicType::Const {
                node.error_spot(format!("Condition of if statement should be int/const"));
            }
            let new_on_false = if let Some(on_false_block) = on_false {
                Some(Box::new(traverse(on_false_block, ctx)))
            } else {
                None
            };
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: If(
                    Box::new(new_cond),
                    Box::new(traverse(on_true, ctx)),
                    new_on_false,
                ),
                basic_type: BasicType::Nil,
            }
        }
        While(cond, body) => {
            let new_cond = traverse(cond, ctx);
            if new_cond.basic_type != BasicType::Int && new_cond.basic_type != BasicType::Const {
                node.error_spot(format!("Condition of if statement should be int/const"));
            }
            ctx.startpos_loop();
            let new_body = Box::new(traverse(body, ctx));
            ctx.endpos_loop();
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: While(Box::new(new_cond), new_body),
                basic_type: BasicType::Nil,
            }
        }
        Return(expr) => {
            let new_expr: Option<Box<Node>>;
            let mut ret_type: BasicType;
            let (name, ret) = ctx.get_cur_func();
            if let Some(exp) = expr {
                let new_exp = traverse(exp, ctx);
                ret_type = new_exp.basic_type.clone();
                new_expr = Some(Box::new(new_exp));
            } else {
                ret_type = BasicType::Void;
                new_expr = None;
            }
            if ret_type == BasicType::Const {
                ret_type = BasicType::Int;
            }
            if ret_type != ret {
                node.error_spot(format!("Return type of {} does not match", name));
            }
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: Return(new_expr),
                basic_type: BasicType::Nil,
            }
        }
        Func(ret, name, args, body) => {
            ctx.set_cur_func(name, ret);
            let mut new_args = vec![];
            ctx.enter_scope();
            for arg in args {
                new_args.push(traverse(arg, ctx));
            }
            ctx.insert(
                name.clone(),
                BasicType::Func(Box::new(ret.clone())),
                Node::new(NodeType::Func(
                    ret.clone(),
                    name.clone(),
                    new_args.clone(),
                    body.clone(),
                )),
            );
            let new_body = traverse(body, ctx);
            ctx.exit_scope();
            Node {
                startpos: node.startpos,
                endpos: node.endpos,
                node_type: Func(ret.clone(), name.clone(), new_args, Box::new(new_body)),
                basic_type: BasicType::Nil,
            }
        }
        _ => unreachable!(),
    }
}

fn eval(node: &Node, ctx: &Runtime) -> i32 {
    // step1. 实现二元运算符的Eval.
    impl TokenType {
        fn calc(&self, lhs: i32, rhs: i32) -> i32 {
            use TokenType::*;
            match self {
                //5种算术运算
                Plus => lhs + rhs,
                Minus => lhs - rhs,
                Multi => lhs * rhs,
                Divide => lhs / rhs,
                Mods => lhs % rhs,
                //6种关系运算
                Equal => (lhs == rhs) as i32,
                NotEqual => (lhs != rhs) as i32,
                Lesserthan => (lhs < rhs) as i32,
                Greaterthan => (lhs > rhs) as i32,
                LessEqual => (lhs <= rhs) as i32,
                GreatEqual => (lhs >= rhs) as i32,
                //2种逻辑运算
                And => (lhs != 0 && rhs != 0) as i32,
                Or => (lhs != 0 || rhs != 0) as i32,
                _ => unreachable!(),
            }
        }
    }
    use NodeType::*;
    match &node.node_type {
        Nil => return 0,
        Call(name, _, _) => {
            node.error_spot(format!(
                "Cannot call function {} in constant expression",
                name
            ));
            unreachable!()
        }
        Number(num) => num.clone(),
        BinOp(ttype, lhs, rhs) => {
            let l = eval(&lhs, ctx);
            let r = eval(&rhs, ctx);
            ttype.calc(l, r)
        }
        Access(name, indexes, _) => {
            /* Access a variable
             *  1. If the variable is a const, return the value of the const
             *  2. If the variable is a const array, return the value of the const array
             */
            let (btype, def_node) = ctx.find(&name, node);
            match btype {
                BasicType::Const => {
                    //Access a const with index
                    if indexes.is_some() {
                        node.error_spot(format!("Access constant {} with index", name));
                    }
                    if let NodeType::Decl(_, _, _, initlist, _) = def_node.node_type.clone() {
                        if let NodeType::Number(num) = initlist.unwrap()[0].node_type {
                            return num;
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                BasicType::ConstArray(dims) => {
                    if let Some(index) = indexes {
                        if index.len() == dims.len() {
                            /* Calculate the offset of the array */
                            let mut offset = 0;
                            for (i, indexnode) in index.iter().enumerate() {
                                let id = eval(indexnode, ctx);
                                if let Some(n) = dims.get(i + 1) {
                                    offset += id * (*n as i32);
                                } else {
                                    offset += id;
                                }
                            }
                            if let NodeType::Decl(_, _, _, initlist, _) = node.node_type.clone() {
                                if let Some(n) = initlist.unwrap().get(offset as usize) {
                                    // 用if let拿到当前的Node.
                                    if let NodeType::Number(num) = n.node_type {
                                        // 如果是Number类型, 则返回值
                                        return num;
                                    } else {
                                        unreachable!()
                                    }
                                } else {
                                    //如果索引超出范围, 则报错
                                    node.error_spot(format!("Index of {} out of range", name));
                                    unreachable!()
                                }
                            } else {
                                unreachable!()
                            }
                        } else {
                            node.error_spot(format!(
                                "Dimension of {} should be {} instead of {}",
                                name,
                                dims.len(),
                                index.len()
                            ));
                            unreachable!()
                        }
                    } else {
                        node.error_spot(format!("{} should be accessed with index", name));
                        unreachable!()
                    }
                }
                BasicType::Int | BasicType::IntArray(_) => {
                    node.error_spot(format!("{} should be a constant", name));
                    unreachable!()
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

/* 根据给定维度和初始化列表展开初始化. */
fn expand_inits(
    dims: &Vec<Node>,
    inits: &Vec<Node>,
    need_eval: bool,
    ctx: &mut Runtime,
    level: usize,
) -> Vec<Node> {
    if level == dims.len() {
        inits
            .last()
            .unwrap()
            .error_spot(format!("Dimension of initializer exceeded"));
    }
    // max为各维度长度的乘积, 就是总元素的个数.
    let mut max = 1;
    for dim_node in dims.get(level..).unwrap() {
        if let NodeType::Number(dim) = dim_node.node_type {
            max *= dim;
        }
    }
    let mut expanded = vec![];
    for init_node in inits {
        if let NodeType::InitList(inits2) = &init_node.node_type {
            for new_init in expand_inits(dims, &inits2, need_eval, ctx, level + 1) {
                expanded.push(new_init);
            }
        } else {
            let new_init = if need_eval {
                Node {
                    startpos: init_node.startpos,
                    endpos: init_node.endpos,
                    node_type: NodeType::Number(eval(init_node, ctx)),
                    basic_type: BasicType::Const,
                }
            } else {
                let ini = traverse(init_node, ctx);
                ini
            };
            expanded.push(new_init);
        }
    }
    if expanded.len() > max as usize {
        inits
            .last()
            .unwrap()
            .error_spot(format!("Length of initializer exceeded"));
    } else {
        for _ in expanded.len()..(max as usize) {
            expanded.push(Node {
                startpos: 0,
                endpos: 0,
                node_type: NodeType::Number(0),
                basic_type: BasicType::Const,
            });
        }
    }
    expanded
}

pub fn semantic(ast: &Vec<Node>, path: &String) -> Vec<Node> {
    unsafe { FILEPATH = path.clone() }
    let mut ctx = Runtime::new();
    /* 遍历AST树, 并对每个节点进行"语义分析", 相当于AST的interpreter(解释器) */
    let mut new_nodes = vec![];
    for node in ast {
        match &node.node_type {
            NodeType::DeclStmt(_) => {
                let new = traverse(node, &mut ctx);
                new_nodes.push(new);
            }
            _ => {}
        }
    }
    for node in ast {
        match &node.node_type {
            NodeType::DeclStmt(_) => {}
            _ => {
                let new = traverse(node, &mut ctx);
                new_nodes.push(new);
            }
        }
    }
    new_nodes
}
