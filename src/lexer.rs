use crate::TokenType;

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

pub fn tokenize(path: String) -> Vec<Token> {}
