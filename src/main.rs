use std::path::Path;
use sysy_alpha::{lexer::tokenize, utils::print_tokens};

fn main() {
    let source_path = String::from("./test.sy");
    let tokens = tokenize(source_path);
    let target_path = String::from("./test.tokens");
    print_tokens(&tokens, Path::new(&target_path));
}
