mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
use ast::Expr;
use std::path::Path;
use crate::ast::BinOp;

fn main() {
    let source = match std::fs::read_to_string("examples/sample.spp") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: The file 'examples/sample.spp' could not be found.");
            return;
        }
    };

    let tokens = lex(&source);
    let mut parser = Parser::new(tokens);
    let mut exprs = Vec::new();

    // Parse all statements until tokens are exhausted
    while !parser.is_finished() {
        if let Some(print_expr) = parser.parse_console_print_expr() {
            exprs.push(print_expr);
        } else if let Some(exit_expr) = parser.parse_exit_expr() {
            exprs.push(exit_expr);
        } else {
            panic!("No valid expression found at token position {}", parser.pos());
        }
    }

    // Check if directory exists before generating file
    let output_path = Path::new("build");
    if !output_path.exists() {
        eprintln!("Error: Output directory 'build' does not exist.");
        return;
    }

    // Generate NASM code for all expressions
    codegen::generate_nasm(&exprs, Path::new("build/out.asm"));

    // Evaluate each expression in order (for debugging)
    for expr in &exprs {
        let result = eval(&expr);
        println!("Evaluation result: {}", result);
    }
}

fn eval(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => *n,
        Expr::StringLiteral(s) => {
            println!("{}", s); // Output the string literal
            0
        }
        Expr::BinaryOp { op, left, right } => {
            let l = eval(left);
            let r = eval(right);
            match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => l / r,
            }
        }
        Expr::Print(e) => {
            match **e {
                Expr::Number(n) => println!("{}", n),
                Expr::StringLiteral(ref s) => println!("{}", s),
                _ => println!("Unsupported print expression"),
            }
            0
        }
        Expr::Exit(e) => eval(e),
    }
}