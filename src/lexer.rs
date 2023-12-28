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

/* 实现Debug trait, 让Token可以使用{:?}被打印到控制台或者指定文件. */
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //get token content
        let content: String = self.buf[self.startpos..self.endpos].iter().collect();
        //write token to stdout Stream.
        write!(
            f,
            "Token{{\tline:{:?}\ttype:{:?}\tvalue:{:?}\t}}",
            self.line_no, self.sort, content,
        )
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
            line_no: 1,     //各IDE,行号都是从1开始.
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
        let mut file = File::open(path).expect("File cannot be opened");
        file.read_to_string(&mut content)
            .expect("File cannot be converted to string");
        // rust的编码是unicode(utf-8), 不支持字符串用下标访问, !:必须把字符串转换为字符数组.
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
            //否则就是十进制数, 10进制数又分10进制整数和10进制浮点数.
            _ => self.parse_decimal(),
        }
    }

    //  解析10进制整数和浮点数.
    fn parse_decimal(&mut self) {
        let start = self.current;
        let mut integer_sum = 0;
        let mut integer_len = 0;
        let mut fraction_sum = 0;
        let mut fraction_len = 0;
        let mut is_float = false;
        for c in self.chars[self.current..].iter() {
            if let Some(val) = c.to_digit(10) {
                if is_float {
                    fraction_sum = fraction_sum * 10 + val;
                    fraction_len += 1;
                } else {
                    integer_sum = integer_sum * 10 + val;
                    integer_len += 1;
                }
            } else if *c == '.' {
                is_float = true;
            } else {
                break;
            }
        }
        if is_float && fraction_len > 0 {
            let float_value =
                integer_sum as f64 + fraction_sum as f64 / 10_f64.powi(fraction_len as i32);
            self.current = start + integer_len + fraction_len + 1;
            let mut t = self.new_token(TokenType::FloatNumber(float_value as f32));
            t.endpos = self.current;
            self.tokens.push(t);
        } else {
            let int_value = integer_sum;
            self.current = start + integer_len;
            let mut t = self.new_token(TokenType::IntNumber(int_value as i32));
            t.endpos = self.current;
            self.tokens.push(t);
        }
    }

    //解析8进制和16进制数,同时进行进制表示检查。
    fn parse_number(&mut self, base: u32) {
        let light = match base {
            8 => 1,
            16 => 2,
            _ => unreachable!(),
        };
        let mut sum = 0;
        let mut len = 0;
        let start = self.current; // Store the initial value of self.current
        let mut flag = true;
        for c in self.chars[self.current..].iter() {
            if let Some(val) = c.to_digit(base) {
                sum = sum * base as i32 + val as i32;
                len += 1;
            } else {
                if c.is_alphanumeric() {
                    flag = false;
                    len += 1;
                    continue;
                }
                if flag == false {
                    if light == 1 {
                        self.error(
                            "Lexer error: Illegal OCTAL number",
                            "Error type A at this line: Illegal octal number",
                        );
                    } else if light == 2 {
                        self.error(
                            "Lexer error: Illegal HEXIDECIMAL number",
                            "Error type A at this line: Illegal hexadecimal number",
                        );
                    }
                }
                break;
            }
        }
        self.current = start + len;
        if flag {
            let mut t = self.new_token(TokenType::IntNumber(sum));
            t.endpos = self.current;
            self.tokens.push(t);
        } else {
            let mut t = self.new_token(TokenType::WrongFormat(
                "Wrong Oct/Hex representation!".into(),
            ));
            t.endpos = self.current;
            self.tokens.push(t);
        }
    }

    /*
        扫描标识符, 并判断是否是关键字.
        整体的思路是:
        step1. 提取出标识符的name, 它可能是关键字, 也可能是标识符.
        step2. 把name与预先做好的关键字表进行匹配, 匹配到了就是关键字.
        step3. 遍历关键字表完了都没匹配上, 就是真正意义上的标识符.
        tips: 不管是标识符还是关键字, 识别好了都得new一个token出来把它们信息装好后推入tokens.
    */
    fn scan_identifier(&mut self, keywords: &HashMap<String, TokenType>) {
        //step1. name got
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
        //step2. Keyword ?
        let mut t: Token;
        if let Some(sort) = keywords.get(&name) {
            t = self.new_token(sort.clone())
        } else {
            //step3. Identifier!
            t = self.new_token(TokenType::Identifier(name))
        }
        //step4. add to tokens.
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

    /* 用于处理Lexical Analysis阶段的报错信息 */
    fn error(&mut self, msg: &str, suggest: &str) {
        /* step1. collect error info */
        let mut len = 0;
        let thisline = self.line_starts[self.line_no - 1];
        let mut white_space_pos = 0;
        for &c in self.chars[thisline..].iter() {
            if c == '\n' {
                break;
            }
            if c == '=' {
                white_space_pos = len;
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

        let c: String = self.chars[thisline + white_space_pos + 1..thisline + len]
            .iter()
            .collect();
        // 指出错误字符具体位置, 并打印出修正意见
        println!("{} {}:{}", "^", suggest, c);
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
                        self.error(
                            "invalid character!",
                            "Error type A at this line:Invalid character",
                        );
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
       0.这是一个库函数(暴露给外界), 库函数一般是封装内部对象的实例函数, 所以需要先new一个对象,再调用该对象的方法.
       1."tokenize"这个动作的执行者是Lexer, 先New一个Lexer作为执行词法分析的实体.
       2.调用Lexer的成员函数scan(),扫描整个文件,把扫描到的一个个词法单元装入lexer.tokens中.
       3.返回tokens
    */
    let mut lexer = Lexer::new(Rc::new(path));
    lexer.scan(&keyword_table_init(), &double_sign_table_init());
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
