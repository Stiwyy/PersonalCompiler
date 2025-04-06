mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
use ast::{Expr, BinOp};

fn main() {
    let source = std::fs::read_to_string("examples/sample.spp").unwrap();
    let tokens = lex(&source);
    let mut parser = Parser::new(tokens);

    let expr = if let Some(print_expr) = parser.parse_console_print_expr() {
        print_expr
    } else if let Some(exit_expr) = parser.parse_exit_expr() {
        exit_expr
    } else {
        panic!("No valid expression found")
    };

    let result = eval(&expr);
    println!("Evaluation result: {}", result);
    if let Expr::Number(n) = expr {
        codegen::generate_nasm(n, std::path::Path::new("build/out.asm"));
    }
}

fn eval(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => *n,
        Expr::StringLiteral(s) => {
            println!("{}", s);
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
    }
}
