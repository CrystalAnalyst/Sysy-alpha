use crate::TokenType;
use std::collections::HashMap;
use std::fs::File;
use std::io::preclude::Read;
use std::rc::Rc;

enum CharType {
    Spacebar,    // ' ','\'t'
    Linefeed,    // '\n'
    Alphabet,    // 'a-z''A-Z'
    Digit,       // '0-9'
    Other(char), // 表示在一个"特殊"字符char,特殊字符在于它既不是数字也不是字母.
}

#[derive[Clone]]
pub struct Token {
    pub sort: TokenType,
    pub buf: Rc<Vec<char>>,
    pub source: Rc<String>,
    pub lineStart: usize,
    pub startpos: usize,
    pub endpos: usize,
    pub lineNo: usize,
}

pub struct Lexer {
    chars: Rc<Vec<char>>,
    current: usize,
    lineStarts: Vec<usize>,
    lineNo: usize,
    tokens: Vec<Token>,
    source: Rc<String>,
    is_panicked: bool,
}

impl Lexer {
    /*
        奥卡姆剃刀原理：如非必要,勿增实体
        下列只列出Lexer必须实现的方法,
    */
    fn new(path: Rc<String>) -> Self {}
    fn getSource(path: &str) -> Vec<char> {}
    fn preProcess(&self) -> Option<CharType> {}
}

/* tokenize: use Lexer to tokenize the source(stored in path), charStreams -> Tokens */
pub fn tokenize(path: String) -> Vec<Token> {
    /*
       整体的解决步骤：
       1.找到path指向的文件并打开,
       2.读取文件中的字符流到buf中,
       3.调用Lexer的方法把字符流提取成一个个token,
       4.把token放在可变长度数组("向量")Vec中, 返回.
    */
}

fn keyword_check() -> HashMap<String, TokenType> {
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

fn Binocular_check() -> HashMap<String, TokenType> {
    let mut table = HashMap::new();

    table.insert("==".into(), TokenType::Equal);
    table.insert("!=".into(), TokenType::NotEqual);
    table.insert("&&".into(), TokenType::And);
    table.insert("||".into(), TokenType::Or);
    table.insert(">=".into(), TokenType::GreatEqual);
    table.insert("<=".into(), TokenType::LessEqual);
    table
}
