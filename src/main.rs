// SPP Compiler
// Created by: Stiwyy
// Date: 2025-08-17 12:01:20

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

mod lexer;
mod parser;
mod ast;
mod codegen;

use lexer::lex;
use parser::Parser;
// use crate::ast::{Expr, BinOp}; 
// use std::collections::HashMap; 

// #[derive(Clone, Debug)]
// enum ConstValue {
//     Number(i32),
//     Float(f64),
//     String(String),
//     Boolean(bool),
//     Array(Vec<ConstValue>),
//     Null,
// }
// 
// impl std::fmt::Display for ConstValue {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             ConstValue::Number(n) => write!(f, "{}", n),
//             ConstValue::Float(n) => write!(f, "{}", n),
//             ConstValue::String(s) => write!(f, "{}", s),
//             ConstValue::Boolean(b) => write!(f, "{}", b),
//             ConstValue::Array(arr) => {
//                 write!(f, "[")?;
//                 for (i, val) in arr.iter().enumerate() {
//                     if i > 0 {
//                         write!(f, ", ")?;
//                     }
//                     write!(f, "{}", val)?;
//                 }
//                 write!(f, "]")
//             },
//             ConstValue::Null => write!(f, "null"),
//         }
//     }
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    // Parse options
    let mut input_path = "";
    let mut output_path = "";
    let mut output_dir = ".";
    
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--version" || args[i] == "-v" {
            println!("SPP Compiler v0.1.0");
            process::exit(0);
        } else if args[i] == "--help" || args[i] == "-h" {
            print_usage(&args[0]);
            process::exit(0);
        } else if args[i].starts_with("--output=") {
            output_path = &args[i]["--output=".len()..];
        } else if args[i].starts_with("-o") && args[i].len() > 2 {
            output_path = &args[i][2..];
        } else if args[i] == "-o" && i + 1 < args.len() {
            output_path = &args[i + 1];
            i += 1;
        } else if args[i].starts_with("--output-dir=") {
            output_dir = &args[i]["--output-dir=".len()..];
        } else if !args[i].starts_with("-") {
            input_path = &args[i];
        }
        i += 1;
    }
    
    if input_path.is_empty() {
        eprintln!("Error: No input file specified");
        print_usage(&args[0]);
        process::exit(1);
    }
    
    compile_file(input_path, output_path, output_dir);
    
    // // Create a constants map to track defined constants
    // let mut constants = HashMap::new();
    // // Create a variables map to track defined variables
    // let mut variables = HashMap::new();
    //
    // for expr in &exprs {
    //     let result = eval(&expr, &mut constants, &mut variables);
    //     println!("Evaluation result: {}", result);
    // }
}

fn print_usage(program_name: &str) {
    println!("Usage: {} [options] <input_file.spp>", program_name);
    println!("Options:");
    println!("  -h, --help                Display this help message");
    println!("  -v, --version             Display version information");
    println!("  -o, --output=<file>       Specify output executable name");
    println!("  --output-dir=<dir>        Specify output directory (default: current dir)");
}

fn compile_file(input_path: &str, output_path: &str, output_dir: &str) {
    // Ensure the file has .spp extension
    if !input_path.ends_with(".spp") {
        eprintln!("Error: Input file must have .spp extension");
        process::exit(1);
    }

    // Read the source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: The file '{}' could not be found or read.", input_path);
            process::exit(1);
        }
    };

    // Create output directory if it doesnt exist
    let output_dir_path = Path::new(output_dir);
    if !output_dir_path.exists() {
        fs::create_dir_all(output_dir_path).expect("Failed to create output directory");
    }

    // Determine output file name
    let exe_path = if !output_path.is_empty() {
        Path::new(output_dir).join(output_path)
    } else {
        let input_basename = Path::new(input_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        Path::new(output_dir).join(input_basename.to_string())
    };
    
    // Create temporary build directory
    let temp_dir = env::temp_dir().join(format!("spp-build-{}", process::id()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    
    // Parse source code
    let tokens = lex(&source);
    let mut parser = Parser::new(tokens);
    let mut exprs = Vec::new();

    // Parse all statements
    while !parser.is_finished() {
        if let Some(expr) = parser.parse_const_declaration() {
            exprs.push(expr);
        } else if let Some(expr) = parser.parse_let_declaration() {
            exprs.push(expr);
        } else if let Some(expr) = parser.parse_assignment() {
            exprs.push(expr);
        } else if let Some(expr) = parser.parse_console_print_expr() {
            exprs.push(expr);
        } else if let Some(expr) = parser.parse_exit_expr() {
            exprs.push(expr);
        } else if let Some(expr) = parser.parse_if_statement() {
            exprs.push(expr);
        } else {
            eprintln!("Syntax error at token position {}", parser.pos());
            process::exit(1);
        }
    }

    // Generate NASM code
    let asm_code = codegen::generate_nasm(&exprs);
    let asm_path = temp_dir.join("output.asm");
    fs::write(&asm_path, asm_code).expect("Failed to write assembly file");

    println!("Compiling {} to assembly...", input_path);
    
    // Assemble and link
    let obj_path = temp_dir.join("output.o");
    
    // Run NASM to assemble
    let nasm_status = Command::new("nasm")
        .args(["-f", "elf64", "-o"])
        .arg(&obj_path)
        .arg(&asm_path)
        .status()
        .expect("Failed to execute NASM. Is it installed?");
    
    if !nasm_status.success() {
        eprintln!("NASM assembly failed");
        fs::remove_dir_all(temp_dir).ok(); // Clean up
        process::exit(1);
    }
    
    // Run LD to link
    let ld_status = Command::new("ld")
        .arg("-o")
        .arg(&exe_path)
        .arg(&obj_path)
        .status()
        .expect("Failed to execute LD. Is it installed?");
    
    if !ld_status.success() {
        eprintln!("Linking failed");
        fs::remove_dir_all(temp_dir).ok(); // Clean up
        process::exit(1);
    }
    
    // Make the executable executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&exe_path)
            .expect("Failed to get file metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe_path, perms).expect("Failed to set file permissions");
    }
    
    // Clean up temp directory
    fs::remove_dir_all(temp_dir).ok();
    
    println!("Compilation successful. Executable created: {}", exe_path.display());
}

// fn eval(expr: &Expr, constants: &mut HashMap<String, ConstValue>, variables: &mut HashMap<String, ConstValue>) -> ConstValue {
//     match expr {
//         Expr::Number(n) => ConstValue::Number(*n),
//         Expr::Float(f) => ConstValue::Float(*f),
//         Expr::StringLiteral(s) => ConstValue::String(s.clone()),
//         Expr::Boolean(b) => ConstValue::Boolean(*b),
//         Expr::Null => ConstValue::Null,
//         Expr::Array(elements) => {
//             let mut values = Vec::new();
//             for elem in elements {
//                 values.push(eval(elem, constants, variables));
//             }
//             ConstValue::Array(values)
//         },
//         Expr::BinaryOp { op, left, right } => {
//             let l = eval(left, constants, variables);
//             let r = eval(right, constants, variables);
//             
//             // String concatenation
//             match (&l, &r) {
//                 (ConstValue::String(ls), _) => {
//                     if matches!(op, BinOp::Add) {
//                         return ConstValue::String(format!("{}{}", ls, r));
//                     }
//                 },
//                 (_, ConstValue::String(rs)) => {
//                     if matches!(op, BinOp::Add) {
//                         return ConstValue::String(format!("{}{}", l, rs));
//                     }
//                 },
//                 _ => {}
//             }
//             
//             // Numeric operations
//             match (l, r) {
//                 (ConstValue::Number(ln), ConstValue::Number(rn)) => {
//                     match op {
//                         BinOp::Add => ConstValue::Number(ln + rn),
//                         BinOp::Sub => ConstValue::Number(ln - rn),
//                         BinOp::Mul => ConstValue::Number(ln * rn),
//                         BinOp::Div => ConstValue::Number(ln / rn),
//                         BinOp::Equal => ConstValue::Boolean(ln == rn),
//                         BinOp::NotEqual => ConstValue::Boolean(ln != rn),
//                         BinOp::Lt => ConstValue::Boolean(ln < rn),
//                         BinOp::Gt => ConstValue::Boolean(ln > rn),
//                         BinOp::Lte => ConstValue::Boolean(ln <= rn),
//                         BinOp::Gte => ConstValue::Boolean(ln >= rn),
//                     }
//                 },
//                 (ConstValue::Float(lf), ConstValue::Float(rf)) => {
//                     match op {
//                         BinOp::Add => ConstValue::Float(lf + rf),
//                         BinOp::Sub => ConstValue::Float(lf - rf),
//                         BinOp::Mul => ConstValue::Float(lf * rf),
//                         BinOp::Div => ConstValue::Float(lf / rf),
//                         BinOp::Equal => ConstValue::Boolean(lf == rf),
//                         BinOp::NotEqual => ConstValue::Boolean(lf != rf),
//                         BinOp::Lt => ConstValue::Boolean(lf < rf),
//                         BinOp::Gt => ConstValue::Boolean(lf > rf),
//                         BinOp::Lte => ConstValue::Boolean(lf <= rf),
//                         BinOp::Gte => ConstValue::Boolean(lf >= rf),
//                     }
//                 },
//                 (ConstValue::Number(ln), ConstValue::Float(rf)) => {
//                     let lf = ln as f64;
//                     match op {
//                         BinOp::Add => ConstValue::Float(lf + rf),
//                         BinOp::Sub => ConstValue::Float(lf - rf),
//                         BinOp::Mul => ConstValue::Float(lf * rf),
//                         BinOp::Div => ConstValue::Float(lf / rf),
//                         BinOp::Equal => ConstValue::Boolean(lf == rf),
//                         BinOp::NotEqual => ConstValue::Boolean(lf != rf),
//                         BinOp::Lt => ConstValue::Boolean(lf < rf),
//                         BinOp::Gt => ConstValue::Boolean(lf > rf),
//                         BinOp::Lte => ConstValue::Boolean(lf <= rf),
//                         BinOp::Gte => ConstValue::Boolean(lf >= rf),
//                     }
//                 },
//                 (ConstValue::Float(lf), ConstValue::Number(rn)) => {
//                     let rf = rn as f64;
//                     match op {
//                         BinOp::Add => ConstValue::Float(lf + rf),
//                         BinOp::Sub => ConstValue::Float(lf - rf),
//                         BinOp::Mul => ConstValue::Float(lf * rf),
//                         BinOp::Div => ConstValue::Float(lf / rf),
//                         BinOp::Equal => ConstValue::Boolean(lf == rf),
//                         BinOp::NotEqual => ConstValue::Boolean(lf != rf),
//                         BinOp::Lt => ConstValue::Boolean(lf < rf),
//                         BinOp::Gt => ConstValue::Boolean(lf > rf),
//                         BinOp::Lte => ConstValue::Boolean(lf <= rf),
//                         BinOp::Gte => ConstValue::Boolean(lf >= rf),
//                     }
//                 },
//                 (ConstValue::Boolean(lb), ConstValue::Boolean(rb)) => {
//                     match op {
//                         BinOp::Equal => ConstValue::Boolean(lb == rb),
//                         BinOp::NotEqual => ConstValue::Boolean(lb != rb),
//                         _ => panic!("Unsupported operation for boolean values"),
//                     }
//                 },
//                 _ => panic!("Cannot perform operation between different types"),
//             }
//         },
//         Expr::Print(e) => {
//             // Handle printing differently based on expression type
//             let value = eval(e, constants, variables);
//             println!("{}", value);
//             value
//         },
//         Expr::Exit(e) => eval(e, constants, variables),
//         Expr::Const { name, value } => {
//             // Check if constant already exists
//             if constants.contains_key(name) {
//                 panic!("Error: Constant '{}' already defined", name);
//             }
//             let val = eval(value, constants, variables);
//             constants.insert(name.clone(), val.clone());
//             val
//         },
//         Expr::Let { name, value } => {
//             // Check if variable already exists as a constant
//             if constants.contains_key(name) {
//                 panic!("Error: Cannot declare variable '{}', a constant with the same name already exists", name);
//             }
//             
//             let val = eval(value, constants, variables);
//             // Store the value in the variables map
//             variables.insert(name.clone(), val.clone());
//             val
//         },
//         Expr::Assign { name, value } => {
//             // Check if its a constant
//             if constants.contains_key(name) {
//                 panic!("Error: Cannot reassign constant '{}'", name);
//             }
//             
//             // Check if the variable exists
//             if !variables.contains_key(name) {
//                 panic!("Error: Variable '{}' not defined before assignment", name);
//             }
//             
//             let val = eval(value, constants, variables);
//             // Update the variable with the new value
//             variables.insert(name.clone(), val.clone());
//             val
//         },
//         Expr::Variable(name) => {
//             if let Some(val) = constants.get(name) {
//                 val.clone()
//             } else if let Some(val) = variables.get(name) {
//                 val.clone()
//             } else {
//                 panic!("Error: Undefined identifier: {}", name);
//             }
//         },
//         Expr::If { condition, then_branch, else_branch } => {
//             let cond_result = eval(&**condition, constants, variables);
//             
//             let is_true = match cond_result {
//                 ConstValue::Boolean(b) => b,
//                 _ => panic!("Condition must be boolean"),
//             };
//             
//             if is_true {
//                 let mut result = ConstValue::Null;
//                 for stmt in then_branch {
//                     result = eval(&**stmt, constants, variables);
//                 }
//                 result
//             } else if let Some(else_b) = else_branch {
//                 let mut result = ConstValue::Null;
//                 for stmt in else_b {
//                     result = eval(&**stmt, constants, variables);
//                 }
//                 result
//             } else {
//                 ConstValue::Null
//             }
//         }
//     }
// }