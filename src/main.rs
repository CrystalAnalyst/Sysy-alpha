use std::io::{self};
use sysy_alpha::{
    lexer::{self, tokenize},
    utils::print_tokens,
};

fn main() {
    println!("请输入文件路径:");
    let mut file_path = String::new();
    io::stdin()
        .read_line(&mut file_path)
        .expect("Failed to read input");

    println!("您输入的文件路径为: {}", file_path);

    let tokens = tokenize(file_path);
}
