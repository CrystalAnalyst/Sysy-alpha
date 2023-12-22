use std::path::Path;
use sysy_alpha::{
    lexer::tokenize, parser::parse, semantics::semantic, utils::print_tokens, utils::print_tree,
};

fn main() {
    /* 定义文件路径: .sy源代码路径, token输出路径, ast输出路径. */
    let source_path = String::from("./test.sy");
    let source = source_path.clone();
    let token_path = String::from("./test.tokens");
    let ast_path = String::from("./test.ast");

    /* 词法分析, 源字符流 -> 词法单元流tokens */
    let tokens = tokenize(source_path);
    print_tokens(&tokens, Path::new(&token_path));

    /* 语法分析, 词法单元流tokens -> 语法树ast, todo: 支持浮点类型的语法分析 */
    let ast = parse(tokens);
    print_tree(&ast, Path::new(&ast_path), "ast", false);

    /* 语义分析, 语法树ast -> 语义树sem(附带类型信息的ast) */
    let annotated_ast = semantic(&ast, &source);
    print_tree(&annotated_ast, Path::new(&ast_path), "sem", true);
}
