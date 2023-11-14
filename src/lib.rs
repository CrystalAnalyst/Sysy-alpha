pub mod lexer;
pub mod utils;

#[derive(Clone, Debug, PartialEq)]

pub enum TokenType {
    //Literals: 带值的枚举类型,类比扑克牌的花色和面值.
    IntNumber(i32),
    FloatNumber(f32),
    Identifier(String),

    //Keywords
    /*--return value--*/
    Void,
    Int,
    Float,
    Const,
    IntConst,
    FloatConst,

    /*--control flow--*/
    If,
    Else,
    While,
    Continue,
    Break,
    Return,

    /*--operators--*/
    Plus,
    Minus,
    Multi,
    Divide,
    Mods,
    Assign,

    /*--Relational Algebra--*/
    Equal,
    NotEqual,
    Lesserthan,
    Greaterthan,
    LessEqual,
    GreatEqual,

    /*--logical--*/
    And,
    Or,
    Not,

    /*--Symbols--*/
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasicType {
    Nil,
    Int,
    Float,
    Const, //这里的Const现在约定是整形常数.
    Void,
    IntArray(Vec<usize>),
    FloatArray(Vec<usize>),
    ConstArray(Vec<usize>), //约定是整形常数数组.
    Func(Box<BasicType>),   //return type
}

pub enum NodeType {
    /*
        以下每一个枚举成员都可能是Ast中的一个Node所属的类型之一
        根据Rust枚举类型可以带值的特性,可以把变元对应的产生式放入其中.
    */

    /* 常变量声明-获取类 */
    Decl,
    DeclStmt,
    InitList,
    Assign,
    BinOp,
    Aceess,

    /* 函数类 */
    FuncDef,
    Block,
    Return,
    Call,

    /* 结构-循环类 */
    If,
    While,
    Conitnue,
    Break,

    /* 结点值类 */
    Nil,
    Number(i32),
    FloatNumber(f32),
}
