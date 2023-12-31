use crate::lexer::Token;
use crate::BasicType;
use crate::NodeType;
use crate::Scope;
use crate::TokenType;
#[derive(Clone)]
pub struct Node {
    pub node_type: NodeType,   //NodeType是Ast的节点类型
    pub basic_type: BasicType, //BasicType是SysY语言的基本类型
    pub startpos: usize,       //startpos是(该)节点在源代码字符流的起始位置
    pub endpos: usize,         //endpos是(该)节点在源代码字符流的结束位置
}

impl Node {
    /*
       Node可执行的"动作列表" (ActionList)
       构造函数(创建一个特定类型的新Node),
       零值初始化, 定界, 二元操作(因为二元运算符太多,这里统一抽出来).
    */
    pub fn new(ntype: NodeType) -> Self {
        Node {
            node_type: ntype,
            basic_type: BasicType::Nil,
            startpos: 0,
            endpos: 0,
        }
    }
    fn zero_init() -> Self {
        Node::new(NodeType::Number(0))
    }
    fn bound(mut self, start: usize, end: usize) -> Self {
        self.startpos = start;
        self.endpos = end;
        self
    }
    fn binary_operation(sort: TokenType, lhs: Node, rhs: Node) -> Self {
        Node::new(NodeType::BinOp(sort, Box::new(lhs), Box::new(rhs)))
    }
}

pub struct Parser {
    tokens: Vec<Token>, //用于存放lexer解析后的一个个token
    current: usize,     //current代表当前处理token的下标
}

impl Parser {
    /*------------------构造函数------------------*/
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    /*------------------辅助函数-------------------*/
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
        let mut sign = String::new();
        if t.sort != sort {
            match sort {
                TokenType::Comma => sign = "','".to_string(),
                TokenType::Semicolon => sign = "';'".to_string(),
                TokenType::LeftBrace => sign = "'{'".to_string(),
                TokenType::RightBrace => sign = "'}'".to_string(),
                TokenType::LeftBracket => sign = "'['".to_string(),
                TokenType::RightBracket => sign = "']'".to_string(),
                TokenType::LeftParen => sign = "'('".to_string(),
                TokenType::RightParen => sign = "')'".to_string(),
                _ => {}
            }
            t.wrong_token(format!("Error type B at this line: missing {:?}", sign));
        }
        self.current += 1;
    }

    /*------------语法分析:核心函数列表-------------*/

    /*-----------------变量类---------------------*/
    fn get_basic_type(&mut self) -> BasicType {
        let t = self.get_current_token();
        self.current += 1;
        let result = match t.sort {
            TokenType::Void => Some(BasicType::Void),
            TokenType::Int => Some(BasicType::Int),
            TokenType::Float => Some(BasicType::Float),
            TokenType::Const => {
                self.type_check(TokenType::Int); //读一个Const马上要读一个Int.
                Some(BasicType::Const)
            }
            _ => {
                t.wrong_token("Error type B at this line: invalid type declare".into());
                None
            }
        };
        result.expect("Typename required")
    }

    fn get_identifier(&mut self) -> String {
        let name: String;
        if let TokenType::Identifier(id) = &self.get_current_token().sort {
            self.current += 1;
            name = id.clone();
        } else {
            self.get_current_token()
                .wrong_token("Error typbe B at this line: expect function or value name".into());
            return "".to_string();
        }
        name
    }

    fn seek_array(&mut self, is_param: bool) -> Option<Vec<Node>> {
        let mut v = vec![]; //初始化一个空向量, v的值代表了各维度上的长度.
        let mut allow_empty = is_param;
        //while的目的是找到当前的维度dimensionality, 通常来讲, 一维数组[], 二维数组[][] 差不多了。
        while self.type_judge(TokenType::LeftBracket) {
            let startpos = self.get_startpos();
            if allow_empty {
                allow_empty = false;
                while !self.type_judge(TokenType::RightBracket) {
                    self.current += 1;
                }
                let endpos = self.get_endpos();
                v.push(Node::new(NodeType::Nil).bound(startpos, endpos));
                continue;
            }
            let len = self.const_exp(false);
            v.push(len);
            self.type_check(TokenType::RightBracket);
        } //while结束后, v中应该已经有了所有的维度了.

        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }

    /*----------------语句类---------------------- */
    fn decl_stmt(&mut self, scope: Scope) -> Node {
        let startpos = self.get_startpos();
        let t = self.get_current_token();
        self.current += 1;
        let basic_type = match t.sort {
            TokenType::Const => {
                self.type_check(TokenType::Int);
                Some(BasicType::Const)
            }
            TokenType::Int => Some(BasicType::Int),
            TokenType::Float => Some(BasicType::Float),
            _ => {
                t.wrong_token("Error type B at this line: type define".into());
                None
            }
        }
        .expect("type_check type define");

        /*
           几个声明的例子, 对号入座：
           int a=1, b=2, c=3;
           int f(int x,int y) {return x+y;}
           int array1[5] = {0,1,2,3,4};
           const int Seven = 7;
        */
        let mut first = true;
        let mut decl_list = vec![]; //声明列表
        while !self.type_judge(TokenType::Semicolon) {
            if first {
                first = false;
            } else {
                // 除了声明的第一个元素,后面都先读逗号.
                self.type_check(TokenType::Comma);
            }
            let startpos = self.get_startpos();
            let name = self.get_identifier(); //解析出当前声明的name,
            let dims = self.seek_array(false); //解析出当前声明的维度,
            let init: Option<Vec<Node>>;
            if self.type_judge(TokenType::Assign) {
                //有等于号, 说明要初始化
                if dims.is_none() {
                    //add.exp()用于初始化单个变量
                    init = Some(vec![self.add_exp(false)]);
                } else {
                    //init_val()用于初始化数组
                    init = Some(self.init_list());
                }
            } else if basic_type == BasicType::Const {
                self.get_current_token()
                    .wrong_token("Error type B at this line: assign in const declaration".into());
                unreachable!();
            } else {
                init = None;
            }
            let endpos = self.get_endpos();
            /* 声明节点 */
            decl_list.push(
                Node::new(NodeType::Decl(
                    basic_type.clone(),
                    name,
                    dims,
                    init,
                    scope.clone(),
                ))
                .bound(startpos, endpos),
            );
        }
        let endpos = self.get_endpos();
        //声明语句
        Node::new(NodeType::DeclStmt(decl_list)).bound(startpos, endpos)
    }

    fn init_list(&mut self) -> Vec<Node> {
        // init_val:初始化列表,用于初始化数组.
        // 一个数组：int a[5] = {1, 2, 3, 4, 5};
        // 二维数组：int a[5][5] = { {1, 2, 3, 4, 5}, {1, 2, 3, 4, 5} };
        let mut init = vec![];
        let mut first = true;
        self.type_check(TokenType::LeftBrace); // 左大括号
        while !self.type_judge(TokenType::RightBrace) {
            // 首元素(元素0), 然后,ele1 ,ele2 ,ele3 ...
            if first {
                first = false;
            } else {
                self.type_check(TokenType::Comma);
            }
            // 解析当前元素的值
            let startpos = self.get_startpos();
            match self.get_current_token().sort {
                TokenType::LeftBrace => {
                    let n = Node::new(NodeType::InitList(self.init_list()));
                    let endpos = self.get_endpos();
                    init.push(n.bound(startpos, endpos));
                }
                TokenType::Identifier(_) | TokenType::IntNumber(_) | TokenType::LeftParen => {
                    init.push(self.add_exp(false));
                }
                _ => {
                    self.get_current_token()
                        .wrong_token("Error type B at this line : expession or initlist".into());
                }
            }
        }
        init
    }

    fn stmt(&mut self) -> Node {
        let startpos = self.get_startpos();
        let t = self.get_current_token();
        self.current += 1;
        match t.sort {
            TokenType::Identifier(id) => {
                let pos = self.current;
                let index = self.seek_array(false);
                // Token是标识符, 后面还跟着一个=号, 一眼赋值语句。
                if self.type_judge(TokenType::Assign) {
                    let exp = self.add_exp(false);
                    self.type_check(TokenType::Semicolon);
                    let endpos = self.get_endpos();
                    Node::new(NodeType::Assign(
                        id,
                        index,
                        Box::new(exp),
                        Box::new(Node::zero_init()),
                    ))
                    .bound(startpos, endpos)
                } else {
                    // 否则是"表达式语句"(表达式后面跟着一个分号)
                    self.current = pos - 1;
                    let exp = self.add_exp(false);
                    self.type_check(TokenType::Semicolon);
                    let endpos = self.get_endpos();
                    Node::new(NodeType::ExprStmt(Box::new(exp))).bound(startpos, endpos)
                }
            }
            TokenType::Int | TokenType::Const => {
                self.current -= 1;
                self.decl_stmt(Scope::Local)
            }
            TokenType::Float => {
                self.current -= 1;
                self.decl_stmt(Scope::Local)
            }
            TokenType::LeftBrace => {
                self.current -= 1;
                self.block()
            }
            TokenType::If => {
                let on_false: Option<Box<Node>>;
                self.type_check(TokenType::LeftParen);
                let cond = self.l_or_exp();
                self.type_check(TokenType::RightParen);
                let on_true = self.stmt();
                if self.type_judge(TokenType::Else) {
                    on_false = Some(Box::new(self.stmt()));
                } else {
                    on_false = None;
                }
                let endpos = self.get_endpos();
                Node::new(NodeType::If(Box::new(cond), Box::new(on_true), on_false))
                    .bound(startpos, endpos)
            }
            TokenType::While => {
                self.type_check(TokenType::LeftParen);
                let cond = self.l_or_exp();
                self.type_check(TokenType::RightParen);
                let body = self.stmt();
                let endpos = self.get_endpos();
                Node::new(NodeType::While(Box::new(cond), Box::new(body))).bound(startpos, endpos)
            }
            TokenType::Break => {
                self.type_check(TokenType::Semicolon);
                let endpos = self.get_endpos();
                Node::new(NodeType::Break).bound(startpos, endpos)
            }
            TokenType::Continue => {
                self.type_check(TokenType::Semicolon);
                let endpos = self.get_endpos();
                Node::new(NodeType::Continue).bound(startpos, endpos)
            }
            TokenType::Return => {
                let ret: Option<Box<Node>>;
                if self.type_judge(TokenType::Semicolon) {
                    ret = None;
                } else {
                    ret = Some(Box::new(self.add_exp(false)));
                    self.type_check(TokenType::Semicolon);
                }
                let endpos = self.get_endpos();
                Node::new(NodeType::Return(ret)).bound(startpos, endpos)
            }
            _ => {
                let exp = self.add_exp(false);
                self.type_check(TokenType::Semicolon);
                let endpos = self.get_endpos();
                Node::new(NodeType::ExprStmt(Box::new(exp))).bound(startpos, endpos)
            }
        }
    }

    /*---------------函数类-----------------------*/

    fn param(&mut self) -> Node {
        let startpos = self.get_startpos();
        self.type_check(TokenType::Int);
        let name = self.get_identifier();
        let dim = self.seek_array(true);
        let basic_type: BasicType;
        if dim.is_none() {
            basic_type = BasicType::Int;
        } else {
            basic_type = BasicType::IntArray(vec![0]);
        }
        let endpos = self.get_endpos();
        Node::new(NodeType::Decl(basic_type, name, dim, None, Scope::Params))
            .bound(startpos, endpos)
    }

    fn block(&mut self) -> Node {
        let startpos = self.get_startpos();
        let mut stmts = vec![];
        self.type_check(TokenType::LeftBrace);
        while !self.type_judge(TokenType::RightBrace) {
            stmts.push(self.stmt());
        }
        let endpos = self.get_endpos();
        Node::new(NodeType::Block(stmts)).bound(startpos, endpos)
    }

    /*-----------------表达式类----------------- */

    fn primary_exp(&mut self, cond: bool) -> Node {
        let t = self.get_current_token();
        let startpos = t.startpos;
        self.current += 1;

        let result = match &t.sort {
            TokenType::LeftParen => {
                let exp = self.const_exp(cond);
                if self.type_judge(TokenType::RightParen) {
                    Some(exp)
                } else {
                    None
                }
            }
            TokenType::IntNumber(num) => Some(Node::new(NodeType::Number(*num))),
            TokenType::FloatNumber(num) => Some(Node::new(NodeType::FloatNumber(*num))),
            TokenType::Identifier(id) => {
                if self.type_judge(TokenType::LeftParen) {
                    let mut args = vec![];
                    if !self.type_judge(TokenType::RightParen) {
                        args.push(self.const_exp(cond));
                        while self.type_judge(TokenType::Comma) {
                            args.push(self.const_exp(cond));
                        }
                        if self.type_judge(TokenType::RightParen) {
                            Some(Node::new(NodeType::Call(
                                id.clone(),
                                args,
                                Box::new(Node::zero_init()),
                            )))
                        } else {
                            None
                        }
                    } else {
                        Some(Node::new(NodeType::Call(
                            id.clone(),
                            args,
                            Box::new(Node::zero_init()),
                        )))
                    }
                } else {
                    Some(Node::new(NodeType::Access(
                        id.to_string(),
                        self.seek_array(false),
                        Box::new(Node::zero_init()),
                    )))
                }
            }
            _ => {
                t.wrong_token("Error type B at this line : Expression cannot resolved!".into());
                None
            }
        };

        let endpos = self.get_endpos();
        match result {
            Some(node) => node.bound(startpos, endpos),
            None => Node::zero_init().bound(startpos, endpos),
        }
    }

    /* Unary expessions:一元表达式 */
    // 明确一点, SysY语言的单目运算符(作用于单独一个变量的运算符)有+,-,!
    // 其中, +a代表自增1, -a代表自减1, !a代表取反(只能在条件表达式中使用).
    fn unary_exp(&mut self, cond: bool) -> Node {
        /* params: cond代表是否是条件表达式 */
        let startpos = self.get_startpos();
        loop {
            if self.type_judge(TokenType::Plus) {
                // 自增
                continue;
            } else if self.type_judge(TokenType::Minus) {
                // 自减
                let mut rhs = Node::binary_operation(
                    TokenType::Minus,
                    Node::zero_init(),
                    self.primary_exp(cond),
                );
                let endpos = self.get_endpos();
                rhs = rhs.bound(startpos, endpos);
                return rhs;
            } else if cond && self.type_judge(TokenType::Not) {
                // 取反
                let mut rhs = Node::binary_operation(
                    TokenType::Equal,
                    self.primary_exp(cond),
                    Node::zero_init(),
                );
                let endpos = self.get_endpos();
                rhs = rhs.bound(startpos, endpos);
                return rhs;
            } else {
                break;
            }
        }

        self.primary_exp(cond)
    }

    /* mul_exp:乘除模表达式
     *    - mul_exp * mul_exp
     *    - mul_exp / mul_exp
     *    - mul_exp % mul_exp */
    fn mul_exp(&mut self, cond: bool) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.unary_exp(cond);

        /* 循环处理乘除模表达式 */
        loop {
            if self.type_judge(TokenType::Multi) {
                // 乘
                lhs = Node::binary_operation(TokenType::Multi, lhs, self.unary_exp(cond));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::Divide) {
                // 除
                lhs = Node::binary_operation(TokenType::Divide, lhs, self.unary_exp(cond));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::Mods) {
                // 模
                lhs = Node::binary_operation(TokenType::Mods, lhs, self.unary_exp(cond));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                return lhs;
            }
        }
    }

    /* add_exp:加减表达式
     *    - add_exp + add_exp
     *    - add_exp - add_exp
     *    - mul_exp */
    fn add_exp(&mut self, cond: bool) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.mul_exp(cond);

        loop {
            if self.type_judge(TokenType::Plus) {
                //加法
                lhs = Node::binary_operation(TokenType::Plus, lhs, self.mul_exp(cond));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::Minus) {
                //减法
                lhs = Node::binary_operation(TokenType::Minus, lhs, self.mul_exp(cond));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                //mul_exp， 直接返回lhs.
                return lhs;
            }
        }
    }

    /* const_exp:常量表达式 */
    fn const_exp(&mut self, cond: bool) -> Node {
        self.add_exp(cond)
    }

    /* rel_exp:关系表达式
     *    - rel_exp < rel_exp
     *    - rel_exp > rel_exp
     *    - rel_exp <= rel_exp
     *    - rel_exp >= rel_exp
     *    - add_exp */
    fn rel_exp(&mut self) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.add_exp(true);
        loop {
            if self.type_judge(TokenType::Lesserthan) {
                // <
                lhs = Node::binary_operation(TokenType::Lesserthan, lhs, self.add_exp(true));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::Greaterthan) {
                // >
                lhs = Node::binary_operation(TokenType::Greaterthan, lhs, self.add_exp(true));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::LessEqual) {
                // <=
                lhs = Node::binary_operation(TokenType::LessEqual, lhs, self.add_exp(true));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::GreatEqual) {
                // >=
                lhs = Node::binary_operation(TokenType::GreatEqual, lhs, self.add_exp(true));
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                return lhs;
            }
        }
    }

    /* eq_exp:相等性表达式
     *    - rel_exp
     *    - eq_exp == rel_exp
     *    - eq_exp != rel_exp */
    fn eq_exp(&mut self) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.rel_exp();
        loop {
            if self.type_judge(TokenType::Equal) {
                lhs = Node::binary_operation(TokenType::Equal, lhs, self.rel_exp());
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else if self.type_judge(TokenType::NotEqual) {
                lhs = Node::binary_operation(TokenType::NotEqual, lhs, self.rel_exp());
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                return lhs;
            }
        }
    }

    /* l_and_exp:逻辑与表达式
     *    - EqExp
     *    - LAndExp && EqExp
     * */
    fn l_and_exp(&mut self) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.eq_exp();
        loop {
            if self.type_judge(TokenType::And) {
                lhs = Node::binary_operation(TokenType::And, lhs, self.eq_exp());
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                return lhs;
            }
        }
    }

    /** l_or_exp:逻辑或表达式
     *    - l_and_exp || l_or_exp
     *    - l_and_exp */
    fn l_or_exp(&mut self) -> Node {
        let startpos = self.get_startpos();
        let mut lhs = self.l_and_exp();
        loop {
            if self.type_judge(TokenType::Or) {
                lhs = Node::binary_operation(TokenType::Or, lhs, self.l_and_exp());
                let endpos = self.get_endpos();
                lhs = lhs.bound(startpos, endpos);
            } else {
                return lhs;
            }
        }
    }

    /* 处理编译单元, 每处理好一个就返回一个ast中的Node.
     * 处理依据SysY(2022)语言定义:
     * CompUnit → [ CompUnit ] ( Decl | FuncDef ) */
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
                params.push(self.param());
                while self.type_judge(TokenType::Comma) {
                    params.push(self.param());
                }
                self.type_check(TokenType::RightParen);
            }
            let body = self.block();
            let endpos = self.get_endpos();
            return Node::new(NodeType::Func(basic_type, name, params, Box::new(body)))
                .bound(startpos, endpos);
        }

        self.current = index;
        self.decl_stmt(Scope::Global)
    }
}

impl Token {
    fn wrong_token(&self, expect: String) {
        let lstart = *self.line_start;
        //出错的信息是保存在self.buf中的, 根据index可以把它取出来, 当然这里要转换为迭代器再用collect收集.
        let errline: String = self.buf[*self.line_start..self.endpos].iter().collect();

        //step1.告诉你你出错的类型, 这里是语法分析出错, 具体是遇到了不合规的Token
        println!("{}: {}", "Parsing error", "Error type B found.",);
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
        print!("   {}", "|");
        for _ in 0..self.startpos - lstart + 1 {
            print!("{}", ' ');
        }
        println!(
            "{} {}",
            "^", //^表示在行首,
            expect
        );

        println!("   {}", "|");
        //panic!("Untype_checked token");
    }
}

/*----------------对外提供的库函数------------------*/
pub fn parse(tokens: Vec<Token>) -> Vec<Node> {
    let mut ast_nodes = vec![];
    let len = tokens.len();
    let mut parser = Parser::new(tokens);
    while parser.current != len {
        ast_nodes.push(parser.comp_unit());
    }
    ast_nodes
}
