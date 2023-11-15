use crate::lexer::Token;
use crate::BasicType;
use crate::NodeType;
use crate::Scope;
use crate::TokenType;

#[derive(Clone)]
pub struct Node {
    pub node_type: NodeType,   //NodeType是Ast的节点类型
    pub basic_type: BasicType, //BasicType是SysY语言的基本类型
    pub startpos: usize,       //startpos是(该)节点在源代码中的起始位置
    pub endpos: usize,         //endpos是(该)节点在源代码中的结束位置
}

impl Node {
    pub fn new(ntype: NodeType) -> Self {
        Node {
            node_type: ntype,
            basic_type: BasicType::Nil,
            startpos: 0,
            endpos: 0,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>, //用于存放lexer解析后的一个个token
    current: usize,     //current代表当前处理token的下标
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn get_current_token(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn get_startpos(&self) -> usize {
        self.tokens[self.current].startpos
    }

    fn get_endpos(&self) -> usize {
        self.tokens[self.current - 1].endpos
    }

    fn type_judge(&mut self, sort: TokenType) -> bool {
        let t = self.get_current_token();
        if t.sort != sort {
            return false;
        } else {
            self.current += 1;
            return true;
        }
    }

    fn type_check(&mut self, sort: TokenType) {
        let t = self.get_current_token();
        if t.sort != sort {
            t.wrong_token(format!("{:?}", sort));
        }
        self.current += 1;
    }

    fn get_basic_type(&self) -> BasicType {
        let t = self.get_current_token();
        self.current += 1;
        let result = match t.sort {
            TokenType::Void => Some(BasicType::Void),
            TokenType::Int => Some(BasicType::Int),
            TokenType::Const => {
                self.type_check(TokenType::Int); //从这里你可以看出Const是怎么被解析的, 读一个Const马上要读一个Int.
                Some(BasicType::Const)
            }
            _ => {
                t.wrong_token("invalid type declare".into());
                None
            }
        };
        result.expect("Typename required")
    }

    /*
    fn comp_unit(&mut self) -> Node {

        /* 初始化变量:获取当前token的索引, 起始位置, 基本类型, 变量名 */
        let index = self.current;
        let startpos = self.get_startpos();
        let basic_type = self.get_basic_type();
        let name = self.get_identifier();

        /* 如果当前token是左括号, 说明是函数定义 */
        if self.type_judge(TokenType::LeftParen) {
            let mut params = vec![];
            if !self.type_judge(TokenType::RightParen) {
                params.push(self.func_f_param());
                while self.type_judge(TokenType::Comma) {
                    params.push(self.func_f_param());
                }
                self.type_check(TokenType::RightParen);
            }
            let body = self.block();
            let endpos = self.get_endpos();
            return Node::new(NodeType::Func(basic_type, name, args, Box::new(body)))
                .set_range(startpos, endpos);
        }

        self.pos = pos;
        self.decl_stmt(Scope::Global)
    }
    */
}

pub fn parse(tokens: Vec<Token>) -> Vec<Node> {
    let mut ast_nodes = vec![];
    let len = tokens.len();
    let mut parser = Parser::new(tokens);
    while parser.current != len {
        ast_nodes.push(parser.comp_unit());
    }
    ast_nodes
}

impl Token {
    fn wrong_token(&self, expect: String) {
        let lstart = *self.line_start;
        //出错的信息是保存在self.buf中的, 根据index可以把它取出来, 当然这里要转换为迭代器再用collect收集.
        let errline: String = self.buf[*self.line_start..self.endpos].iter().collect();

        //学习下编译器给你指出错误时的“说话艺术”（三部曲）
        //step1.告诉你你出错的类型, 这里是语法分析出错, 具体是遇到了不合规的Token
        println!("{}: {}", "parser error", "Unexpected token",);
        //step2.告诉你出错的地点:文件名(路径),行号,列号
        println!(
            "  {} {}:{}:{}",
            "-->",
            self.source,
            self.line_no,
            self.startpos - lstart + 1 //列号是从1开始的, 所以最后+1.
        );
        //step3.告诉你出错的具体内容
        println!("   {}", "|");
        println!(
            "{:3}{} {}",
            self.line_no.to_string(),
            "|",
            errline //errline才是错误的具体内容
        );

        //指出完毕你的错误, 然后教你怎么改正错误(suggestion), expect是关键.
        print!("   {}", "|");
        for _ in 0..self.startpos - lstart + 1 {
            print!("{}", ' ');
        }
        println!(
            "{} {}{}",
            "^", //^表示在行首,
            "Expect ",
            expect //告诉你这个token应该是怎样的,expect就是说它应该是那样,而不是现在这样.
        );

        println!("   {}", "|");
        panic!("Unexpected token");
    }
}
