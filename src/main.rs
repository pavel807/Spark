mod ast;
mod codegen;
mod lexer;
mod parser;

use logos::Logos;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Использование: spark <файл.sk> [-o выходной_файл] [--run]");
        return;
    }

    let input_file = &args[1];

    let path = Path::new(input_file);
    if path.extension().map(|e| e != "sk").unwrap_or(true) {
        eprintln!("❌ Ошибка: ожидается файл с расширением .sk (Spark Lang)");
        std::process::exit(1);
    }

    let mut output_name = "app";
    let mut should_run = false;

    // Обработка аргументов
    for i in 1..args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_name = &args[i + 1];
        }
        if args[i] == "--run" {
            should_run = true;
        }
    }

    let source = fs::read_to_string(input_file).expect("❌ Файл не найден");

    let tokens: Vec<lexer::Token> = lexer::Token::lexer(&source)
        .map(|res| res.expect("❌ Ошибка лексера"))
        .collect();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    codegen::compile_to_binary(ast, output_name);

    // Если флаг --run активен, запускаем программу сразу
    if should_run {
        println!("🏃 Запуск {}...", output_name);
        let mut child = Command::new(format!("./{}", output_name))
            .spawn()
            .expect("❌ Не удалось запустить программу");
        child.wait().expect("❌ Программа завершилась с ошибкой");
    }
}
