use crate::ast::{Expr, BinOp};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

pub fn generate_nasm(exprs: &[Expr], output_path: &Path) -> String {
    let mut asm = String::new();
    let mut data_section = String::new();
    let mut text_section = String::new();
    let mut string_counter = 0;
    let mut constants: HashMap<String, i32> = HashMap::new();

    // Add digit buffer for number printing
    data_section.push_str("digit_buffer db '00000000000', 10, 0\n");

    // Collect string literals for .data section
    let mut string_labels = Vec::new();
    for expr in exprs {
        if let Expr::Print(inner) = expr {
            if let Expr::StringLiteral(s) = &**inner {
                let label = format!("msg{}", string_counter);
                string_counter += 1;
                string_labels.push((label.clone(), s));
                data_section.push_str(&format!("{} db \"{}\", 10, 0\n", label, s));
            }
        }
    }

    // Assemble .data section if we have strings
    if !data_section.is_empty() {
        asm.push_str("section .data\n");
        asm.push_str(&data_section);
        asm.push_str("\n");
    }

    // Start .text section
    asm.push_str("section .text\n");
    asm.push_str("global _start\n\n");
    asm.push_str("_start:\n");

    // Process each expression to generate corresponding assembly
    for expr in exprs {
        match expr {
            Expr::Print(inner) => {
                match &**inner {
                    Expr::StringLiteral(s) => {
                        // Find the label for this string
                        let (label, _) = string_labels.iter()
                            .find(|(_, str)| *str == s)
                            .expect("String not found in data section");
                        // Write syscall to print the string
                        text_section.push_str(&format!("    ; Print: {}\n", s));
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str(&format!("    mov rdx, {}\n", s.len() + 1)); // +1 for newline
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::Variable(name) => {
                        // Print a constant value
                        if let Some(&value) = constants.get(name) {
                            text_section.push_str(&format!("    ; Print variable: {}\n", name));
                            text_section.push_str("    mov rax, 1          ; sys_write\n");
                            text_section.push_str("    mov rdi, 1          ; stdout\n");
                            
                            // Convert number to string and add to data section
                            let var_label = format!("var_{}", name);
                            let value_str = value.to_string();
                            data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
                            
                            text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                            text_section.push_str(&format!("    mov rdx, {}\n", value_str.len() + 1)); // +1 for newline
                            text_section.push_str("    syscall\n\n");
                        } else {
                            panic!("Undefined variable: {}", name);
                        }
                    },
                    Expr::Number(n) => {
                        // Print a literal number
                        text_section.push_str(&format!("    ; Print number: {}\n", n));
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        
                        // Convert number to string and add to data section
                        let num_label = format!("num{}", string_counter);
                        string_counter += 1;
                        let num_str = n.to_string();
                        data_section.push_str(&format!("{} db \"{}\", 10, 0\n", num_label, num_str));
                        
                        text_section.push_str(&format!("    mov rsi, {}\n", num_label));
                        text_section.push_str(&format!("    mov rdx, {}\n", num_str.len() + 1)); // +1 for newline
                        text_section.push_str("    syscall\n\n");
                    },
                    _ => {
                        // Handle complex expressions (including binary operations)
                        text_section.push_str("    ; Print expression result\n");
                        
                        // Generate code to evaluate the expression
                        generate_expression_code(&**inner, &mut text_section, &constants);
                        
                        // Print the result (which will be in RAX)
                        text_section.push_str("    ; Print the evaluated expression result\n");
                        text_section.push_str("    push rax\n");
                        text_section.push_str("    call print_number\n");
                        text_section.push_str("    add rsp, 8\n\n");
                    }
                }
            },
            Expr::Exit(code) => {
                if let Expr::Number(n) = &**code {
                    text_section.push_str("    ; Exit program\n");
                    text_section.push_str("    mov rax, 60         ; sys_exit\n");
                    text_section.push_str(&format!("    mov rdi, {}\n", n));
                    text_section.push_str("    syscall\n\n");
                } else if let Expr::Variable(name) = &**code {
                    if let Some(&value) = constants.get(name) {
                        text_section.push_str(&format!("    ; Exit program with constant {}\n", name));
                        text_section.push_str("    mov rax, 60         ; sys_exit\n");
                        text_section.push_str(&format!("    mov rdi, {}\n", value));
                        text_section.push_str("    syscall\n\n");
                    } else {
                        panic!("Undefined variable in exit: {}", name);
                    }
                } else {
                    // Handle expressions in exit code
                    text_section.push_str("    ; Exit program with expression result\n");
                    generate_expression_code(&**code, &mut text_section, &constants);
                    text_section.push_str("    mov rdi, rax        ; Move result to exit code\n");
                    text_section.push_str("    mov rax, 60         ; sys_exit\n");
                    text_section.push_str("    syscall\n\n");
                }
            },
            Expr::Const { name, value } => {
                // For constants, we evaluate and store them
                match &**value {
                    Expr::Number(n) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        constants.insert(name.clone(), *n);
                        text_section.push_str(&format!("    ; Constant {} = {}\n", name, n));
                    },
                    _ => panic!("Currently only numeric constants are supported")
                }
            },
            _ => {}
        }
    }

    // Add default exit if not explicitly present
    if !exprs.iter().any(|e| matches!(e, Expr::Exit(_))) {
        text_section.push_str("    ; Default exit\n");
        text_section.push_str("    mov rax, 60\n");
        text_section.push_str("    xor rdi, rdi\n");
        text_section.push_str("    syscall\n");
    }

    // Add print_number function
    let print_number_function = "\n; Function to print a number in RAX\n\
        print_number:\n\
        mov rcx, digit_buffer\n\
        add rcx, 10             ; Point to end of buffer (before newline)\n\
        mov rbx, 10             ; Divisor\n\
        mov rax, [rsp+8]        ; Get parameter (number to print)\n\
        \n\
        ; Handle special case of 0\n\
        test rax, rax\n\
        jnz .convert_loop\n\
        mov byte [rcx], '0'\n\
        jmp .print_result\n\
        \n\
        .convert_loop:\n\
        xor rdx, rdx\n\
        div rbx                 ; Divide RAX by 10, remainder in RDX\n\
        add dl, '0'             ; Convert to ASCII\n\
        mov [rcx], dl           ; Store in buffer\n\
        dec rcx                 ; Move pointer back\n\
        test rax, rax\n\
        jnz .convert_loop       ; Continue if quotient not zero\n\
        \n\
        .print_result:\n\
        ; Calculate string length\n\
        mov rdx, digit_buffer\n\
        add rdx, 11             ; Point to end of buffer (after newline)\n\
        sub rdx, rcx            ; Calculate length\n\
        inc rcx                 ; Adjust pointer back to first digit\n\
        \n\
        ; Print the number\n\
        mov rax, 1              ; sys_write\n\
        mov rdi, 1              ; stdout\n\
        mov rsi, rcx            ; Buffer pointer\n\
        syscall\n\
        ret\n";

    
    if !data_section.is_empty() && !asm.contains("section .data") {
        // Insert data section at the beginning
        let text_part = asm.clone();
        asm.clear();
        asm.push_str("section .data\n");
        asm.push_str(&data_section);
        asm.push_str("\n");
        asm.push_str(&text_part);
    }

    asm.push_str(&text_section);
    asm.push_str(print_number_function);

    // Write to file
    if let Err(e) = fs::write(output_path, &asm) {
        eprintln!("Error writing to file: {}", e);
    }

    asm
}

// Helper function to generate expression evaluation code
fn generate_expression_code(expr: &Expr, text_section: &mut String, constants: &HashMap<String, i32>) {
    match expr {
        Expr::Number(n) => {
            text_section.push_str(&format!("    mov rax, {}\n", n));
        },
        Expr::Variable(name) => {
            if let Some(&value) = constants.get(name) {
                text_section.push_str(&format!("    mov rax, {}\n", value));
            } else {
                panic!("Undefined variable: {}", name);
            }
        },
        Expr::BinaryOp { op, left, right } => {
            // First evaluate the right expression and push result to stack
            generate_expression_code(right, text_section, constants);
            text_section.push_str("    push rax\n");
            
            // Then evaluate the left expression (result in RAX)
            generate_expression_code(left, text_section, constants);
            
            // Pop right result into RBX
            text_section.push_str("    pop rbx\n");
            
            // Perform the operation
            match op {
                BinOp::Add => text_section.push_str("    add rax, rbx\n"),
                BinOp::Sub => text_section.push_str("    sub rax, rbx\n"),
                BinOp::Mul => text_section.push_str("    imul rax, rbx\n"),
                BinOp::Div => {
                    text_section.push_str("    xor rdx, rdx\n");
                    text_section.push_str("    div rbx\n");
                },
            }
        },
        _ => panic!("Unsupported expression in code generation"),
    }
}