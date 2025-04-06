mod lexer;
mod parser;
mod ast;
mod codegen;

use parser::Parser;
use lexer::lex;

fn main() {
    let source = std::fs::read_to_string("examples/sample.spp").unwrap();
    let tokens = lex(&source);
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_exit_expr().expect("Parse error");

    let result = eval(&expr);
    println!("Exit code evaluated to: {}", result);

    codegen::generate_nasm(result, std::path::Path::new("build/out.asm"));
}
use crate::ast::{Expr, BinOp};

fn eval(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(n) => *n,
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
    }
}
