use crate::ast::{Expr, Stmt};
use std::fs;
use std::process::Command;
use tempfile::Builder;

pub fn compile_to_binary(ast: Vec<Stmt>, output: &str, opt_level: i32) {
    let mut rust_code = String::from("use std::io::{Read, Write};\n\n");

    let mut funcs = Vec::new();
    let mut classes = Vec::new();
    let mut main_stmts = Vec::new();

    for stmt in ast {
        match stmt {
            Stmt::Def { .. } => funcs.push(stmt),
            Stmt::Class { .. } => classes.push(stmt),
            Stmt::Import { .. } => main_stmts.push(stmt),
            _ => main_stmts.push(stmt),
        }
    }

    for class in classes {
        rust_code.push_str(&gen_stmt(class));
        rust_code.push('\n');
    }

    for func in funcs {
        rust_code.push_str(&gen_stmt(func));
    }

    rust_code.push_str("fn main() {\n");
    for stmt in main_stmts {
        rust_code.push_str(&gen_stmt(stmt));
    }
    rust_code.push_str("}\n");

    let tmp_dir = Builder::new().prefix("spark_build").tempdir().unwrap();
    let rs_path = tmp_dir.path().join("main.rs");

    fs::write(&rs_path, &rust_code).expect("Failed to write Rust file");

    let opt_args: Vec<&str> = match opt_level {
        1 => vec!["-C", "opt-level=1"],
        2 => vec!["-C", "opt-level=2"],
        3 => vec!["-C", "opt-level=3"],
        _ => vec!["-C", "opt-level=3"],
    };

    let status = Command::new("rustc")
        .args(&opt_args)
        .arg(rs_path.to_str().unwrap())
        .arg("-o")
        .arg(output)
        .status();

    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✅ Spark скомпилировался в: {} (opt-level={})", output, opt_level);
    } else {
        eprintln!("❌ Ошибка компиляции Rust.");
    }
}

fn gen_stmt(stmt: Stmt) -> String {
    match stmt {
        Stmt::Assign { target, value } => {
            format!("    let {} = {};\n", target, gen_expr(value))
        }
        Stmt::AugAssign { target, op, value } => {
            format!("    {} {} = {};\n", target, op, gen_expr(value))
        }
        Stmt::Expr(expr) => {
            let e = gen_expr(expr);
            if e.starts_with("println!") {
                format!("    {};\n", e)
            } else if e.contains(" = ") {
                format!("    let {};\n", e)
            } else {
                format!("    println!(\"{{}}\", {});\n", e)
            }
        }
        Stmt::Print(expr) => {
            format!("    println!(\"{{}}\", {});\n", gen_expr(expr))
        }
        Stmt::If { condition, then_branch, elif_branches, else_branch } => {
            let mut code = format!("    if {} {{\n", gen_expr(condition));
            for s in then_branch {
                code.push_str(&indent_stmt(&gen_stmt(s)));
            }
            code.push_str("    }");
            for (cond, body) in elif_branches {
                code.push_str(&format!("\n    else if {} {{\n", gen_expr(cond)));
                for s in body {
                    code.push_str(&indent_stmt(&gen_stmt(s)));
                }
                code.push_str("    }");
            }
            if let Some(else_stmts) = else_branch {
                code.push_str(" else {\n");
                for s in else_stmts {
                    code.push_str(&indent_stmt(&gen_stmt(s)));
                }
                code.push_str("    }");
            }
            code.push('\n');
            code
        }
        Stmt::While { condition, body } => {
            let mut code = format!("    while {} {{\n", gen_expr(condition));
            for s in body {
                code.push_str(&indent_stmt(&gen_stmt(s)));
            }
            code.push_str("    }\n");
            code
        }
        Stmt::For { var, iter, body, else_body } => {
            let iter_str = gen_expr(iter);
            let mut code = if iter_str.contains("0..") || iter_str.contains("range") {
                format!("    for {} in {} {{\n", var, iter_str)
            } else {
                format!("    for {} in 0..{} {{\n", var, iter_str)
            };
            for s in body {
                code.push_str(&indent_stmt(&gen_stmt(s)));
            }
            code.push_str("    }");
            if let Some(else_stmts) = else_body {
                code.push_str(" else {\n");
                for s in else_stmts {
                    code.push_str(&indent_stmt(&gen_stmt(s)));
                }
                code.push_str("    }");
            }
            code.push('\n');
            code
        }
        Stmt::Def { name, args, body } => {
            let params: Vec<String> = args.iter().map(|(s, _)| format!("{}: &str", s)).collect();
            let mut code = format!("fn {}({}) -> String {{\n", name, params.join(", "));
            for s in body {
                code.push_str(&gen_stmt(s));
            }
            code.push_str("}\n");
            code
        }
        Stmt::Class { name, bases: _, body } => {
            let mut struct_code = format!("struct {} {{\n", name);
            let mut impl_code = format!("impl {} {{\n", name);
            let mut has_new = false;
            
            for stmt in body {
                match stmt {
                    Stmt::Def { name: func_name, args, body: func_body } => {
                        if func_name == name {
                            let params: Vec<String> = args.iter().map(|(s, _)| s.clone()).collect();
                            impl_code.push_str(&format!("    pub fn new({}) -> Self {{\n", params.join(", ")));
                            for s in func_body {
                                impl_code.push_str(&gen_stmt(s));
                            }
                            impl_code.push_str("    }\n");
                            has_new = true;
                        } else {
                            let params: Vec<String> = args.iter().map(|(s, _)| s.clone()).collect();
                            impl_code.push_str(&format!("    pub fn {}({}) {{\n", func_name, params.join(", ")));
                            for s in func_body {
                                impl_code.push_str(&gen_stmt(s));
                            }
                            impl_code.push_str("    }\n");
                        }
                    }
                    Stmt::Assign { target, value: _ } => {
                        struct_code.push_str(&format!("    pub {}: i32,\n", target));
                    }
                    _ => {}
                }
            }
            
            if !has_new {
                impl_code.push_str(&format!("    pub fn new() -> Self {{\n",));
                impl_code.push_str(&format!("        {} {{ }}\n", name));
                impl_code.push_str("    }\n");
            }
            
            struct_code.push_str("}\n\n");
            impl_code.push_str("}\n");
            
            struct_code + &impl_code
        }
        Stmt::Try { body, except_branches, else_branch, finally_body } => {
            let mut code = String::new();
            code.push_str("    {\n");
            
            for s in body {
                code.push_str(&gen_stmt(s));
            }
            
            if !except_branches.is_empty() {
                for (exc_type, alias, exc_body) in except_branches {
                    let _type_str = exc_type.unwrap_or_else(|| "Box<dyn std::error::Error>".to_string());
                    let alias_str = alias.unwrap_or_else(|| "e".to_string());
                    code.push_str(&format!("    }}\n    Err({}) => {{\n", alias_str));
                    for s in exc_body {
                        code.push_str(&indent_stmt(&gen_stmt(s)));
                    }
                    code.push_str("    }\n");
                }
            }
            
            if let Some(else_stmts) = else_branch {
                code.push_str("    Ok(_) => {\n");
                for s in else_stmts {
                    code.push_str(&indent_stmt(&gen_stmt(s)));
                }
                code.push_str("    }\n");
            }
            
            if let Some(finally_stmts) = finally_body {
                for s in finally_stmts {
                    code.push_str(&gen_stmt(s));
                }
            }
            
            code.push_str("    }\n");
            code
        }
        Stmt::Raise(expr) => {
            if let Some(e) = expr {
                format!("    panic!(\"{{}}\", {});\n", gen_expr(e))
            } else {
                "    panic!(\"Raised exception\");\n".to_string()
            }
        }
        Stmt::Assert(cond, msg) => {
            if let Some(m) = msg {
                format!("    assert!({}, \"{{}}\", {});\n", gen_expr(cond), gen_expr(m))
            } else {
                format!("    assert!({});\n", gen_expr(cond))
            }
        }
        Stmt::Return(expr) => {
            if let Some(e) = expr {
                let expr_str = gen_expr(e);
                if !expr_str.contains('=') {
                    format!("    {}\n", expr_str)
                } else {
                    format!("    let {};\n", expr_str)
                }
            } else {
                "    return;\n".to_string()
            }
        }
        Stmt::Break => "    break;\n".to_string(),
        Stmt::Continue => "    continue;\n".to_string(),
        Stmt::Pass => "".to_string(),
        Stmt::Global(vars) => {
            format!("    // global {}\n", vars.join(", "))
        }
        Stmt::Nonlocal(vars) => {
            format!("    // nonlocal {}\n", vars.join(", "))
        }
        Stmt::Import { module, items, alias } => {
            if let Some(a) = alias {
                format!("    // import {} as {}\n", module, a)
            } else if let Some(items) = items {
                format!("    // from {} import {}\n", module, items.join(", "))
            } else {
                format!("    // import {}\n", module)
            }
        }
        Stmt::With { items, body } => {
            let mut code = String::new();
            for (expr, alias) in items {
                let var = alias.unwrap_or_else(|| "_".to_string());
                code.push_str(&format!("    let {} = {};\n", var, gen_expr(expr)));
            }
            for s in body {
                code.push_str(&gen_stmt(s));
            }
            code
        }
    }
}

fn indent_stmt(stmt: &str) -> String {
    stmt.lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn gen_expr(expr: Expr) -> String {
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Float(f) => f.to_string(),
        Expr::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Expr::Ident(i) => i,
        Expr::Bool(b) => if b { "true" } else { "false" }.to_string(),
        Expr::None => "None".to_string(),
        Expr::Array(els) => {
            let parts: Vec<String> = els.into_iter().map(gen_expr).collect();
            format!("vec![{}]", parts.join(", "))
        }
        Expr::Tuple(els) => {
            let parts: Vec<String> = els.into_iter().map(gen_expr).collect();
            format!("({})", parts.join(", "))
        }
        Expr::Dict(pairs) => {
            let parts: Vec<String> = pairs.iter()
                .map(|(k, v)| format!("{} => {}", gen_expr(k.clone()), gen_expr(v.clone())))
                .collect();
            format!("std::collections::HashMap::from([{}])", parts.join(", "))
        }
        Expr::Set(els) => {
            let parts: Vec<String> = els.into_iter().map(gen_expr).collect();
            format!("std::collections::HashSet::from([{}])", parts.join(", "))
        }
        Expr::BinaryOp(left, op, right) => {
            if op == "+" {
                let l = gen_expr(*left);
                let r = gen_expr(*right);
                let l_str = l.starts_with('"');
                let r_str = r.starts_with('"');
                if l_str || r_str {
                    format!("format!(\"{{}}{{}}\", {}, {})", l, r)
                } else {
                    format!("{} + {}", l, r)
                }
            } else {
                format!("({} {} {})", gen_expr(*left), op, gen_expr(*right))
            }
        }
        Expr::UnaryOp(op, expr) => {
            format!("({}{})", op, gen_expr(*expr))
        }
        Expr::Compare(left, op, right) => {
            format!("({} {} {})", gen_expr(*left), op, gen_expr(*right))
        }
        Expr::Index { array, index } => {
            format!("{}[{}]", gen_expr(*array), gen_expr(*index))
        }
        Expr::MethodCall { receiver, method, args } => {
            let args_str: Vec<String> = args.iter().map(|a| gen_expr(a.clone())).collect();
            format!("{}.{}({})", gen_expr(*receiver), method, args_str.join(", "))
        }
        Expr::Call { func, args } => {
            let fname = gen_expr(*func);
            let args_str: Vec<String> = args.iter().map(|a| gen_expr(a.clone())).collect();
            
            if fname == "print" {
                if args_str.is_empty() {
                    "()".to_string()
                } else {
                    format!("println!(\"{{}}\", {})", args_str.join(", "))
                }
            } else if fname == "range" {
                if args_str.len() == 1 {
                    format!("0..{}", args_str[0])
                } else if args_str.len() == 2 {
                    format!("{}..{}", args_str[0], args_str[1])
                } else {
                    format!("0..1")
                }
            } else if args_str.is_empty() {
                format!("{}.to_string()", fname)
            } else {
                format!("{}({}).to_string()", fname, args_str.join(", "))
            }
        }
        Expr::Lambda { args, body } => {
            let params: Vec<String> = args.iter().map(|s| s.clone()).collect();
            let body_str = gen_expr(*body);
            format!("|{}| {}", params.join(", "), body_str)
        }
        Expr::Ternary { cond, then, else_ } => {
            format!(
                "if {} {{ {} }} else {{ {} }}",
                gen_expr(*cond),
                gen_expr(*then),
                gen_expr(*else_)
            )
        }
        Expr::Input(_prompt) => {
            format!("{{ input() }}")
        }
    }
}