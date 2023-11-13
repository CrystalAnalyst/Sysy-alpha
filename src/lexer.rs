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
    /*
        需要注意的是, rust中的结构体成员默认是私有的, 也就是模块外部不可见, 模块内可见
        如果需要让结构体字段暴露出来, 让外部模块也可调用, 则要在字段前加上pub.
        这与Golang不同, Golang通过首字母大小写来控制可见性,
        而Rust通过默认private, 加上pub前缀公开来控制可见性.
    */
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

        tips:Rust的impl struct相当于Go的interface, 可以“类比”C++中写在对象内部的成员函数.
        impl struct是一系列方法签名, 相当于是为这个struct对象定义了一系列行为(Action).
        在impl中, Self表示结构体类型, self表示结构体的实例,
        参数中带有&self的方法可以用 instance.method()调用, 否则只能用 structName::method()调用.
    */

    fn new(path: Rc<String>) -> Self {
        Lexer {
            chars: Rc::new(Self::getSource(&path)),
            current: 0,
            lineStarts: vec![0],
            lineNo: 0,
            tokens: vec![], //用于存放解析好的token。
            source: path,
            is_panicked: false,
        }
    }

    fn getSource(path: &str) -> Vec<char> {
        let mut content = String::new();
        /*
            expect用于错误处理, Rust中没有Java/Cpp中Try-Catch的结构,
            用expect来表明错误, 如果有错, 则会输出expect括号中的内容, 没错则会无视。
        */
        let mut file = File::open(path).expect("File cannot be opened");
        file.read_to_string(&mut content)
            .expect("File cannot be converted to string");
        // rust的编码是unicode(utf-8), 不支持字符串用下标访问, 必须把字符串转换为字符数组.
        File::Close(path).expect("File close error!");
        content.chars().collect()
    }

    fn preProcess(&self) -> Option<CharType> {}

    fn scan(
        &mut self,
        keywords: &HashMap<String, TokenType>,
        binoculars: &HashMap<String, TokenType>,
    ) -> Vec<Token> {
    }
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
    let mut lexer = Lexer::new();
    lexer.scan(&keyword_check(), &Binocular_check());
    if lexer.is_panicked {
        panic!("Lexer paniced!");
    }
    lexer.tokens
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
