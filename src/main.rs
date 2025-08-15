use std::env;
use std::path::Path;
use std::process;

mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
use crate::ast::{Expr, BinOp};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum ConstValue {
    Number(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<ConstValue>),
    Null,
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConstValue::Number(n) => write!(f, "{}", n),
            ConstValue::Float(n) => write!(f, "{}", n),
            ConstValue::String(s) => write!(f, "{}", s),
            ConstValue::Boolean(b) => write!(f, "{}", b),
            ConstValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            },
            ConstValue::Null => write!(f, "null"),
        }
    }
}

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if a file path was provided
    if args.len() < 2 {
        eprintln!("Error: No input file specified");
        eprintln!("Usage: {} <input_file.spp>", args[0]);
        process::exit(1);
    }
    
    // Use the provided file path
    let input_path = &args[1];
    
    // Read the source file
    let source = match std::fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: The file '{}' could not be found or read.", input_path);
            process::exit(1);
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
        process::exit(1);
    }

    // Generate NASM code for all expressions
    let asm_code = codegen::generate_nasm(&exprs);
    std::fs::write(Path::new("build/out.asm"), asm_code).expect("Failed to write assembly file");

    // Create a constants map to track defined constants
    let mut constants = HashMap::new();
    
    for expr in &exprs {
        let result = eval(&expr, &mut constants);
        println!("Evaluation result: {}", result);
    }
}

fn eval(expr: &Expr, constants: &mut HashMap<String, ConstValue>) -> ConstValue {
    match expr {
        Expr::Number(n) => ConstValue::Number(*n),
        Expr::Float(f) => ConstValue::Float(*f),
        Expr::StringLiteral(s) => ConstValue::String(s.clone()),
        Expr::Boolean(b) => ConstValue::Boolean(*b),
        Expr::Null => ConstValue::Null,
        Expr::Array(elements) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(eval(elem, constants));
            }
            ConstValue::Array(values)
        },
        Expr::BinaryOp { op, left, right } => {
            let l = eval(left, constants);
            let r = eval(right, constants);
            
            // String concatenation
            match (&l, &r) {
                (ConstValue::String(ls), _) => {
                    if matches!(op, BinOp::Add) {
                        return ConstValue::String(format!("{}{}", ls, r));
                    }
                },
                (_, ConstValue::String(rs)) => {
                    if matches!(op, BinOp::Add) {
                        return ConstValue::String(format!("{}{}", l, rs));
                    }
                },
                _ => {}
            }
            
            // Numeric operations
            match (l, r) {
                (ConstValue::Number(ln), ConstValue::Number(rn)) => {
                    match op {
                        BinOp::Add => ConstValue::Number(ln + rn),
                        BinOp::Sub => ConstValue::Number(ln - rn),
                        BinOp::Mul => ConstValue::Number(ln * rn),
                        BinOp::Div => ConstValue::Number(ln / rn),
                        BinOp::Equal => ConstValue::Boolean(ln == rn),
                        BinOp::NotEqual => ConstValue::Boolean(ln != rn),
                        BinOp::Lt => ConstValue::Boolean(ln < rn),
                        BinOp::Gt => ConstValue::Boolean(ln > rn),
                        BinOp::Lte => ConstValue::Boolean(ln <= rn),
                        BinOp::Gte => ConstValue::Boolean(ln >= rn),
                    }
                },
                (ConstValue::Float(lf), ConstValue::Float(rf)) => {
                    match op {
                        BinOp::Add => ConstValue::Float(lf + rf),
                        BinOp::Sub => ConstValue::Float(lf - rf),
                        BinOp::Mul => ConstValue::Float(lf * rf),
                        BinOp::Div => ConstValue::Float(lf / rf),
                        BinOp::Equal => ConstValue::Boolean(lf == rf),
                        BinOp::NotEqual => ConstValue::Boolean(lf != rf),
                        BinOp::Lt => ConstValue::Boolean(lf < rf),
                        BinOp::Gt => ConstValue::Boolean(lf > rf),
                        BinOp::Lte => ConstValue::Boolean(lf <= rf),
                        BinOp::Gte => ConstValue::Boolean(lf >= rf),
                    }
                },
                (ConstValue::Number(ln), ConstValue::Float(rf)) => {
                    let lf = ln as f64;
                    match op {
                        BinOp::Add => ConstValue::Float(lf + rf),
                        BinOp::Sub => ConstValue::Float(lf - rf),
                        BinOp::Mul => ConstValue::Float(lf * rf),
                        BinOp::Div => ConstValue::Float(lf / rf),
                        BinOp::Equal => ConstValue::Boolean(lf == rf),
                        BinOp::NotEqual => ConstValue::Boolean(lf != rf),
                        BinOp::Lt => ConstValue::Boolean(lf < rf),
                        BinOp::Gt => ConstValue::Boolean(lf > rf),
                        BinOp::Lte => ConstValue::Boolean(lf <= rf),
                        BinOp::Gte => ConstValue::Boolean(lf >= rf),
                    }
                },
                (ConstValue::Float(lf), ConstValue::Number(rn)) => {
                    let rf = rn as f64;
                    match op {
                        BinOp::Add => ConstValue::Float(lf + rf),
                        BinOp::Sub => ConstValue::Float(lf - rf),
                        BinOp::Mul => ConstValue::Float(lf * rf),
                        BinOp::Div => ConstValue::Float(lf / rf),
                        BinOp::Equal => ConstValue::Boolean(lf == rf),
                        BinOp::NotEqual => ConstValue::Boolean(lf != rf),
                        BinOp::Lt => ConstValue::Boolean(lf < rf),
                        BinOp::Gt => ConstValue::Boolean(lf > rf),
                        BinOp::Lte => ConstValue::Boolean(lf <= rf),
                        BinOp::Gte => ConstValue::Boolean(lf >= rf),
                    }
                },
                (ConstValue::Boolean(lb), ConstValue::Boolean(rb)) => {
                    match op {
                        BinOp::Equal => ConstValue::Boolean(lb == rb),
                        BinOp::NotEqual => ConstValue::Boolean(lb != rb),
                        _ => panic!("Unsupported operation for boolean values"),
                    }
                },
                _ => panic!("Cannot perform operation between different types"),
            }
        },
        Expr::Print(e) => {
            // Handle printing differently based on expression type
            let value = eval(e, constants);
            println!("{}", value);
            value
        },
        Expr::Exit(e) => eval(e, constants),
        Expr::Const { name, value } => {
            // Check if constant already exists
            if constants.contains_key(name) {
                panic!("Error: Constant '{}' already defined", name);
            }
            let val = eval(value, constants);
            constants.insert(name.clone(), val.clone());
            val
        },
        Expr::Variable(name) => {
            // Look up the variable
            constants.get(name)
                .cloned()
                .unwrap_or_else(|| panic!("Error: Undefined variable: {}", name))
        },
    }
}
