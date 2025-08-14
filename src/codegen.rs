// Mostly AI generated code for NASM assembly generation
use crate::ast::{Expr, BinOp};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

// Define a comprehensive ConstValue enum
#[derive(Clone, Debug)]
pub enum ConstValue {
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

pub fn generate_nasm(exprs: &[Expr], output_path: &Path) -> String {
    let mut asm = String::new();
    let mut data_section = String::new();
    let mut text_section = String::new();
    let mut string_counter = 0;
    let mut constants: HashMap<String, ConstValue> = HashMap::new();

    // Add digit buffer for number printing
    data_section.push_str("digit_buffer db '00000000000', 10, 0\n");
    
    // Add buffer for string operations
    data_section.push_str("str_buffer times 1024 db 0\n");
    
    // Add buffer for boolean printing
    data_section.push_str("true_str db 'true', 10, 0\n");
    data_section.push_str("false_str db 'false', 10, 0\n");
    data_section.push_str("null_str db 'null', 10, 0\n");
    data_section.push_str("array_open db '[', 0\n");
    data_section.push_str("array_close db ']', 10, 0\n");
    data_section.push_str("array_separator db ', ', 0\n");

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
                        // Find the label for this string
                        let (label, _) = string_labels.iter()
                            .find(|(_, str)| str == s)
                            .expect("String not found in data section");
                        // Write syscall to print the string
                        text_section.push_str(&format!("    ; Print: {}\n", s));
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str(&format!("    mov rdx, {}\n", s.len() + 1)); // +1 for newline
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::Boolean(b) => {
                        // Print boolean value
                        text_section.push_str(&format!("    ; Print boolean: {}\n", b));
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        if *b {
                            text_section.push_str("    mov rsi, true_str\n");
                            text_section.push_str("    mov rdx, 5       ; 'true' + newline\n");
                        } else {
                            text_section.push_str("    mov rsi, false_str\n");
                            text_section.push_str("    mov rdx, 6       ; 'false' + newline\n");
                        }
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::Null => {
                        // Print null value
                        text_section.push_str("    ; Print null\n");
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str("    mov rsi, null_str\n");
                        text_section.push_str("    mov rdx, 5          ; 'null' + newline\n");
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::Array(elements) => {
                        // Print array opening
                        text_section.push_str("    ; Print array\n");
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str("    mov rsi, array_open\n");
                        text_section.push_str("    mov rdx, 1          ; '['\n");
                        text_section.push_str("    syscall\n\n");
                        
                        // Print each element
                        for (i, elem) in elements.iter().enumerate() {
                            // Print element
                            generate_print_expr(elem, &mut text_section, &constants, &string_labels);
                            
                            // Print separator if not last element
                            if i < elements.len() - 1 {
                                text_section.push_str("    mov rax, 1          ; sys_write\n");
                                text_section.push_str("    mov rdi, 1          ; stdout\n");
                                text_section.push_str("    mov rsi, array_separator\n");
                                text_section.push_str("    mov rdx, 2          ; ', '\n");
                                text_section.push_str("    syscall\n\n");
                            }
                        }
                        
                        // Print array closing
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str("    mov rsi, array_close\n");
                        text_section.push_str("    mov rdx, 2          ; ']' + newline\n");
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::Variable(name) => {
                        // Print a constant value based on its type
                        if let Some(value) = constants.get(name) {
                            match value {
                                ConstValue::Number(n) => {
                                    text_section.push_str(&format!("    ; Print numeric variable: {}\n", name));
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    
                                    // Convert number to string and add to data section
                                    let var_label = format!("var_{}", name);
                                    let value_str = n.to_string();
                                    data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
                                    
                                    text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                                    text_section.push_str(&format!("    mov rdx, {}\n", value_str.len() + 1)); // +1 for newline
                                    text_section.push_str("    syscall\n\n");
                                },
                                ConstValue::Float(f) => {
                                    text_section.push_str(&format!("    ; Print float variable: {}\n", name));
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    
                                    // Convert float to string and add to data section
                                    let var_label = format!("var_{}", name);
                                    let value_str = f.to_string();
                                    data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
                                    
                                    text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                                    text_section.push_str(&format!("    mov rdx, {}\n", value_str.len() + 1)); // +1 for newline
                                    text_section.push_str("    syscall\n\n");
                                },
                                ConstValue::String(s) => {
                                    // Find the label for this string
                                    let var_label = format!("var_{}", name);
                                    
                                    text_section.push_str(&format!("    ; Print string variable: {}\n", name));
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                                    text_section.push_str(&format!("    mov rdx, {}\n", s.len() + 1)); // +1 for newline
                                    text_section.push_str("    syscall\n\n");
                                },
                                ConstValue::Boolean(b) => {
                                    text_section.push_str(&format!("    ; Print boolean variable: {}\n", name));
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    if *b {
                                        text_section.push_str("    mov rsi, true_str\n");
                                        text_section.push_str("    mov rdx, 5       ; 'true' + newline\n");
                                    } else {
                                        text_section.push_str("    mov rsi, false_str\n");
                                        text_section.push_str("    mov rdx, 6       ; 'false' + newline\n");
                                    }
                                    text_section.push_str("    syscall\n\n");
                                },
                                ConstValue::Array(_) => {
                                    // For arrays, print a placeholder (a full implementation would be complex)
                                    text_section.push_str(&format!("    ; Print array variable: {}\n", name));
                                    
                                    // This is a simplified version - a complete implementation would iterate through array items
                                    let var_label = format!("var_{}_label", name);
                                    data_section.push_str(&format!("{} db \"[Array]\", 10, 0\n", var_label));
                                    
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                                    text_section.push_str("    mov rdx, 8          ; '[Array]' + newline\n");
                                    text_section.push_str("    syscall\n\n");
                                },
                                ConstValue::Null => {
                                    text_section.push_str(&format!("    ; Print null variable: {}\n", name));
                                    text_section.push_str("    mov rax, 1          ; sys_write\n");
                                    text_section.push_str("    mov rdi, 1          ; stdout\n");
                                    text_section.push_str("    mov rsi, null_str\n");
                                    text_section.push_str("    mov rdx, 5          ; 'null' + newline\n");
                                    text_section.push_str("    syscall\n\n");
                                }
                            }
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
                    Expr::Float(f) => {
                        // Print a literal float
                        text_section.push_str(&format!("    ; Print float: {}\n", f));
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        
                        // Convert float to string and add to data section
                        let num_label = format!("float{}", string_counter);
                        string_counter += 1;
                        let num_str = f.to_string();
                        data_section.push_str(&format!("{} db \"{}\", 10, 0\n", num_label, num_str));
                        
                        text_section.push_str(&format!("    mov rsi, {}\n", num_label));
                        text_section.push_str(&format!("    mov rdx, {}\n", num_str.len() + 1)); // +1 for newline
                        text_section.push_str("    syscall\n\n");
                    },
                    Expr::BinaryOp { op: BinOp::Add, left, right } => {
                        // Check if either operand is a string
                        let left_is_string = is_string_expr(left, &constants);
                        let right_is_string = is_string_expr(right, &constants);
                        
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
                    if let Some(value) = constants.get(name) {
                        match value {
                            ConstValue::Number(n) => {
                                text_section.push_str(&format!("    ; Exit program with constant {}\n", name));
                                text_section.push_str("    mov rax, 60         ; sys_exit\n");
                                text_section.push_str(&format!("    mov rdi, {}\n", n));
                                text_section.push_str("    syscall\n\n");
                            },
                            ConstValue::Boolean(b) => {
                                text_section.push_str(&format!("    ; Exit program with boolean constant {}\n", name));
                                text_section.push_str("    mov rax, 60         ; sys_exit\n");
                                text_section.push_str(&format!("    mov rdi, {}\n", if *b { 1 } else { 0 }));
                                text_section.push_str("    syscall\n\n");
                            },
                            _ => panic!("Cannot use non-numeric constant in exit code: {}", name),
                        }
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
                // For constants, we evaluate and store them - support multiple types
                match &**value {
                    Expr::Number(n) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        constants.insert(name.clone(), ConstValue::Number(*n));
                        text_section.push_str(&format!("    ; Constant {} = {}\n", name, n));
                    },
                    Expr::Float(f) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        constants.insert(name.clone(), ConstValue::Float(*f));
                        text_section.push_str(&format!("    ; Constant {} = {}\n", name, f));
                    },
                    Expr::StringLiteral(s) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        
                        // Add string to data section
                        let const_label = format!("var_{}", name);
                        data_section.push_str(&format!("{} db \"{}\", 10, 0\n", const_label, s));
                        
                        // Store in constants map
                        constants.insert(name.clone(), ConstValue::String(s.clone()));
                        text_section.push_str(&format!("    ; Constant {} = \"{}\"\n", name, s));
                    },
                    Expr::Boolean(b) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        constants.insert(name.clone(), ConstValue::Boolean(*b));
                        text_section.push_str(&format!("    ; Constant {} = {}\n", name, b));
                    },
                    Expr::Array(elements) => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        
                        // Evaluate each element in the array
                        let mut values = Vec::new();
                        for elem in elements {
                            match &**elem {
                                Expr::Number(n) => values.push(ConstValue::Number(*n)),
                                Expr::Float(f) => values.push(ConstValue::Float(*f)),
                                Expr::StringLiteral(s) => values.push(ConstValue::String(s.clone())),
                                Expr::Boolean(b) => values.push(ConstValue::Boolean(*b)),
                                Expr::Null => values.push(ConstValue::Null),
                                _ => panic!("Only literal values are supported in array initialization")
                            }
                        }
                        
                        constants.insert(name.clone(), ConstValue::Array(values));
                        text_section.push_str(&format!("    ; Constant {} = [array with {} elements]\n", name, elements.len()));
                    },
                    Expr::Null => {
                        // Check if constant already exists
                        if constants.contains_key(name) {
                            panic!("Constant '{}' already defined", name);
                        }
                        constants.insert(name.clone(), ConstValue::Null);
                        text_section.push_str(&format!("    ; Constant {} = null\n", name));
                    },
                    _ => panic!("Only literal values are supported for constants")
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

    // If we added variables to the data section during processing,
    // we need to make sure the data section is included in the output
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

// Helper function to generate print code for various expression types
fn generate_print_expr(expr: &Expr, text_section: &mut String, 
                      constants: &HashMap<String, ConstValue>,
                      string_labels: &Vec<(String, String)>) {
    match expr {
        Expr::StringLiteral(s) => {
            // Find the label for this string
            let (label, _) = string_labels.iter()
                .find(|(_, str)| str == s)
                .expect("String not found in data section");
            
            text_section.push_str(&format!("    ; Print string: {}\n", s));
            text_section.push_str("    mov rax, 1          ; sys_write\n");
            text_section.push_str("    mov rdi, 1          ; stdout\n");
            text_section.push_str(&format!("    mov rsi, {}\n", label));
            text_section.push_str(&format!("    mov rdx, {}\n", s.len()));
            text_section.push_str("    syscall\n\n");
        },
        Expr::Number(n) => {
            text_section.push_str(&format!("    ; Print number: {}\n", n));
            text_section.push_str(&format!("    mov rax, {}\n", n));
            text_section.push_str("    push rax\n");
            text_section.push_str("    call print_number\n");
            text_section.push_str("    add rsp, 8\n");
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    ; Print boolean: {}\n", b));
            text_section.push_str("    mov rax, 1          ; sys_write\n");
            text_section.push_str("    mov rdi, 1          ; stdout\n");
            if *b {
                text_section.push_str("    mov rsi, true_str\n");
                text_section.push_str("    mov rdx, 4       ; 'true'\n");
            } else {
                text_section.push_str("    mov rsi, false_str\n");
                text_section.push_str("    mov rdx, 5       ; 'false'\n");
            }
            text_section.push_str("    syscall\n\n");
        },
        // Add more expression types as needed
        _ => {
            text_section.push_str("    ; Print expression result\n");
            generate_expression_code(expr, text_section, constants);
            text_section.push_str("    push rax\n");
            text_section.push_str("    call print_number\n");
            text_section.push_str("    add rsp, 8\n");
        }
    }
}

// Helper function to collect string literals
fn collect_string_literals(expr: &Expr, string_labels: &mut Vec<(String, String)>, 
                          string_counter: &mut usize, data_section: &mut String) {
    match expr {
        Expr::StringLiteral(s) => {
            // Check if we already have this string
            if !string_labels.iter().any(|(_, str)| str == s) {
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
        Expr::Array(elements) => {
            for elem in elements {
                collect_string_literals(elem, string_labels, string_counter, data_section);
            }
        },
        _ => {}
    }
}

// Helper function to check if an expression is a string or contains a string
fn is_string_expr(expr: &Expr, constants: &HashMap<String, ConstValue>) -> bool {
    match expr {
        Expr::StringLiteral(_) => true,
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                match value {
                    ConstValue::String(_) => true,
                    _ => false,
                }
            } else {
                false
            }
        },
        Expr::BinaryOp { op: BinOp::Add, left, right } => 
            is_string_expr(left, constants) || is_string_expr(right, constants),
        _ => false
    }
}

// Generate code for string concatenation
fn generate_string_concat(expr: &Expr, text_section: &mut String, 
                         constants: &HashMap<String, ConstValue>, 
                         string_labels: &Vec<(String, String)>) {
    match expr {
        Expr::StringLiteral(s) => {
            // Find the label for this string
            let (label, _) = string_labels.iter()
                .find(|(_, str)| str == s)
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
        Expr::Float(f) => {
            // For float concatenation, convert to string and store in data section
            let float_label = format!("float_concat_{}", f.to_string().replace(".", "_"));
            let _float_str = f.to_string();
            text_section.push_str(&format!("    ; Append float: {}\n", f));
			text_section.push_str(&format!("    mov rsi, {}\n", float_label));
			text_section.push_str("    call append_string\n");
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    ; Append boolean: {}\n", b));
            if *b {
                text_section.push_str("    mov rsi, true_str\n");
            } else {
                text_section.push_str("    mov rsi, false_str\n");
            }
            text_section.push_str("    call append_string\n");
        },
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                match value {
                    ConstValue::Number(n) => {
                        text_section.push_str(&format!("    ; Append numeric variable: {}\n", name));
                        text_section.push_str(&format!("    mov rax, {}\n", n));
                        text_section.push_str("    call append_number\n");
                    },
                    ConstValue::Float(f) => {
                    	text_section.push_str(&format!("    ; Append float variable: {}\n", name));
						let _float_str = f.to_string();
                        let float_label = format!("float_var_{}", name);
                        text_section.push_str(&format!("    mov rsi, {}\n", float_label));
						text_section.push_str("    call append_string\n");
                    },
                    ConstValue::String(_) => {
                        text_section.push_str(&format!("    ; Append string variable: {}\n", name));
                        text_section.push_str(&format!("    mov rsi, var_{}\n", name));
                        text_section.push_str("    call append_string\n");
                    },
                    ConstValue::Boolean(b) => {
                        text_section.push_str(&format!("    ; Append boolean variable: {}\n", name));
                        if *b {
                            text_section.push_str("    mov rsi, true_str\n");
                        } else {
                            text_section.push_str("    mov rsi, false_str\n");
                        }
                        text_section.push_str("    call append_string\n");
                    },
                    ConstValue::Array(_) => {
                        text_section.push_str(&format!("    ; Append array variable: {}\n", name));
						text_section.push_str("    mov rsi, array_open\n");
						text_section.push_str("    call append_string\n");
                        
                        // We would need more complex logic to iterate through array elements
                        // For simplicity, just append "[Array]"
                        let _array_label = format!("array_repr_{}", name);
						text_section.push_str("    mov rsi, array_close\n");
						text_section.push_str("    call append_string\n");
                    },
                    ConstValue::Null => {
                        text_section.push_str(&format!("    ; Append null variable: {}\n", name));
                        text_section.push_str("    mov rsi, null_str\n");
                        text_section.push_str("    call append_string\n");
                    }
                }
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
fn generate_expression_code(expr: &Expr, text_section: &mut String, constants: &HashMap<String, ConstValue>) {
    match expr {
        Expr::Number(n) => {
            text_section.push_str(&format!("    mov rax, {}\n", n));
        },
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                match value {
                    ConstValue::Number(n) => {
                        text_section.push_str(&format!("    mov rax, {}\n", n));
                    },
                    ConstValue::Boolean(b) => {
                        text_section.push_str(&format!("    mov rax, {}\n", if *b { 1 } else { 0 }));
                    },
                    _ => {
                        panic!("Cannot use non-numeric constant in numeric expression: {}", name);
                    }
                }
            } else {
                panic!("Undefined variable: {}", name);
            }
        },
        Expr::BinaryOp { op, left, right } => {
            // Check if this is a string operation
            if matches!(op, BinOp::Add) && (is_string_expr(left, constants) || is_string_expr(right, constants)) {
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
                BinOp::Equal => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    sete al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
                BinOp::NotEqual => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    setne al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
                BinOp::Lt => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    setl al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
                BinOp::Gt => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    setg al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
                BinOp::Lte => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    setle al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
                BinOp::Gte => {
                    text_section.push_str("    cmp rax, rbx\n");
                    text_section.push_str("    setge al\n");
                    text_section.push_str("    movzx rax, al\n");
                },
            }
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    mov rax, {}\n", if *b { 1 } else { 0 }));
        },
        _ => panic!("Unsupported expression in code generation"),
    }
}