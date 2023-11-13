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
