use crate::lexer::Token;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn print_tokens(tokens: &Vec<Token>, path: &Path) {
    //用于将Token向量写入文件中
    let mut output = File::create(path.with_extension("tokens")).unwrap();
    let mut i = 0;
    for token in tokens {
        //使用一个循环, 迭代向量中的每一个token, 将它们按指定格式写入文件中
        output
            .write_fmt(format_args!("TokenNo:{}\n{:?}\n", i, token))
            .expect("");
        i += 1;
    }
}
