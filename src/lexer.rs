use crate::TokenType;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::rc::Rc;

enum CharType {
    Spacebar,    // ' ','\'t'
    Linefeed,    // '\n (LF), todo:support \r \n (CRLF)'
    Alphabet,    // 'a-z''A-Z'
    Digit,       // '0-9'
    Other(char), // 表示在一个"特殊"字符char,特殊字符在于它既不是数字也不是字母.
}

/*----------------About token-----------------*/

#[derive(Clone)]
pub struct Token {
    /*
        需要注意的是, rust中的结构体成员默认是私有的, 也就是模块外部不可见, 模块内可见
        如果需要让结构体字段暴露出来, 让外部模块也可调用, 则要在字段前加上pub.
        这与Golang不同, Golang通过首字母大小写来控制可见性,
        而Rust通过默认private, 加上pub前缀公开来控制可见性.
    */
    pub sort: TokenType,
    pub buf: Rc<Vec<char>>,
    pub source: Rc<String>,
    pub line_start: Rc<usize>,
    pub line_no: usize,
    pub startpos: usize,
    pub endpos: usize,
}

/* 实现Debug trait, 让Token可以被打印到控制台或者指定文件. */
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //get token content
        let content: String = self.buf[self.startpos..self.endpos].iter().collect();
        //write token to stdout Stream.
        write!(
            f,
            "Token{{\tline:{:?}\ttype:{:?}\tvalue:{:?}\t}}",
            self.line_no,
            self.sort,
            content,
            /*
                self.startpos - *self.line_start, //开始列号.
                self.endpos - *self.line_start    //结束列号.
            */
        )
        /*
            返回一个Result, 这个Result是什么? 它是一个枚举类型, 它有两个值, Ok和Err.
            Ok表示成功, 而Err表示失败. 我们这里返回的是Ok, 所以返回Ok(())
        */
    }
}

impl Token {
    pub fn new(
        sort: TokenType,
        buf: Rc<Vec<char>>,
        source: Rc<String>,
        line_start: Rc<usize>,
        line_no: usize,
        startpos: usize,
        endpos: usize,
    ) -> Self {
        let _ = endpos;
        Token {
            sort,
            buf,
            source,
            line_start,
            line_no,
            startpos,
            endpos: 0,
        }
    }
}

/*----------------About Lexer----------------- */
pub struct Lexer {
    chars: Rc<Vec<char>>,
    current: usize,
    line_starts: Vec<usize>,
    line_no: usize,
    tokens: Vec<Token>,
    source: Rc<String>,
    is_panicked: bool,
}

impl Lexer {
    /*
        奥卡姆剃刀原理：如非必要,勿增实体
        下列只列出Lexer必须实现的方法,

        tips:Rust的impl struct相当于Go的interface, 可以“类比”C++中写在对象内部的成员函数.
        impl struct是一系列方法签名, 相当于是为这个struct对象定义了一系列行为(Action).
        在impl中, Self表示结构体类型, self表示结构体的实例,
        参数中带有&self的方法可以用 instance.method()调用, 否则只能用 structName::method()调用(类似于C++的静态函数).
    */

    /* Lexer的构造函数 */
    fn new(path: Rc<String>) -> Self {
        Lexer {
            chars: Rc::new(Self::get_source(&path)),
            current: 0,
            line_starts: vec![0],
            line_no: 1,     //调研各大IDE, 行号都是从1开始.
            tokens: vec![], //用于存放提取出来的token。
            source: path,
            is_panicked: false,
        }
    }

    /* 给予Lexer识别并提取不同类型token的能力 */
    fn new_token(&self, sort: TokenType) -> Token {
        Token::new(
            sort,
            self.chars.clone(),
            self.source.clone(),
            Rc::new(self.line_starts[self.line_no - 1]), //行号从1开始,列号从0开始.
            self.line_no,
            self.current,
            0,
        )
    }

    /* 读取文件内容 */
    fn get_source(path: &str) -> Vec<char> {
        let mut content = String::new();
        /*
            type_check用于错误处理, Rust中没有Java/Cpp中Try-Catch的结构,
            用type_check来表明错误, 如果有错, 则会输出type_check括号中的内容, 没错则会无视。
        */
        let mut file = File::open(path).expect("File cannot be opened");
        file.read_to_string(&mut content)
            .expect("File cannot be converted to string");
        // rust的编码是unicode(utf-8), 不支持字符串用下标访问, 必须把字符串转换为字符数组.
        // 注意: 这里的chars()是迭代器, 不是数组, 所以访问单个字符的时候用方法get(). 范式为:chars.get()
        content.chars().collect()
    }

    /* 预处理, 主要是去掉空格和换行符, 并将其转换为对应的枚举类型.*/
    fn pre_process(&self) -> Option<CharType> {
        self.chars.get(self.current).map(|c| match c {
            ' ' | '\t' => CharType::Spacebar,
            '\n' => CharType::Linefeed,
            'a'..='z' | 'A'..='Z' => CharType::Alphabet,
            '0'..='9' => CharType::Digit,
            _ => CharType::Other(*c),
        })
    }

    fn number(&mut self) {
        match self.chars.get(self.current..self.current + 2) {
            //若是以0x(0X)开头, 则说明是十六进制数.
            Some(&['0', 'x']) | Some(&['0', 'X']) => {
                self.current += 2;
                self.parse_number(16);
            }
            //若是以0与任何一个字符开头, 则说明是八进制数.
            Some(&['0', _]) => {
                self.parse_number(8);
            }
            //否则就是十进制数.
            _ => self.parse_number(10),
        }
    }

    fn parse_number(&mut self, base: u32) {
        let mut sum = 0;
        let mut len = 0;
        //从self.pos开始, 读取len个字符, 并将其转换为数字, 然后将其相加.
        for c in self.chars[self.current..].iter() {
            //c代表当前字符, 并将其转换为数字, 并将其赋值给val.
            if let Some(val) = c.to_digit(base) {
                // 在这里, val代表是当前字符的数字值, base是进制基, sum是求得的和.
                sum = sum * base as i32 + val as i32;
                len += 1;
            } else {
                break;
            }
        }
        //将sum转换为字符串, 并将其赋值给t.
        let mut t = self.new_token(TokenType::IntNumber(sum));
        /*--将pos向后移动len个字符, 并将t的end设置为pos, 然后将t存入tokens中.-- */
        self.current += len;
        t.endpos = self.current;
        self.tokens.push(t);
    }
    /*
        fn get_real_number(&mut self) {
            /* 用于区分浮点数和整数, 其中'.'算作浮点数的一部分 */
            let setoff = self.current;
            let mut cur = self.chars.get(self.current);
            /* 对当前拿到的cur进行检查, 如果cur代表数字则往后遍历, 否则停下 */
            while cur.map_or(false, |c| c.is_ascii_digit()) {
                self.current += 1;
                cur = self.chars.get(self.current);
            }
            self.current = setoff;
            if cur == Some(&'.') {
                //println!("find the float number!");
                self.parse_float();
            } else {
                //println!("find the integer number!");
                self.scan_number();
            }
        }

        fn parse_float(&mut self) {
            /* 将字符串表示的浮点数转化为f32浮点数 */
            let mut float_str: Vec<char> = Vec::new();
            let mut cur = self.chars.get(self.current);
            /* 对当前拿到的cur进行检查, 如果cur不为空, 则继续, 否则停下. */
            while cur.is_some()
                && (*cur.unwrap() == '.' || (*cur.unwrap() >= '0' && *cur.unwrap() <= '9'))
            {
                float_str.push(*cur.unwrap());
                self.current += 1;
                cur = self.chars.get(self.current);
            }

            let mut s = String::new();
            for c in float_str {
                s.push(c);
            }

            // 把Vec<char>中的字符拼起来转换为f32类型放入sum中.
            let sum = s.parse::<f32>().unwrap();
            let mut t = self.new_token(TokenType::FloatNumber(sum));
            t.endpos = self.current;
            self.tokens.push(t);
        }

        /* 扫描数字,并根据不同进制进行处理. */
        fn scan_number(&mut self) {
            match self.chars.get(self.current..self.current + 2) {
                //若是以0x(0X)开头, 则说明是十六进制数.
                Some(&['0', 'x']) | Some(&['0', 'X']) => {
                    self.current += 2;
                    self.parse_number(16);
                }
                //若是以0开头,则说明是八进制数(参考C语言的规范).
                Some(&['0', _]) => {
                    self.parse_number(8);
                }
                //否则就是十进制数.
                _ => self.parse_number(10),
            }
        }

        /* 根据进制计算数字的值,将"值信息"放入token中,再将token推入tokens(存放token的向量)中. */
        fn parse_number(&mut self, base: u32) {
            let mut sum = 0;
            let mut len = 0;
            for c in self.chars[self.current..].iter() {
                if let Some(val) = c.to_digit(base) {
                    if base == 16 {
                        if !['A', 'B', 'C', 'D', 'E', 'F'].contains(&c) {
                            println!("Invalid character: {}", c);
                        } else {
                            sum = sum * base as i32 + val as i32;
                            len += 1;
                        }
                    }
                    if base == 8 {

                    }
                } else {
                    break;
                }
            }
            let mut t = self.new_token(TokenType::IntNumber(sum));
            self.current += len;
            t.endpos = self.current;
            self.tokens.push(t);
        }
    */

    /*
        扫描标识符, 并判断是否是关键字.
        整体的思路是:
        step1. 提取出标识符的name, 它可能是关键字, 也可能是标识符.
        step2. 把name与预先做好的关键字表进行匹配, 匹配到了就是关键字.
        step3. 遍历关键字表完了都没匹配上, 就是真正意义上的标识符.
        tips: 不顾按时标识符还是关键字, 识别好了都得new一个token出来把它们信息装好后推入tokens.
    */
    fn scan_identifier(&mut self, keywords: &HashMap<String, TokenType>) {
        //step1.
        let mut len = 1;
        while let Some(c) = self.chars.get(self.current + len) {
            //读取一个字符到变量c, 然后对c进行判断, 如果是标识符三要素: 字母, 数字, 下划线则继续
            if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == &'_' {
                len += 1;
            } else {
                break;
            }
        }
        let name: String = self.chars[self.current..self.current + len]
            .iter()
            .collect();
        //step2.
        let mut t: Token;
        if let Some(sort) = keywords.get(&name) {
            t = self.new_token(sort.clone())
        } else {
            //step3.
            t = self.new_token(TokenType::Identifier(name))
        }
        //add to tokens.
        self.current += len;
        t.endpos = self.current; //更新当前Token的end字段位置
        self.tokens.push(t); //把识别到的token加入tokens中, 这就是词法分析的根本目的嘛！
    }

    /* 处理行注释 */
    fn line_comment(&mut self) {
        while self.chars.get(self.current) != Some(&'\n') {
            self.current += 1;
        }
    }

    /*
    块注释的处理思路, 首先,因为是预读识别出/*来的, 所以要更新current指针,
    然后用while循环从字符流chars中源源不断地拿到单个字符进行解析, 分三种情况,
        1. 读到*字符, 预读下一个是不是/, 如果是则注释结束, 更新current指针返回
        2. 读到\n字符, 则要更新行号, 而且每次行号更新后还要刷新每行的起始列号(要考虑缩进的问题)
        3. 两者都不是, 则忽略所读的内容, current指针向前加1即可
    如果循环结束了, 都没有返回, 说明根本没读到*/这个结束的标注, 则报错.
     */
    fn block_comment(&mut self) {
        self.current += 2;
        while let Some(&c) = self.chars.get(self.current) {
            if c == '*' {
                if let Some(&judge) = self.chars.get(self.current + 1) {
                    if judge == '/' {
                        self.current += 2;
                        return;
                    }
                }
            }
            if c == '\n' {
                self.line_no += 1;
                self.line_starts.push(self.current + 1);
            }
            self.current += 1; // '\n'和其它单个字符在这里一起+1了.
        }
        self.error(
            "block comment not end",
            "maybe you can close the comment by adding */ ?",
        );
    }

    /* 用于处理报错信息, 完全自定义 */
    fn error(&mut self, msg: &str, suggest: &str) {
        /* step1. collect error info */
        let mut len = 0;
        let thisline = self.line_starts[self.line_no - 1];
        for &c in self.chars[thisline..].iter() {
            if c == '\n' {
                break;
            }
            len += 1;
        }
        let error_info: String = self.chars[thisline..thisline + len].iter().collect();
        /* step2. print error info */
        println!("{}: {}", "Lexical analysis error", msg);
        println!(
            "{} file:{}, line:{}, column:{}.",
            "Error location ---->",
            self.source,
            self.line_no,
            self.current - thisline + 1
        );
        println!("  {}  ", "|");
        println!(" {:3}{} {}", self.line_no.to_string(), "|", error_info);
        /* step3. give suggestion on correcting*/
        print!("    {}", "|");
        // 获取错误字符的具体位置, 在前面填充若干个空格
        for _ in 0..self.current - thisline + 1 {
            print!("{}", ' ');
        }
        // 指出错误字符具体位置, 并打印出修正意见
        println!("{} {}", "^", suggest);
        println!("  {}", "|");
        self.current += 1;
        self.is_panicked = true;
    }

    /* Lexer做词法分析的核心函数, 调用了上述所有封装好的函数, 对源字符流进行解析. */
    fn scan(
        &mut self,
        keywords: &HashMap<String, TokenType>,
        double_signs: &HashMap<String, TokenType>,
    ) {
        while let Some(target) = self.pre_process() {
            match target {
                CharType::Spacebar => {
                    self.current += 1;
                }
                CharType::Linefeed => {
                    self.current += 1;
                    self.line_no += 1;
                    self.line_starts.push(self.current);
                }
                CharType::Digit => self.number(),
                CharType::Alphabet => self.scan_identifier(keywords),

                CharType::Other('/') => match self.chars.get(self.current + 1) {
                    Some('/') => self.line_comment(),
                    Some('*') => self.block_comment(),
                    _ => {
                        let mut t = self.new_token(TokenType::Divide);
                        self.current += 1;
                        t.endpos = self.current;
                        self.tokens.push(t);
                    }
                },

                CharType::Other(_) => {
                    if let Some(operator) = self.chars.get(self.current..self.current + 2) {
                        let operation_unit: String = operator.iter().collect();
                        if let Some(sort) = double_signs.get(&operation_unit) {
                            let mut t = self.new_token(sort.clone());
                            self.current += 2;
                            t.endpos = self.current;
                            self.tokens.push(t);
                            continue;
                        }
                    }
                    if let Some(operator) = Self::single_sign(self.chars[self.current]) {
                        let mut t = self.new_token(operator.clone());
                        self.current += 1;
                        t.endpos = self.current;
                        self.tokens.push(t);
                    } else {
                        self.error("invalid character!", "check if it is a ASCII character");
                    }
                }
            }
        }
    }

    /* 单符号表 */
    fn single_sign(c: char) -> Option<TokenType> {
        use TokenType::*;
        match c {
            '+' => Some(Plus),
            '-' => Some(Minus),
            '*' => Some(Multi),
            '/' => Some(Divide),
            '%' => Some(Mods),
            '=' => Some(Assign),

            '<' => Some(Lesserthan),
            '>' => Some(Greaterthan),
            '!' => Some(Not),

            ',' => Some(Comma),
            ';' => Some(Semicolon),
            '(' => Some(LeftParen),
            ')' => Some(RightParen),
            '[' => Some(LeftBracket),
            ']' => Some(RightBracket),
            '{' => Some(LeftBrace),
            '}' => Some(RightBrace),

            _ => None,
        }
    }
}

/*---------------Library function----------------*/

/* tokenize: use Lexer to tokenize the source(stored in path), charStreams -> Tokens */
pub fn tokenize(path: String) -> Vec<Token> {
    /*
       整体的解决步骤：
       0.这是一个库函数, 库函数一般是封装内部对象的实例函数, 所以需要先new一个对象,再调用该对象的方法.
       1."tokenize"这个动作的执行者是Lexer, 先New一个Lexer作为执行词法分析的实体.
       2.调用Lexer的成员函数scan(), 它执行的过程就是词法分析的过程, 把扫描到的一个个词法单元装入lexer.tokens中.
       3.如果在scan的过程中有错误(在lexer中设置了检查panic的字段),则结束程序.
       4.没有panic则正常结束, tokens读取完毕, 返回lexer.tokens
    */
    let mut lexer = Lexer::new(Rc::new(path));
    lexer.scan(&keyword_table_init(), &double_sign_table_init());
    if lexer.is_panicked {
        panic!("Lexer paniced!");
    }
    lexer.tokens
}

/*---------------tools function-------------------*/

/* 关键字表 */
fn keyword_table_init() -> HashMap<String, TokenType> {
    let mut table = HashMap::new();

    /* int,float,void,const, if,else,while,continue,break,return */
    table.insert("int".into(), TokenType::Int);
    table.insert("float".into(), TokenType::Float);
    table.insert("void".into(), TokenType::Void);
    table.insert("const".into(), TokenType::Const);

    table.insert("if".into(), TokenType::If);
    table.insert("else".into(), TokenType::Else);
    table.insert("while".into(), TokenType::While);
    table.insert("continue".into(), TokenType::Continue);
    table.insert("break".into(), TokenType::Break);
    table.insert("return".into(), TokenType::Return);
    table
}

/* 双符号运算符表 */
fn double_sign_table_init() -> HashMap<String, TokenType> {
    let mut table = HashMap::new();

    table.insert("==".into(), TokenType::Equal);
    table.insert("!=".into(), TokenType::NotEqual);
    table.insert("&&".into(), TokenType::And);
    table.insert("||".into(), TokenType::Or);
    table.insert(">=".into(), TokenType::GreatEqual);
    table.insert("<=".into(), TokenType::LessEqual);
    table
}
