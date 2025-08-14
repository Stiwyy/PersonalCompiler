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
    
    // Add buffer for string operations
    data_section.push_str("str_buffer times 1024 db 0\n");

    // Collect string literals for .data section
    let mut string_labels = Vec::new();
    for expr in exprs {
        collect_string_literals(expr, &mut string_labels, &mut string_counter, &mut data_section);
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
                        // Find the label for this string - FIX THE COMPARISON HERE
                        let (label, _) = string_labels.iter()
                            .find(|(_, str)| str == s)  // Changed from *str == s
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
                    Expr::BinaryOp { op: BinOp::Add, left, right } => {
                        // Check if either operand is a string
                        let left_is_string = is_string_expr(left);
                        let right_is_string = is_string_expr(right);
                        
                        if left_is_string || right_is_string {
                            // Handle string concatenation
                            text_section.push_str("    ; String concatenation\n");
                            text_section.push_str("    mov rdi, str_buffer  ; Destination buffer\n");
                            text_section.push_str("    xor rcx, rcx         ; Reset counter\n\n");
                            
                            // Process left operand
                            generate_string_concat(left, &mut text_section, &constants, &string_labels);
                            
                            // Process right operand
                            generate_string_concat(right, &mut text_section, &constants, &string_labels);
                            
                            // Add newline and null terminator
                            text_section.push_str("    mov byte [rdi], 10   ; Add newline\n");
                            text_section.push_str("    inc rdi\n");
                            text_section.push_str("    mov byte [rdi], 0    ; Add null terminator\n");
                            
                            // Print the concatenated string
                            text_section.push_str("    mov rax, 1           ; sys_write\n");
                            text_section.push_str("    mov rdi, 1           ; stdout\n");
                            text_section.push_str("    mov rsi, str_buffer  ; String buffer\n");
                            text_section.push_str("    mov rdx, rcx         ; String length + newline\n");
                            text_section.push_str("    add rdx, 1           ; Add 1 for newline\n");
                            text_section.push_str("    syscall\n\n");
                        } else {
                            // Regular numeric expression
                            text_section.push_str("    ; Print numeric expression result\n");
                            generate_expression_code(&**inner, &mut text_section, &constants);
                            text_section.push_str("    push rax\n");
                            text_section.push_str("    call print_number\n");
                            text_section.push_str("    add rsp, 8\n\n");
                        }
                    },
                    _ => {
                        // Handle complex expressions (including binary operations)
                        text_section.push_str("    ; Print expression result\n");
                        generate_expression_code(&**inner, &mut text_section, &constants);
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

    // Add string helper functions
    let string_helper_functions = "\n; Function to append a string to buffer\n\
        append_string:\n\
        ; RSI = source string, RDI = destination buffer, RCX = current length\n\
        .copy_loop:\n\
        mov al, [rsi]          ; Get character from source\n\
        test al, al            ; Check for null terminator\n\
        jz .done               ; If null, we're done\n\
        mov [rdi], al          ; Copy to destination\n\
        inc rsi                ; Advance source pointer\n\
        inc rdi                ; Advance destination pointer\n\
        inc rcx                ; Increment length counter\n\
        jmp .copy_loop         ; Continue loop\n\
        .done:\n\
        ret\n\
        \n\
        ; Function to append a number to buffer\n\
        append_number:\n\
        ; RAX = number to append, RDI = destination buffer, RCX = current length\n\
        push rdi               ; Save destination pointer\n\
        push rcx               ; Save length counter\n\
        \n\
        ; Convert number to string in reverse order (in local buffer)\n\
        mov r10, rsp           ; Use stack as temporary buffer\n\
        sub rsp, 32            ; Allocate 32 bytes on stack\n\
        mov r11, rsp           ; R11 = start of temp buffer\n\
        mov r12, 10            ; Divisor = 10\n\
        \n\
        ; Handle special case of 0\n\
        test rax, rax\n\
        jnz .convert_num_loop\n\
        mov byte [r11], '0'    ; Store '0'\n\
        inc r11                ; Advance pointer\n\
        jmp .finish_num\n\
        \n\
        .convert_num_loop:\n\
        xor rdx, rdx           ; Clear RDX for division\n\
        div r12                ; Divide RAX by 10, remainder in RDX\n\
        add dl, '0'            ; Convert to ASCII\n\
        mov [r11], dl          ; Store in buffer\n\
        inc r11                ; Advance pointer\n\
        test rax, rax          ; Check if quotient is zero\n\
        jnz .convert_num_loop  ; Continue if not\n\
        \n\
        .finish_num:\n\
        mov byte [r11], 0      ; Add null terminator\n\
        \n\
        ; Now copy the digits in reverse order to destination\n\
        pop rcx                ; Restore length counter\n\
        pop rdi                ; Restore destination pointer\n\
        \n\
        .copy_digits_loop:\n\
        dec r11                ; Move back one character\n\
        mov al, [r11]          ; Get digit\n\
        test al, al            ; Check if it's the null terminator\n\
        jz .copy_digits_done   ; If so, we're done\n\
        mov [rdi], al          ; Copy to destination\n\
        inc rdi                ; Advance destination pointer\n\
        inc rcx                ; Increment length counter\n\
        cmp r11, rsp           ; Check if we've reached the start of our buffer\n\
        jne .copy_digits_loop  ; Continue if not\n\
        \n\
        .copy_digits_done:\n\
        add rsp, 32            ; Free temporary buffer\n\
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
    asm.push_str(string_helper_functions);

    // Write to file
    if let Err(e) = fs::write(output_path, &asm) {
        eprintln!("Error writing to file: {}", e);
    }

    asm
}

// Helper function to collect string literals
fn collect_string_literals(expr: &Expr, string_labels: &mut Vec<(String, String)>, 
                          string_counter: &mut usize, data_section: &mut String) {
    match expr {
        Expr::StringLiteral(s) => {
            // Check if we already have this string - FIX THE COMPARISON HERE
            if !string_labels.iter().any(|(_, str)| str == s) {  // Changed from *str == s
                let label = format!("msg{}", *string_counter);
                *string_counter += 1;
                string_labels.push((label.clone(), s.clone()));
                data_section.push_str(&format!("{} db \"{}\", 0\n", label, s));
            }
        },
        Expr::Print(inner) => collect_string_literals(inner, string_labels, string_counter, data_section),
        Expr::Exit(inner) => collect_string_literals(inner, string_labels, string_counter, data_section),
        Expr::Const { value, .. } => collect_string_literals(value, string_labels, string_counter, data_section),
        Expr::BinaryOp { left, right, .. } => {
            collect_string_literals(left, string_labels, string_counter, data_section);
            collect_string_literals(right, string_labels, string_counter, data_section);
        },
        _ => {}
    }
}

// Helper function to check if an expression is a string or contains a string
fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::StringLiteral(_) => true,
        Expr::BinaryOp { op: BinOp::Add, left, right } => is_string_expr(left) || is_string_expr(right),
        _ => false
    }
}

// Generate code for string concatenation
fn generate_string_concat(expr: &Expr, text_section: &mut String, 
                         constants: &HashMap<String, i32>, 
                         string_labels: &Vec<(String, String)>) {
    match expr {
        Expr::StringLiteral(s) => {
            // Find the label for this string - FIX THE COMPARISON HERE
            let (label, _) = string_labels.iter()
                .find(|(_, str)| str == s)  // Changed from *str == s
                .expect("String not found in data section");
            
            text_section.push_str(&format!("    ; Append string: {}\n", s));
            text_section.push_str(&format!("    mov rsi, {}\n", label));
            text_section.push_str("    call append_string\n");
        },
        Expr::Number(n) => {
            text_section.push_str(&format!("    ; Append number: {}\n", n));
            text_section.push_str(&format!("    mov rax, {}\n", n));
            text_section.push_str("    call append_number\n");
        },
        Expr::Variable(name) => {
            if let Some(&value) = constants.get(name) {
                text_section.push_str(&format!("    ; Append variable: {}\n", name));
                text_section.push_str(&format!("    mov rax, {}\n", value));
                text_section.push_str("    call append_number\n");
            } else {
                panic!("Undefined variable: {}", name);
            }
        },
        Expr::BinaryOp { op: BinOp::Add, left, right } => {
            // Recursively process each part
            generate_string_concat(left, text_section, constants, string_labels);
            generate_string_concat(right, text_section, constants, string_labels);
        },
        _ => panic!("Unsupported expression in string concatenation")
    }
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
            // Check if this is a string operation
            if matches!(op, BinOp::Add) && (is_string_expr(left) || is_string_expr(right)) {
                panic!("String operations should be handled by generate_string_concat");
            }
            
            // First, evaluate the right expression and push result to stack
            generate_expression_code(right, text_section, constants);
            text_section.push_str("    push rax\n");
            
            // Then, evaluate the left expression (result in RAX)
            generate_expression_code(left, text_section, constants);
            
            // Pop right result into RBX
            text_section.push_str("    pop rbx\n");
            
            // Perform the operation
            match op {
                BinOp::Add => text_section.push_str("    add rax, rbx\n"),
                BinOp::Sub => text_section.push_str("    sub rax, rbx\n"),
                BinOp::Mul => text_section.push_str("    imul rax, rbx\n"),
                BinOp::Div => {
                    text_section.push_str("    xor rdx, rdx\n"); // Clear RDX for division
                    text_section.push_str("    div rbx\n");
                },
            }
        },
        _ => panic!("Unsupported expression in code generation"),
    }
}