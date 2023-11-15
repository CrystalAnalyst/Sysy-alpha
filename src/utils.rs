use crate::lexer::Token;
use crate::parser::Node;
use crate::NodeType;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn print_tokens(tokens: &Vec<Token>, path: &Path) {
    //用于将Token向量写入文件中
    let mut output = File::create(path.with_extension("tokens")).unwrap();
    let mut i = 0;
    for token in tokens {
        //使用一个循环, 迭代向量中的每一个token, 将它们按指定格式写入文件中
        output
            .write_fmt(format_args!("TokenNo:{}\n{:?}\n", i, token))
            .expect("");
        i += 1;
    }
}

pub fn print_tree(ast: &Vec<Node>, path: &Path, extension: &str, with_type: bool) {
    /*
     *  打印两种类型的AST树, 用with_type来控制,
     *  一种是带"类型信息"的(语义分析后的AST),
     *  另一种是不带类型的(语法分析后的AST).
     */
    let mut output = File::create(path.with_extension(extension)).unwrap();

    // 对ast进行遍历,从root自顶向下深度优先搜索, 递归处理每一个节点.
    for n in ast {
        visit(&n, 0, &mut output, with_type);
    }

    // visit函数的作用是：递归地遍历AST,并将每个节点的信息写入指定的output文件中.
    fn visit(node: &Node, level: u32, output: &mut File, with_type: bool) {
        /*
        params:
            node初值是AST的root,
            level是当前缩进的级别,
            output文件对象,
            with_type用于区分是带类型信息的AST还是不带类型信息的AST.
        */

        //递归(dfs)遍历AST树, 并将其写入文件中, 整体的算法流程看下来就是递归下降Recursive Descending.
        match &node.node_type {
            //DeclStmt
            NodeType::DeclStmt(nodes) => {
                print_len(level, format!("DeclStmt"), output);
                for n in nodes {
                    visit(&n, level + 1, output, with_type);
                }
            }
            //Func
            NodeType::FuncDef(ret, name, args, body) => {
                print_len(level, format!("Func {},returns {:?}", name, ret), output);
                //output.write(b"//args\n");
                for arg in args {
                    visit(&arg, level + 1, output, with_type);
                }
                //output.write(b"//body\n");
                visit(&body, level + 1, output, with_type);
            }
            //Number
            NodeType::Number(num) => {
                let mut str = format!("Number {}", num);
                if with_type {
                    str.push_str(&format!(" with type {:?}", node.basic_type));
                }
                print_len(level, str, output);
            }
            //FloatNumber
            NodeType::FloatNumber(num) => {
                let mut str = format!("FloatNumber {}", num);
                if with_type {
                    str.push_str(&format!(" with type {:?}", node.basic_type));
                }
                print_len(level, str, output);
            }
            //Nil
            NodeType::Nil => print_len(level, "Nil".into(), output),
            //Declare
            /* 一些SysY语言中变量声明的例子,
              1. int a = 10;
              2. int a[2][5] = { {1,2,3,4,5}, {6,7,8,9,10} };
              3. int f(int x,int y) {return x+y;}
            */
            NodeType::Decl(basic_type, name, dims, init, scope) => {
                print_len(
                    level,
                    format!("Declare of {}({:?}) in {:?} scope", name, basic_type, scope),
                    output,
                );
                //output.write(b"//dims\n");
                if let Some(dimslist) = dims {
                    for dim in dimslist {
                        visit(&dim, level + 1, output, with_type);
                    }
                }
                //output.write(b"//init\n");
                if let Some(initlist) = init {
                    for init1 in initlist {
                        visit(&init1, level + 1, output, with_type);
                    }
                }
            }
            //InitList
            NodeType::InitList(list) => {
                print_len(level, "Initlist".into(), output);
                for i in list {
                    visit(&i, level + 1, output, with_type);
                }
            }
            //Access
            NodeType::Aceess(name, indexes, _) => {
                let mut str = format!("Access {}", name);
                if with_type {
                    str.push_str(&format!(" with type {:?}", node.basic_type));
                }
                print_len(level, str, output);
                if let Some(indexeslist) = indexes {
                    for index in indexeslist {
                        visit(&index, level + 1, output, with_type);
                    }
                }
            }
            //BinOp
            NodeType::BinOp(ttype, lhs, rhs) => {
                let mut str = format!("Binop {:?}", ttype);
                if with_type {
                    str.push_str(&format!(" with type {:?}", node.basic_type));
                }
                print_len(level, str, output);
                //output.write(b"//lhs\n");
                visit(&lhs, level + 1, output, with_type);
                //output.write(b"//rhs\n");
                visit(&rhs, level + 1, output, with_type);
            }
            //Call
            NodeType::Call(name, args, _) => {
                let mut str = format!("Function call {}", name);
                if with_type {
                    str.push_str(&format!(" with type {:?}", node.basic_type));
                }
                print_len(level, str, output);
                for arg in args {
                    visit(&arg, level + 1, output, with_type);
                }
            }
            //Assign
            NodeType::Assign(name, indexes, rhs, _) => {
                print_len(level, format!("Assign {}", name), output);
                //output.write(b"//indexes\n");
                if let Some(indexlist) = indexes {
                    for index in indexlist {
                        visit(&index, level + 1, output, with_type);
                    }
                }
                //output.write(b"//rhs\n");
                visit(&rhs, level + 1, output, with_type);
            }
            //ExprStmt
            NodeType::ExprStmt(expr) => {
                print_len(level, "ExprStmt".into(), output);
                visit(&expr, level + 1, output, with_type);
            }
            //Block
            NodeType::Block(stmts) => {
                print_len(level, "Block".into(), output);
                for stmt in stmts {
                    visit(&stmt, level + 1, output, with_type);
                }
            }
            //If
            NodeType::If(cond, on_true, on_false) => {
                print_len(level, "If".into(), output);
                //output.write(b"//Cond\n");
                visit(&cond, level + 1, output, with_type);
                //output.write(b"//True\n");
                visit(&on_true, level + 1, output, with_type);
                if let Some(f) = on_false {
                    //output.write(b"//False\n");
                    visit(&f, level + 1, output, with_type);
                }
            }
            //While
            NodeType::While(cond, body) => {
                print_len(level, "While".into(), output);
                //output.write(b"//Cond\n");
                visit(&cond, level + 1, output, with_type);
                //output.write(b"//Body\n");
                visit(&body, level + 1, output, with_type);
            }
            //Break
            NodeType::Break => {
                print_len(level, "Break".into(), output);
            }
            //Continue
            NodeType::Continue => {
                print_len(level, "Continue".into(), output);
            }
            //Return
            NodeType::Return(ret) => {
                print_len(level, "Return".into(), output);
                if let Some(r) = ret {
                    // output.write(b"//Return expr\n");
                    visit(&r, level + 1, output, with_type);
                }
            }
        }
    }

    fn print_len(level: u32, msg: String, output: &mut File) {
        output.write(b"|").expect("write error");
        for _ in 0..level {
            output.write(b"--").expect("write error");
        }
        /* 使用format_args!()来构建格式化字符串，然后使用write_fmt()来写入格式化字符串,
         * 最后使用expect()来处理可能出现的错误, 如果出错就输出"write error".
         */
        output
            .write_fmt(format_args!("{}\n", msg))
            .expect("write error");
    }
}
