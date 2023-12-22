pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod utils;
use parser::Node;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    //Literals: 带值的枚举类型,类比扑克牌的花色和面值.
    IntNumber(i32),
    FloatNumber(f32),
    Identifier(String),
    WrongFormat(String),
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
    Func(Box<BasicType>),   //用于函数的返回值.
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Global,
    Local,
    Params,
}

#[derive(Clone)]
pub enum NodeType {
    /*
        以下每一个枚举成员都可能是Ast中的一个Node所属的类型之一
        !根据Rust枚举类型可以带值的特性,可以把变元对应的产生式放入其中.
        根据SysY语言文法定义, 编译单元CompUnit -> [CompUnit] | (Decl | FuncDef),
        把Decl和FuncDef以及与它们相联系的类型单独作为两大类, 再加上语言本身的保留字两大类.
    */

    /* 常变量声明-获取-操作类 */
    /*
        在Sysyc中, Decl指Value declaration(常变量声明,不包含函数声明).可能的情况如下（不完全):
        1. 整形声明     eg: int a = 10;
        2. 一维数组声明 eg: int a[10] = {1,2,3,4,5,6,7,8,9,10};
        3. 二维数组声明 eg: int a[10][10] = {{1,2,3,4,5,6,7,8,9,10},{1,2,3,4,5,6,7,8,9,10}};
        4. 常数声明    eg: const int Monday = 1;
        通式:Type,Name,[Dimensions,InitList](可选,Option),Scope.
    */
    Decl(
        BasicType,
        String,
        Option<Vec<Node>>,
        Option<Vec<Node>>,
        Scope,
    ),
    /*
        DeclStmt专门用于处理在一条语句中出现多个声明的“变态情况”, 如:
        int a = 10, b = 20, c[5] = {1,2,3,4,5}, d[3][3] = {{1,2,3},{4,5,6},{7,8,9};
        其中的{1,2,3,4,5}和{{1,2,3},{4,5,6},{7,8,9}}之类的就叫做初始化列表InitList.
    */
    DeclStmt(Vec<Node>),
    InitList(Vec<Node>),
    // Name, [index], Exp, lhs_exp.
    // eg: a[1] = 10; 在此之前, a[1] = 0(lhs_exp);
    Assign(String, Option<Vec<Node>>, Box<Node>, Box<Node>),
    // 表达式语句, 一个表达式后跟一个';'
    ExprStmt(Box<Node>),
    // ArrayName, [index], Exp(二维数组按行取可以取出一行元素,Exp在这里就代表多维数组中按某一维度进行访问).
    Access(String, Option<Vec<Node>>, Box<Node>),
    // BinaryOperator, lhs, rhs.
    BinOp(TokenType, Box<Node>, Box<Node>),

    /* 函数类 */
    // Func(Type, Name, [Params], Block).
    Func(BasicType, String, Vec<Node>, Box<Node>),
    Block(Vec<Node>),
    Return(Option<Box<Node>>),
    Call(String, Vec<Node>, Box<Node>),

    /* 结构-循环类 */
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Continue,
    Break,

    /* 结点值类 */
    Nil,
    Number(i32),
    FloatNumber(f32),
}
