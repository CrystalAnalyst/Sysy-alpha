use std::path::Path;
use sysy_alpha::{lexer::tokenize, utils::print_tokens};

fn main() {
    let source_path = String::from("./test.sy");
    let target_path = String::from("./test.tokens");
    let tokens = tokenize(source_path);
    let ast = parse(tokens);
    print_tokens(&tokens, Path::new(&target_path));
    print_tree(&ast, Path::new(&target_path), "ast", false);
}
