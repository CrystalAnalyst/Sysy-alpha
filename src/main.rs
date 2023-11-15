use std::path::Path;
use sysy_alpha::parser::parse;
use sysy_alpha::{lexer::tokenize, utils::print_tokens, utils::print_tree};

fn main() {
    let source_path = String::from("./test.sy");
    let target_path = String::from("./test.tokens");
    let tokens = tokenize(source_path);
    print_tokens(&tokens, Path::new(&target_path));
    let ast = parse(tokens);
    print_tree(&ast, Path::new(&target_path), "ast", false);
}
