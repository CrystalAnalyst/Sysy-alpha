use std::io::{self};
use std::path::Path;
use sysy_alpha::{lexer::tokenize, utils::print_tokens};

fn main() {
    println!("请输入文件路径:");
    let mut file_path = String::new();
    io::stdin()
        .read_line(&mut file_path)
        .expect("Failed to read input");

    println!("您输入的文件路径为: {}", file_path);

    let tokens = tokenize(file_path);

    println!("请输入保存路径:");
    let mut save_path = String::new();
    io::stdin()
        .read_line(&mut save_path)
        .expect("Failed to read line");

    println!("您输入的保存路径为: {}", save_path);

    print_tokens(&tokens, &Path::new(&save_path));

    println!()
}
