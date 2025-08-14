mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
use crate::ast::{Expr, BinOp};
use std::path::Path;
use std::collections::HashMap;

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
        if let Some(const_expr) = parser.parse_const_declaration() {
            exprs.push(const_expr);
        } else if let Some(print_expr) = parser.parse_console_print_expr() {
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

    // Create a constants map to track defined constants
    let mut constants = HashMap::new();
    
    for expr in &exprs {
        let result = eval(&expr, &mut constants);
        println!("Evaluation result: {}", result);
    }
}

fn eval(expr: &Expr, constants: &mut HashMap<String, i32>) -> i32 {
    match expr {
        Expr::Number(n) => *n,
        Expr::StringLiteral(_s) => {
            // Just return 0 when evaluating, don't print here
            0
        }
        Expr::BinaryOp { op, left, right } => {
            // For non-print contexts, just do regular numeric evaluation
            let l = eval(left, constants);
            let r = eval(right, constants);
            match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => l / r,
            }
        }
        Expr::Print(e) => {
            match &**e {
                Expr::StringLiteral(s) => {
                    println!("{}", s);
                    0
                },
                Expr::BinaryOp { op: BinOp::Add, left, right } => {
                    // Check if either operand is a string literal
                    let left_is_string = matches!(**left, Expr::StringLiteral(_));
                    let right_is_string = matches!(**right, Expr::StringLiteral(_));
                    
                    if left_is_string || right_is_string {
                        // Handle string concatenation
                        let left_value = convert_to_string(left, constants);
                        let right_value = convert_to_string(right, constants);
                        
                        println!("{}{}", left_value, right_value);
                        0
                    } else {
                        // Regular numeric addition
                        let value = eval(e, constants);
                        println!("{}", value);
                        value
                    }
                },
                _ => {
                    // For non-string expressions, evaluate and print the result
                    let value = eval(e, constants);
                    println!("{}", value);
                    value
                }
            }
        }
        Expr::Exit(e) => eval(e, constants),
        Expr::Const { name, value } => {
            // Check if constant already exists
            if constants.contains_key(name) {
                panic!("Error: Constant '{}' already defined", name);
            }
            let val = eval(value, constants);
            constants.insert(name.clone(), val);
            val
        }
        Expr::Variable(name) => {
            // Look up the variable
            *constants.get(name).unwrap_or_else(|| 
                panic!("Error: Undefined variable: {}", name))
        }
    }
}

// Helper function to convert any expression to a string
fn convert_to_string(expr: &Expr, constants: &HashMap<String, i32>) -> String {
    match expr {
        Expr::StringLiteral(s) => s.clone(),
        Expr::Number(n) => n.to_string(),
        Expr::Variable(name) => {
            if let Some(&value) = constants.get(name) {
                value.to_string()
            } else {
                panic!("Undefined variable: {}", name)
            }
        },
        // For complex expressions, evaluate them first
        Expr::BinaryOp { .. } => eval(expr, &mut constants.clone()).to_string(),
        _ => format!("<unsupported type>")
    }
}