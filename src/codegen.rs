use crate::ast::{Expr, Stmt};
use std::fs::{self};
use std::process::Command;
use tempfile::Builder;

pub fn compile_to_binary(ast: Vec<Stmt>, output: &str) {
    let mut rust_code = String::from("use std::io::{Read, Write};\n\nfn main() {\n");

    for stmt in ast {
        rust_code.push_str(&gen_stmt(stmt));
    }

    rust_code.push_str("}\n");

    let tmp_dir = Builder::new().prefix("spark_build").tempdir().unwrap();
    let rs_path = tmp_dir.path().join("main.rs");

    fs::write(&rs_path, rust_code).expect("Не удалось записать временный файл");

    let status = Command::new("rustc")
        .args(["-O", rs_path.to_str().unwrap(), "-o", output])
        .status()
        .expect("Не удалось запустить rustc");

    if status.success() {
        println!("✅ Программа Spark успешно собрана в файл: {}", output);
    } else {
        eprintln!("❌ Ошибка компиляции.");
    }
}

fn gen_stmt(stmt: Stmt) -> String {
    match stmt {
        Stmt::Let { name, value } => {
            format!("    let {} = {};\n", name, gen_expr(value))
        }
        Stmt::Print(expr) => {
            format!("    println!(\"{{}}\", {});\n", gen_expr(expr))
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut code = format!("    if {} {{\n", gen_expr(condition));
            for s in then_branch {
                code.push_str(&gen_stmt(s));
            }
            code.push_str("    }");
            if let Some(else_stmts) = else_branch {
                code.push_str(" else {\n");
                for s in else_stmts {
                    code.push_str(&gen_stmt(s));
                }
                code.push_str("    }");
            }
            code.push('\n');
            code
        }
        _ => String::new(),
    }
}

fn gen_expr(expr: Expr) -> String {
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Ident(i) => i,
        Expr::Bool(b) => b.to_string(),
        Expr::Input(prompt) => {
            let prompt_str = prompt.unwrap_or_default();
            format!(
                "{{ let mut __input = String::new(); std::io::stdin().read_line(&mut __input).unwrap(); if !\"{}\".is_empty() {{ print!(\"{}\"); }} __input.trim().to_string() }}",
                prompt_str, prompt_str
            )
        }
        Expr::BinaryOp(left, op, right) => {
            // Если это сложение и один из операндов — строка (определяем по " "), используем формат
            if op == "+" {
                format!(
                    "format!(\"{{}}{{}}\", {}, {})",
                    gen_expr(*left),
                    gen_expr(*right)
                )
            } else {
                format!("({} {} {})", gen_expr(*left), op, gen_expr(*right))
            }
        }
        Expr::Array(els) => {
            let parts: Vec<String> = els.into_iter().map(gen_expr).collect();
            format!("vec![{}]", parts.join(", "))
        }
        Expr::Index { array, index } => {
            // В Rust индекс должен быть usize
            format!("{}[{} as usize]", gen_expr(*array), gen_expr(*index))
        }
        Expr::MethodCall { receiver, method } => match method.as_str() {
            "len" => format!("{}.len()", gen_expr(*receiver)),
            "input" => {
                let prompt = match *receiver {
                    Expr::Str(s) => s,
                    _ => String::new(),
                };
                format!(
                    "{{ let mut __input = String::new(); std::io::stdin().read_line(&mut __input).unwrap(); if !\"{}\".is_empty() {{ print!(\"{}\"); }} __input.trim().to_string() }}",
                    prompt, prompt
                )
            }
            _ => panic!("Метод {} не поддерживается в Spark", method),
        },
    }
}
