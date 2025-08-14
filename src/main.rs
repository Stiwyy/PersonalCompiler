mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
use ast::Expr;
use std::path::Path;
use crate::ast::BinOp;
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

    // Evaluate each expression in order (for debugging)
    for expr in &exprs {
        let result = eval(&expr);
        println!("Evaluation result: {}", result);
    }
}

fn eval(expr: &Expr) -> i32 {
	// Store constants across evaluations
	static mut CONSTANTS: HashMap<String, i32> = None;

	unsafe {
		if CONSTANTS.is_none() {
			CONSTANTS = Some(HashMap::new());
		}
		let constants = CONSTANTS.as_mut().unwrap();

		match expr {
			Expr::Number(n) => *n,
			Expr::StringLiteral(s) => {
				println!("{}", s);
				0
			}
			Expr::BinaryOp { op, left, right} => {
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
					Expr::Variable(ref name) => {
						if let Some(& value) = constants.get(name) {
							println!("{}", value);
						} else {
							println!("Undefined variable: {}", name);
						}
					}
					_ => println!("Unsupported print expression"),
				}
				0
			}
			Expr::Exit(e) => eval(e),
            Expr::Const { name, value } => {
                // Check if constant already exists
                if constants.contains_key(name) {
                    panic!("Error: Constant '{}' already defined", name);
                }
                let val = eval(value);
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
}