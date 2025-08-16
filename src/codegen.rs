use std::collections::HashMap;
use crate::ast::{Expr, BinOp};

// Define the ConstValue enum to store different types of constants
#[derive(Clone)]
pub enum ConstValue {
    Number(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<ConstValue>),
    Null,
}

// Helper function to ensure consistent label naming
fn get_var_label(name: &str, suffix: Option<&str>) -> String {
    if let Some(suffix) = suffix {
        format!("var_{}{}", name, suffix)
    } else {
        format!("var_{}", name)
    }
}

// Add a function to generate unique string labels
fn get_string_label(counter: &mut usize, prefix: &str) -> String {
    let label = format!("{}_{}", prefix, counter);
    *counter += 1;
    label
}

// Generate NASM assembly from parsed expressions
pub fn generate_nasm(exprs: &Vec<Expr>) -> String {
    let mut constants: HashMap<String, ConstValue> = HashMap::new();
    let mut variables: HashMap<String, ConstValue> = HashMap::new();
    let mut string_counter = 0;
    
    let mut text_section = String::from("section .text\n");
    text_section.push_str("global _start\n\n");
    
    // Add helper functions
    text_section.push_str("print_number:\n");
    text_section.push_str("    ; Print a number from the stack\n");
    text_section.push_str("    pop rax      ; Return address\n");
    text_section.push_str("    pop rdi      ; Number to print\n");
    text_section.push_str("    push rax     ; Save return address\n");
    
    text_section.push_str("    mov rsi, buffer\n");
    text_section.push_str("    mov rax, rdi\n");
    text_section.push_str("    mov rbx, 10\n");
    
    text_section.push_str("    ; Handle negative numbers\n");
    text_section.push_str("    test rax, rax\n");
    text_section.push_str("    jns .positive\n");
    text_section.push_str("    neg rax\n");
    text_section.push_str("    mov byte [rsi], '-'\n");
    text_section.push_str("    inc rsi\n");
    
    text_section.push_str(".positive:\n");
    text_section.push_str("    ; Convert to string (reversed)\n");
    text_section.push_str("    mov rcx, buffer_end\n");
    text_section.push_str("    mov byte [rcx], 10   ; Newline\n");
    text_section.push_str("    dec rcx\n");
    
    text_section.push_str(".digit_loop:\n");
    text_section.push_str("    xor rdx, rdx\n");
    text_section.push_str("    div rbx\n");
    text_section.push_str("    add dl, '0'\n");
    text_section.push_str("    mov [rcx], dl\n");
    text_section.push_str("    dec rcx\n");
    text_section.push_str("    test rax, rax\n");
    text_section.push_str("    jnz .digit_loop\n");
    
    text_section.push_str("    ; Calculate string length\n");
    text_section.push_str("    lea rsi, [rcx+1]\n");
    text_section.push_str("    mov rdx, buffer_end\n");
    text_section.push_str("    sub rdx, rsi\n");
    text_section.push_str("    inc rdx       ; Include newline\n");
    
    text_section.push_str("    ; Print the number\n");
    text_section.push_str("    mov rax, 1    ; sys_write\n");
    text_section.push_str("    mov rdi, 1    ; stdout\n");
    text_section.push_str("    syscall\n");
    
    text_section.push_str("    ret\n\n");
    
    // Add string concatenation helper functions
    text_section.push_str("append_string:\n");
    text_section.push_str("    ; Append a string (in RSI) to buffer (in RDI)\n");
    text_section.push_str("    ; RDI is the current position in buffer\n");
    text_section.push_str("    ; RCX is the total length so far\n");
    text_section.push_str(".loop:\n");
    text_section.push_str("    mov al, [rsi]\n");
    text_section.push_str("    test al, al\n");
    text_section.push_str("    jz .done\n");
    text_section.push_str("    mov [rdi], al\n");
    text_section.push_str("    inc rsi\n");
    text_section.push_str("    inc rdi\n");
    text_section.push_str("    inc rcx\n");
    text_section.push_str("    jmp .loop\n");
    text_section.push_str(".done:\n");
    text_section.push_str("    ret\n\n");
    
    text_section.push_str("append_string_without_newline:\n");
    text_section.push_str("    ; Append a string (in RSI) to buffer (in RDI) but skip newlines\n");
    text_section.push_str("    ; RDI is the current position in buffer\n");
    text_section.push_str("    ; RCX is the total length so far\n");
    text_section.push_str(".loop:\n");
    text_section.push_str("    mov al, [rsi]\n");
    text_section.push_str("    test al, al\n");
    text_section.push_str("    jz .done\n");
    text_section.push_str("    cmp al, 10    ; Check for newline\n");
    text_section.push_str("    je .skip\n");
    text_section.push_str("    mov [rdi], al\n");
    text_section.push_str("    inc rdi\n");
    text_section.push_str("    inc rcx\n");
    text_section.push_str(".skip:\n");
    text_section.push_str("    inc rsi\n");
    text_section.push_str("    jmp .loop\n");
    text_section.push_str(".done:\n");
    text_section.push_str("    ret\n\n");
    
    text_section.push_str("append_number:\n");
    text_section.push_str("    ; Append a number (in RAX) to buffer (in RDI)\n");
    text_section.push_str("    ; RAX is the number to append\n");
    text_section.push_str("    ; RDI is the current position in buffer\n");
    text_section.push_str("    ; RCX is the total length so far\n");
    text_section.push_str("    push rbx\n");
    text_section.push_str("    push rdx\n");
    text_section.push_str("    push r8\n");
    text_section.push_str("    push r9\n");
    text_section.push_str("    push rsi\n");
    
    text_section.push_str("    ; Handle negative numbers\n");
    text_section.push_str("    test rax, rax\n");
    text_section.push_str("    jns .positive\n");
    text_section.push_str("    neg rax\n");
    text_section.push_str("    mov byte [rdi], '-'\n");
    text_section.push_str("    inc rdi\n");
    text_section.push_str("    inc rcx\n");
    
    text_section.push_str(".positive:\n");
    text_section.push_str("    mov rsi, rax\n");
    text_section.push_str("    mov rax, rsi\n");
    
    text_section.push_str("    ; Count digits\n");
    text_section.push_str("    xor rdx, rdx\n");
    text_section.push_str("    mov rbx, 10\n");
    text_section.push_str("    mov r8, rdi      ; Save buffer position\n");
    text_section.push_str("    mov r9, 0        ; Digit counter\n");
    
    text_section.push_str(".count_loop:\n");
    text_section.push_str("    inc r9\n");
    text_section.push_str("    xor rdx, rdx\n");
    text_section.push_str("    div rbx\n");
    text_section.push_str("    test rax, rax\n");
    text_section.push_str("    jnz .count_loop\n");
    
    text_section.push_str("    ; r9 now has digit count, rsi has original number\n");
    text_section.push_str("    add r8, r9       ; r8 now points to end of string\n");
    text_section.push_str("    dec r8\n");
    text_section.push_str("    mov rax, rsi\n");
    
    text_section.push_str(".convert_loop:\n");
    text_section.push_str("    xor rdx, rdx\n");
    text_section.push_str("    div rbx\n");
    text_section.push_str("    add dl, '0'\n");
    text_section.push_str("    mov [r8], dl\n");
    text_section.push_str("    dec r8\n");
    text_section.push_str("    test rax, rax\n");
    text_section.push_str("    jnz .convert_loop\n");
    
    text_section.push_str("    ; Update buffer position and length\n");
    text_section.push_str("    add rdi, r9\n");
    text_section.push_str("    add rcx, r9\n");
    
    text_section.push_str("    pop rsi\n");
    text_section.push_str("    pop r9\n");
    text_section.push_str("    pop r8\n");
    text_section.push_str("    pop rdx\n");
    text_section.push_str("    pop rbx\n");
    text_section.push_str("    ret\n\n");
    
    text_section.push_str("_start:\n");
    
    let mut data_section = String::from("section .data\n");
    data_section.push_str("true_str db \"true\", 10, 0\n");
    data_section.push_str("false_str db \"false\", 10, 0\n");
    data_section.push_str("null_str db \"null\", 10, 0\n");
    data_section.push_str("array_open db \"[\", 0\n");
    data_section.push_str("array_close db \"]\", 10, 0\n");
    data_section.push_str("array_separator db \", \", 0\n");
    
    let mut bss_section = String::from("section .bss\n");
    bss_section.push_str("buffer: resb 32\n");
    bss_section.push_str("buffer_end: resb 1\n");
    bss_section.push_str("str_buffer: resb 1024\n");
    
    let mut string_labels: HashMap<String, String> = HashMap::new();
    
    // First pass collect all string literals
    for expr in exprs {
        collect_string_literals(expr, &mut string_counter, &mut string_labels, &mut data_section);
    }
    
    // Define helper functions for code reuse
    fn process_statements(statements: &Vec<Box<Expr>>, text_section: &mut String, data_section: &mut String, 
                     bss_section: &mut String, constants: &mut HashMap<String, ConstValue>, 
                     variables: &mut HashMap<String, ConstValue>, 
                     string_labels: &HashMap<String, String>, 
                     string_counter: &mut usize) {
		for stmt in statements {
			process_statement(&**stmt, text_section, data_section, bss_section, 
							constants, variables, string_labels, string_counter);
		}
	}
    
    fn process_statement(expr: &Expr, text_section: &mut String, data_section: &mut String, bss_section: &mut String, constants: &mut HashMap<String, ConstValue>, variables: &mut HashMap<String, ConstValue>, string_labels: &HashMap<String, String>, string_counter: &mut usize) {
        match expr {
            Expr::Print(inner) => {
                generate_print_code(&**inner, text_section, constants, variables, string_labels, data_section, string_counter);
            },
            Expr::Exit(code) => {
                generate_exit_code(&**code, text_section, constants, variables);
            },
            Expr::Const { name, value } => {
                generate_const_code(name, &**value, text_section, data_section, constants, variables, string_counter);
            },
            Expr::Let { name, value } => {
                generate_let_code(name, &**value, text_section, data_section, bss_section, constants, variables, string_counter);
            },
            Expr::Assign { name, value } => {
                generate_assign_code(name, &**value, text_section, data_section, constants, variables, string_counter);
            },
            Expr::If { condition, then_branch, else_branch } => {
				let mut local_counter = *string_counter;
				let label_end = format!("if_end_{}", local_counter);
				local_counter += 1;
				let label_else = if else_branch.is_some() { format!("if_else_{}", local_counter) } else { label_end.clone() };
				if else_branch.is_some() { local_counter += 1; }
				*string_counter = local_counter;
				
				text_section.push_str("    ; If-Statement (condition evaluation)\n");
				generate_expression_code(&**condition, text_section, constants, variables);
				text_section.push_str("    test rax, rax\n");
				text_section.push_str(&format!("    jz {}\n", label_else));
				
				text_section.push_str("    ; Then-Branch\n");
				process_statements(then_branch, text_section, data_section, bss_section, constants, variables, string_labels, string_counter);
				
				if let Some(else_b) = else_branch {
					text_section.push_str(&format!("    jmp {}\n", label_end));
					text_section.push_str(&format!("{}:\n", label_else));
					text_section.push_str("    ; Else-Branch\n");
					process_statements(else_b, text_section, data_section, bss_section, constants, variables, string_labels, string_counter);
				}
				
				text_section.push_str(&format!("{}:\n", label_end));
			},
            _ => {
                // Handle as expression if applicable
                text_section.push_str("    ; Expression evaluation\n");
                generate_expression_code(expr, text_section, constants, variables);
            }
        }
    }
    
    fn generate_print_code(inner: &Expr, text_section: &mut String, constants: &HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>, string_labels: &HashMap<String, String>, data_section: &mut String, string_counter: &mut usize) {
        match inner {
            Expr::StringLiteral(s) => {
                let label = string_labels.get(s).expect("String label not found");
                text_section.push_str(&format!("    ; Print: {}\n", s));
                text_section.push_str("    mov rax, 1          ; sys_write\n");
                text_section.push_str("    mov rdi, 1          ; stdout\n");
                text_section.push_str(&format!("    mov rsi, {}\n", label));
                text_section.push_str(&format!("    mov rdx, {}\n", s.len() + 1));
                text_section.push_str("    syscall\n\n");
            },
            Expr::Boolean(b) => {
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
                text_section.push_str("    ; Print null\n");
                text_section.push_str("    mov rax, 1          ; sys_write\n");
                text_section.push_str("    mov rdi, 1          ; stdout\n");
                text_section.push_str("    mov rsi, null_str\n");
                text_section.push_str("    mov rdx, 5          ; 'null' + newline\n");
                text_section.push_str("    syscall\n\n");
            },
            Expr::Array(elements) => {
                text_section.push_str("    ; Print array\n");
                text_section.push_str("    mov rax, 1          ; sys_write\n");
                text_section.push_str("    mov rdi, 1          ; stdout\n");
                text_section.push_str("    mov rsi, array_open\n");
                text_section.push_str("    mov rdx, 1          ; '['\n");
                text_section.push_str("    syscall\n\n");
                
                for (i, elem) in elements.iter().enumerate() {
                    print_array_element(elem, text_section, constants, variables, string_labels, data_section, string_counter);
                    
                    if i < elements.len() - 1 {
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str("    mov rsi, array_separator\n");
                        text_section.push_str("    mov rdx, 2          ; ', '\n");
                        text_section.push_str("    syscall\n\n");
                    }
                }
                
                text_section.push_str("    mov rax, 1          ; sys_write\n");
                text_section.push_str("    mov rdi, 1          ; stdout\n");
                text_section.push_str("    mov rsi, array_close\n");
                text_section.push_str("    mov rdx, 2          ; ']' + newline\n");
                text_section.push_str("    syscall\n\n");
            },
            Expr::Variable(name) => {
                let in_constants = constants.contains_key(name);
                let in_variables = variables.contains_key(name);
                if !in_constants && !in_variables {
                    panic!("Undefined variable: {}", name);
                }
                let value = if in_constants {
                    constants.get(name).unwrap()
                } else {
                    variables.get(name).unwrap()
                };
                let var_label = get_var_label(name, None);
                text_section.push_str(&format!("    ; Print variable: {}\n", name));
                text_section.push_str("    mov rdi, str_buffer  ; Destination buffer\n");
                text_section.push_str("    xor rcx, rcx         ; Reset counter\n");
                text_section.push_str("    xor rax, rax         ; Zero for stosb\n");
                text_section.push_str("    mov rcx, 1024        ; Buffer size\n");
                text_section.push_str("    cld                  ; Clear direction flag\n");
                text_section.push_str("    rep stosb           ; Fill buffer with zeros\n");
                text_section.push_str("    mov rdi, str_buffer  ; Reset destination buffer\n");
                text_section.push_str("    xor rcx, rcx         ; Reset counter\n\n");
                match value {
                    ConstValue::String(_) => {
                        if in_constants {
                            text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                        } else {
                            text_section.push_str(&format!("    mov rsi, [var_mem_{}]\n", name));
                        }
                        text_section.push_str("    call append_string_without_newline\n");
                    },
                    ConstValue::Number(n) => {
                        if in_constants {
                            text_section.push_str(&format!("    mov rax, {}\n", n));
                        } else {
                            text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                        }
                        text_section.push_str("    call append_number\n");
                    },
                    ConstValue::Float(_) => {
                        let float_label = get_var_label(name, Some("_float"));
                        if in_constants {
                            text_section.push_str(&format!("    mov rsi, {}\n", float_label));
                        } else {
                            text_section.push_str(&format!("    mov rsi, [var_mem_{}_float]\n", name));
                        }
                        text_section.push_str("    call append_string_without_newline\n");
                    },
                    ConstValue::Boolean(b) => {
                        if in_constants {
                            if *b {
                                text_section.push_str("    mov rsi, true_str\n");
                            } else {
                                text_section.push_str("    mov rsi, false_str\n");
                            }
                        } else {
                            text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                            text_section.push_str("    cmp rax, 0\n");
                            text_section.push_str(&format!("    je .false_{}\n", name));
                            text_section.push_str("    mov rsi, true_str\n");
                            text_section.push_str(&format!("    jmp .done_{}\n", name));
                            text_section.push_str(&format!(".false_{}:\n", name));
                            text_section.push_str("    mov rsi, false_str\n");
                            text_section.push_str(&format!(".done_{}:\n", name));
                        }
                        text_section.push_str("    call append_string_without_newline\n");
                    },
                    ConstValue::Null => {
                        text_section.push_str("    mov rsi, null_str\n");
                        text_section.push_str("    call append_string_without_newline\n");
                    },
                    ConstValue::Array(_) => {
                        let array_label = get_var_label(name, Some("_label"));
                        text_section.push_str(&format!("    mov rsi, {}\n", array_label));
                        text_section.push_str("    call append_string_without_newline\n");
                    },
                }
                text_section.push_str("    mov byte [rdi], 10   ; Add newline\n");
                text_section.push_str("    inc rdi\n");
                text_section.push_str("    inc rcx              ; Count the newline\n");
                text_section.push_str("    mov byte [rdi], 0    ; Add null terminator\n");
                text_section.push_str("    mov rax, 1           ; sys_write\n");
                text_section.push_str("    mov rdi, 1           ; stdout\n");
                text_section.push_str("    mov rsi, str_buffer  ; String buffer\n");
                text_section.push_str("    mov rdx, rcx         ; String length\n");
                text_section.push_str("    syscall\n\n");
            },
            Expr::Number(n) => {
                text_section.push_str(&format!("    ; Print number: {}\n", n));
                text_section.push_str(&format!("    mov rax, {}\n", n));
                text_section.push_str("    push rax\n");
                text_section.push_str("    call print_number\n");
                text_section.push_str("    add rsp, 8\n\n");
            },
            Expr::Float(f) => {
                let num_label = format!("float_{}", *string_counter);
                *string_counter += 1;
                let num_str = f.to_string();
                data_section.push_str(&format!("{} db \"{}\", 10, 0\n", num_label, num_str));
                text_section.push_str(&format!("    ; Print float: {}\n", f));
                text_section.push_str("    mov rax, 1          ; sys_write\n");
                text_section.push_str("    mov rdi, 1          ; stdout\n");
                text_section.push_str(&format!("    mov rsi, {}\n", num_label));
                text_section.push_str(&format!("    mov rdx, {}\n", num_str.len() + 1));
                text_section.push_str("    syscall\n\n");
            },
            Expr::BinaryOp { op, left, right } => {
                if *op == BinOp::Add && (is_string_expr(left, constants, variables) || is_string_expr(right, constants, variables)) {
                    text_section.push_str("    mov rdi, str_buffer  ; Destination buffer\n");
                    text_section.push_str("    xor rcx, rcx         ; Reset counter\n");
                    text_section.push_str("    xor rax, rax         ; Zero for stosb\n");
                    text_section.push_str("    mov rcx, 1024        ; Buffer size\n");
                    text_section.push_str("    cld                  ; Clear direction flag\n");
                    text_section.push_str("    rep stosb           ; Fill buffer with zeros\n");
                    text_section.push_str("    mov rdi, str_buffer  ; Reset destination buffer\n");
                    text_section.push_str("    xor rcx, rcx         ; Reset counter\n\n");
                    
                    generate_string_concat(left, text_section, constants, variables, string_labels, data_section, string_counter);
                    
                    generate_string_concat(right, text_section, constants, variables, string_labels, data_section, string_counter);
                    
                    text_section.push_str("    mov byte [rdi], 10   ; Add newline\n");
                    text_section.push_str("    inc rdi\n");
                    text_section.push_str("    inc rcx              ; Count the newline\n");
                    text_section.push_str("    mov byte [rdi], 0    ; Add null terminator\n");
                    text_section.push_str("    mov rax, 1           ; sys_write\n");
                    text_section.push_str("    mov rdi, 1           ; stdout\n");
                    text_section.push_str("    mov rsi, str_buffer  ; String buffer\n");
                    text_section.push_str("    mov rdx, rcx         ; String length\n");
                    text_section.push_str("    syscall\n\n");
                } else {
                    text_section.push_str("    ; Print numeric expression result\n");
                    generate_expression_code(inner, text_section, constants, variables);
                    text_section.push_str("    push rax\n");
                    text_section.push_str("    call print_number\n");
                    text_section.push_str("    add rsp, 8\n\n");
                }
            },
            _ => {
                text_section.push_str("    ; Print expression result\n");
                generate_expression_code(inner, text_section, constants, variables);
                text_section.push_str("    push rax\n");
                text_section.push_str("    call print_number\n");
                text_section.push_str("    add rsp, 8\n\n");
            }
        }
    }
    
    fn generate_exit_code(code: &Expr, text_section: &mut String, constants: &HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>) {
        match code {
            Expr::Number(n) => {
                text_section.push_str("    ; Exit program\n");
                text_section.push_str("    mov rax, 60         ; sys_exit\n");
                text_section.push_str(&format!("    mov rdi, {}\n", n));
                text_section.push_str("    syscall\n\n");
            },
            Expr::Variable(name) => {
                if constants.contains_key(name) || variables.contains_key(name) {
                    text_section.push_str(&format!("    ; Exit program with variable {}\n", name));
                    text_section.push_str("    mov rax, 60         ; sys_exit\n");
                    
                    if let Some(value) = constants.get(name) {
                        match value {
                            ConstValue::Number(n) => {
                                text_section.push_str(&format!("    mov rdi, {}\n", n));
                            },
                            ConstValue::Boolean(b) => {
                                text_section.push_str(&format!("    mov rdi, {}\n", if *b { 1 } else { 0 }));
                            },
                            _ => {
                                text_section.push_str("    mov rdi, 0      ; Non-numeric value defaults to 0\n");
                            }
                        }
                    } else if let Some(value) = variables.get(name) {
                        match value {
                            ConstValue::Number(_) | ConstValue::Boolean(_) => {
                                text_section.push_str(&format!("    mov rdi, [var_mem_{}]\n", name));
                            },
                            _ => {
                                text_section.push_str("    mov rdi, 0      ; Non-numeric value defaults to 0\n");
                            }
                        }
                    }
                    text_section.push_str("    syscall\n\n");
                } else {
                    panic!("Undefined variable in exit: {}", name);
                }
            },
            _ => {
                text_section.push_str("    ; Exit program with expression result\n");
                generate_expression_code(code, text_section, constants, variables);
                text_section.push_str("    mov rdi, rax        ; Move result to exit code\n");
                text_section.push_str("    mov rax, 60         ; sys_exit\n");
                text_section.push_str("    syscall\n\n");
            }
        }
    }

	fn generate_const_code(name: &str, value: &Expr, text_section: &mut String, data_section: &mut String, 
                      constants: &mut HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>,
                      _string_counter: &mut usize) {
		if constants.contains_key(name) {
			panic!("Constant '{}' already defined", name);
		}
		
		let const_value = evaluate_constant_expr(value, constants, variables);
		constants.insert(name.to_string(), const_value.clone());
		
		match const_value {
			ConstValue::Number(n) => {
				let var_label = get_var_label(name, None);
				let value_str = n.to_string();
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
				text_section.push_str(&format!("    ; Constant {} = {}\n", name, n));
			},
			ConstValue::Float(f) => {
				let var_label = get_var_label(name, None);
				let float_label = get_var_label(name, Some("_float"));
				let float_str = f.to_string();
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, float_str));
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", float_label, float_str));
				text_section.push_str(&format!("    ; Constant {} = {}\n", name, f));
			},
			ConstValue::String(s) => {
				let var_label = get_var_label(name, None);
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, s));
				text_section.push_str(&format!("    ; Constant {} = \"{}\"\n", name, s));
			},
			ConstValue::Boolean(b) => {
				let var_label = get_var_label(name, None);
				let value_str = if b { "true" } else { "false" };
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
				text_section.push_str(&format!("    ; Constant {} = {}\n", name, b));
			},
			ConstValue::Array(values) => {
				let var_label = get_var_label(name, Some("_label"));
				data_section.push_str(&format!("{} db \"[Array]\", 10, 0\n", var_label));
				text_section.push_str(&format!("    ; Constant {} = [array with {} elements]\n", name, values.len()));
			},
			ConstValue::Null => {
				let var_label = get_var_label(name, None);
				data_section.push_str(&format!("{} db \"null\", 10, 0\n", var_label));
				text_section.push_str(&format!("    ; Constant {} = null\n", name));
			},
		}
	}

	fn generate_let_code(name: &str, value: &Expr, text_section: &mut String, data_section: &mut String, 
                    bss_section: &mut String, constants: &HashMap<String, ConstValue>, 
                    variables: &mut HashMap<String, ConstValue>, _string_counter: &mut usize) {
		if constants.contains_key(name) {
			panic!("Cannot declare variable '{}', a constant with the same name already exists", name);
		}
		if variables.contains_key(name) {
			panic!("Variable '{}' already defined", name);
		}
		
		let var_value = evaluate_constant_expr(value, constants, variables);
		variables.insert(name.to_string(), var_value.clone());
		
		match var_value {
			ConstValue::Number(n) => {
				let var_label = get_var_label(name, None);
				let value_str = n.to_string();
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
				text_section.push_str(&format!("    ; Variable {} = {}\n", name, n));
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, n));
			},
			ConstValue::Float(f) => {
				let var_label = get_var_label(name, None);
				let float_label = get_var_label(name, Some("_float"));
				let float_str = f.to_string();
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, float_str));
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", float_label, float_str));
				bss_section.push_str(&format!("var_mem_{}_float: resq 1  ; String rep for float {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}_float], {}\n", name, float_label));
				text_section.push_str(&format!("    ; Variable {} = {}\n", name, f));
				let int_val = (f * 100.0) as i64;
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, int_val));
			},
			ConstValue::String(s) => {
				let var_label = get_var_label(name, None);
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, s));
				text_section.push_str(&format!("    ; Variable {} = \"{}\"\n", name, s));
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, var_label));
			},
			ConstValue::Boolean(b) => {
				let var_label = get_var_label(name, None);
				let value_str = if b { "true" } else { "false" };
				data_section.push_str(&format!("{} db \"{}\", 10, 0\n", var_label, value_str));
				text_section.push_str(&format!("    ; Variable {} = {}\n", name, b));
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, if b { 1 } else { 0 }));
			},
			ConstValue::Array(values) => {
				let var_label = get_var_label(name, Some("_label"));
				data_section.push_str(&format!("{} db \"[Array]\", 10, 0\n", var_label));
				text_section.push_str(&format!("    ; Variable {} = [array with {} elements]\n", name, values.len()));
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, var_label));
			},
			ConstValue::Null => {
				let var_label = get_var_label(name, None);
				data_section.push_str(&format!("{} db \"null\", 10, 0\n", var_label));
				text_section.push_str(&format!("    ; Variable {} = null\n", name));
				bss_section.push_str(&format!("var_mem_{}: resq 1  ; Memory for variable {}\n", name, name));
				text_section.push_str(&format!("    mov qword [var_mem_{}], 0\n", name));
			},
		}
	}
    
    fn generate_assign_code(name: &str, value: &Expr, text_section: &mut String, data_section: &mut String, constants: &HashMap<String, ConstValue>, variables: &mut HashMap<String, ConstValue>, string_counter: &mut usize) {
        if constants.contains_key(name) {
            panic!("Cannot reassign constant '{}'", name);
        }
        if !variables.contains_key(name) {
            panic!("Variable '{}' not defined before assignment", name);
        }
        let new_value = evaluate_constant_expr(value, constants, variables);
        variables.insert(name.to_string(), new_value.clone());
        match new_value {
            ConstValue::Number(n) => {
                text_section.push_str(&format!("    ; Assign {} = {}\n", name, n));
                text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, n));
            },
            ConstValue::Float(f) => {
                let float_label = get_var_label(name, Some("_float"));
                let new_label = format!("{}_updated_{}", float_label, *string_counter);
                *string_counter += 1;
                let float_str = f.to_string();
                data_section.push_str(&format!("{} db \"{}\", 10, 0\n", new_label, float_str));
                text_section.push_str(&format!("    ; Assign {} = {}\n", name, f));
                let int_val = (f * 100.0) as i64;
                text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, int_val));
                text_section.push_str(&format!("    mov qword [var_mem_{}_float], {}\n", name, new_label));
            },
            ConstValue::String(s) => {
                let var_label = get_var_label(name, None);
                let new_label = format!("{}_updated_{}", var_label, *string_counter);
                *string_counter += 1;
                data_section.push_str(&format!("{} db \"{}\", 10, 0\n", new_label, s));
                text_section.push_str(&format!("    ; Assign {} = \"{}\"\n", name, s));
                text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, new_label));
            },
            ConstValue::Boolean(b) => {
                text_section.push_str(&format!("    ; Assign {} = {}\n", name, b));
                text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, if b { 1 } else { 0 }));
            },
            ConstValue::Array(values) => {
                let var_label = get_var_label(name, Some("_label"));
                let new_label = format!("{}_updated_{}", var_label, *string_counter);
                *string_counter += 1;
                data_section.push_str(&format!("{} db \"[Array]\", 10, 0\n", new_label));
                text_section.push_str(&format!("    ; Assign {} = [array with {} elements]\n", name, values.len()));
                text_section.push_str(&format!("    mov qword [var_mem_{}], {}\n", name, new_label));
            },
            ConstValue::Null => {
                text_section.push_str(&format!("    ; Assign {} = null\n", name));
                text_section.push_str(&format!("    mov qword [var_mem_{}], 0\n", name));
            },
        }
    }
    
    // Process all expressions using the reused functions
	let boxed_exprs: Vec<Box<Expr>> = exprs.iter().cloned().map(Box::new).collect();
	process_statements(&boxed_exprs, &mut text_section, &mut data_section, &mut bss_section, 
					&mut constants, &mut variables, &string_labels, &mut string_counter);
    
    format!("{}\n{}\n{}", data_section, bss_section, text_section)
}
// Recursively collect all string literals in expressions
fn collect_string_literals(expr: &Expr, counter: &mut usize, string_labels: &mut HashMap<String, String>, data_section: &mut String) {
    match expr {
        Expr::StringLiteral(s) => {
            if !string_labels.contains_key(s) {
                let label = format!("str_{}", counter);
                *counter += 1;
                string_labels.insert(s.clone(), label.clone());
                data_section.push_str(&format!("{} db \"{}\", 10, 0\n", label, s));
            }
        },
        Expr::Print(inner) => collect_string_literals(inner, counter, string_labels, data_section),
        Expr::Exit(inner) => collect_string_literals(inner, counter, string_labels, data_section),
        Expr::Const { value, .. } => collect_string_literals(value, counter, string_labels, data_section),
        Expr::Let { value, .. } => collect_string_literals(value, counter, string_labels, data_section),
        Expr::Assign { value, .. } => collect_string_literals(value, counter, string_labels, data_section),
        Expr::BinaryOp { left, right, .. } => {
            collect_string_literals(left, counter, string_labels, data_section);
            collect_string_literals(right, counter, string_labels, data_section);
        },
        Expr::Array(elements) => {
            for elem in elements {
                collect_string_literals(elem, counter, string_labels, data_section);
            }
        },
		Expr::If { condition, then_branch, else_branch } => {
            collect_string_literals(condition, counter, string_labels, data_section);
            
            for stmt in then_branch {
                collect_string_literals(stmt, counter, string_labels, data_section);
            }
            
            if let Some(else_b) = else_branch {
                for stmt in else_b {
                    collect_string_literals(stmt, counter, string_labels, data_section);
                }
            }
        },

        _ => {},
    }
}

// Print an array element
fn print_array_element(elem: &Expr, text_section: &mut String, constants: &HashMap<String, ConstValue>,
                      variables: &HashMap<String, ConstValue>, string_labels: &HashMap<String, String>,
                      data_section: &mut String, counter: &mut usize) {
    match elem {
        Expr::StringLiteral(s) => {
            // Find the label for this string
            let label = string_labels.get(s).expect("String not found in data section");
            
            text_section.push_str(&format!("    ; Print array element (string): {}\n", s));
            text_section.push_str("    mov rax, 1          ; sys_write\n");
            text_section.push_str("    mov rdi, 1          ; stdout\n");
            text_section.push_str(&format!("    mov rsi, {}\n", label));
            text_section.push_str(&format!("    mov rdx, {}\n", s.len()));
            text_section.push_str("    syscall\n\n");
        },
        Expr::Number(n) => {
            // Convert number to string and add to data section
            let num_label = format!("num_{}", counter);
            *counter += 1;
            let num_str = n.to_string();
            data_section.push_str(&format!("{} db \"{}\", 0\n", num_label, num_str));
            
            text_section.push_str(&format!("    ; Print array element (number): {}\n", n));
            text_section.push_str("    mov rax, 1          ; sys_write\n");
            text_section.push_str("    mov rdi, 1          ; stdout\n");
            text_section.push_str(&format!("    mov rsi, {}\n", num_label));
            text_section.push_str(&format!("    mov rdx, {}\n", num_str.len()));
            text_section.push_str("    syscall\n\n");
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    ; Print array element (boolean): {}\n", b));
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
        Expr::Null => {
            text_section.push_str("    ; Print array element (null)\n");
            text_section.push_str("    mov rax, 1          ; sys_write\n");
            text_section.push_str("    mov rdi, 1          ; stdout\n");
            text_section.push_str("    mov rsi, null_str\n");
            text_section.push_str("    mov rdx, 4          ; 'null'\n");
            text_section.push_str("    syscall\n\n");
        },
        Expr::Variable(name) => {
            text_section.push_str(&format!("    ; Print array element (variable): {}\n", name));
            
            // Check if it's a constant or variable
            if let Some(value) = constants.get(name) {
                match value {
                    ConstValue::Number(n) => {
                        let num_str = n.to_string();
                        let num_label = format!("array_num_{}", counter);
                        *counter += 1;
                        data_section.push_str(&format!("{} db \"{}\", 0\n", num_label, num_str));
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", num_label));
                        text_section.push_str(&format!("    mov rdx, {}\n", num_str.len()));
                        text_section.push_str("    syscall\n\n");
                    },
                    ConstValue::String(s) => {
                        let label = get_var_label(name, None);
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str(&format!("    mov rdx, {}\n", s.len()));
                        text_section.push_str("    syscall\n\n");
                    },
                    ConstValue::Boolean(b) => {
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
                    _ => {
                        // For other types, use a generic approach
                        let label = get_var_label(name, None);
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str("    mov rdx, 10         ; Assume max 10 chars\n");
                        text_section.push_str("    syscall\n\n");
                    }
                }
            } else if let Some(value) = variables.get(name) {
                match value {
                    ConstValue::Number(n) => {
                        let num_str = n.to_string();
                        let num_label = format!("array_num_{}", counter);
                        *counter += 1;
                        data_section.push_str(&format!("{} db \"{}\", 0\n", num_label, num_str));
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", num_label));
                        text_section.push_str(&format!("    mov rdx, {}\n", num_str.len()));
                        text_section.push_str("    syscall\n\n");
                    },
                    ConstValue::String(s) => {
                        let label = get_var_label(name, None);
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str(&format!("    mov rdx, {}\n", s.len()));
                        text_section.push_str("    syscall\n\n");
                    },
                    ConstValue::Boolean(b) => {
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
                    _ => {
                        // For other types, use a generic approach
                        let label = get_var_label(name, None);
                        
                        text_section.push_str("    mov rax, 1          ; sys_write\n");
                        text_section.push_str("    mov rdi, 1          ; stdout\n");
                        text_section.push_str(&format!("    mov rsi, {}\n", label));
                        text_section.push_str("    mov rdx, 20         ; Assume max 20 chars\n");
                        text_section.push_str("    syscall\n\n");
                    }
                }
            } else {
                panic!("Undefined variable in array: {}", name);
            }
        },
        _ => {
            panic!("Unsupported array element type");
        }
    }
}

// Function to check if an expression will evaluate to a string
fn is_string_expr(expr: &Expr, constants: &HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>) -> bool {
    match expr {
        Expr::StringLiteral(_) => true,
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                matches!(value, ConstValue::String(_))
            } else if let Some(value) = variables.get(name) {
                matches!(value, ConstValue::String(_))
            } else {
                false
            }
        },
        Expr::BinaryOp { op, left, right } => {
            if *op == BinOp::Add {
                is_string_expr(left, constants, variables) || is_string_expr(right, constants, variables)
            } else {
                false
            }
        },
        _ => false,
    }
}

// Generate code for string concatenation
fn generate_string_concat(expr: &Expr, text_section: &mut String, 
                        constants: &HashMap<String, ConstValue>,
                        variables: &HashMap<String, ConstValue>,
                        string_labels: &HashMap<String, String>,
                        data_section: &mut String,
                        counter: &mut usize) {
    match expr {
        Expr::StringLiteral(s) => {
            // Find the label for this string
            let label = string_labels.get(s).expect("String literal not found in labels");
            
            text_section.push_str(&format!("    ; Append string: {}\n", s));
            text_section.push_str(&format!("    mov rsi, {}\n", label));
            text_section.push_str("    call append_string_without_newline\n");
        },
        Expr::Number(n) => {
            text_section.push_str(&format!("    ; Append number: {}\n", n));
            text_section.push_str(&format!("    mov rax, {}\n", n));
            text_section.push_str("    call append_number\n");
        },
        Expr::Float(f) => {
            // For float concatenation, convert to string and store in data section
            let float_label = format!("float_concat_{}", counter);
            *counter += 1;
            let float_str = f.to_string();
            data_section.push_str(&format!("{} db \"{}\", 0\n", float_label, float_str));
            
            text_section.push_str(&format!("    ; Append float: {}\n", f));
            text_section.push_str(&format!("    mov rsi, {}\n", float_label));
            text_section.push_str("    call append_string_without_newline\n");
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    ; Append boolean: {}\n", b));
            if *b {
                text_section.push_str("    mov rsi, true_str\n");
            } else {
                text_section.push_str("    mov rsi, false_str\n");
            }
            text_section.push_str("    call append_string_without_newline\n");
        },
        Expr::Null => {
            text_section.push_str("    ; Append null\n");
            text_section.push_str("    mov rsi, null_str\n");
            text_section.push_str("    call append_string_without_newline\n");
        },
        Expr::Variable(name) => {
            let in_constants = constants.contains_key(name);
            let value = if in_constants {
                constants.get(name).unwrap()
            } else {
                variables.get(name).unwrap()
            };
            match value {
                ConstValue::Number(_) => {
                    text_section.push_str(&format!("    ; Append numeric variable: {}\n", name));
                    text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                    text_section.push_str("    call append_number\n");
                },
                ConstValue::Float(_) => {
                    text_section.push_str(&format!("    ; Append float variable: {}\n", name));
                    let float_label = get_var_label(name, Some("_float"));
                    if in_constants {
                        text_section.push_str(&format!("    mov rsi, {}\n", float_label));
                    } else {
                        text_section.push_str(&format!("    mov rsi, [var_mem_{}_float]\n", name));
                    }
                    text_section.push_str("    call append_string_without_newline\n");
                },
                ConstValue::String(_) => {
                    text_section.push_str(&format!("    ; Append string variable: {}\n", name));
                    let var_label = get_var_label(name, None);
                    if in_constants {
                        text_section.push_str(&format!("    mov rsi, {}\n", var_label));
                    } else {
                        text_section.push_str(&format!("    mov rsi, [var_mem_{}]\n", name));
                    }
                    text_section.push_str("    call append_string_without_newline\n");
                },
                ConstValue::Boolean(_) => {
                    text_section.push_str(&format!("    ; Append boolean variable: {}\n", name));
                    if in_constants {
                        if let ConstValue::Boolean(b) = value {
                            if *b {
                                text_section.push_str("    mov rsi, true_str\n");
                            } else {
                                text_section.push_str("    mov rsi, false_str\n");
                            }
                        }
                    } else {
                        text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                        text_section.push_str("    cmp rax, 0\n");
                        text_section.push_str(&format!("    je .false_{}\n", name));
                        text_section.push_str("    mov rsi, true_str\n");
                        text_section.push_str(&format!("    jmp .done_{}\n", name));
                        text_section.push_str(&format!(".false_{}:\n", name));
                        text_section.push_str("    mov rsi, false_str\n");
                        text_section.push_str(&format!(".done_{}:\n", name));
                    }
                    text_section.push_str("    call append_string_without_newline\n");
                },
                ConstValue::Array(_) => {
                    text_section.push_str(&format!("    ; Append array constant: {}\n", name));
                    let array_label = get_var_label(name, Some("_label"));
                    text_section.push_str(&format!("    mov rsi, {}\n", array_label));
                    text_section.push_str("    call append_string_without_newline\n");
                },
                ConstValue::Null => {
                    text_section.push_str(&format!("    ; Append null constant: {}\n", name));
                    text_section.push_str("    mov rsi, null_str\n");
                    text_section.push_str("    call append_string_without_newline\n");
                }
            }
        },
        Expr::BinaryOp { op, left, right } => {
            if *op == BinOp::Add && (is_string_expr(left, constants, variables) || is_string_expr(right, constants, variables)) {
                // If this is a string concatenation, process each part separately
                generate_string_concat(left, text_section, constants, variables, string_labels, data_section, counter);
                generate_string_concat(right, text_section, constants, variables, string_labels, data_section, counter);
            } else {
                text_section.push_str("    ; Append result of numeric expression\n");
               
                text_section.push_str("    push rbx\n");
                text_section.push_str("    push rdx\n");
                
                generate_expression_code(expr, text_section, constants, variables);
                
                text_section.push_str("    ; Call append_number with result in RAX\n");
                
                text_section.push_str("    pop rdx\n");
                text_section.push_str("    pop rbx\n");
                
                text_section.push_str("    call append_number\n");
            }
        },
        _ => panic!("Unsupported expression in string concatenation"),
    }
}

// Generate code for expressions
fn generate_expression_code(expr: &Expr, text_section: &mut String, constants: &HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>) {
    match expr {
        Expr::Number(n) => {
            text_section.push_str(&format!("    ; Load number: {}\n", n));
            text_section.push_str(&format!("    mov rax, {}\n", n));
        },
        Expr::Float(f) => {
            // Floats would normally require FPU or SSE but for simplicity use integers
            let int_val = (*f * 100.0) as i64; // Scale up by 100 to preserve some decimal places
            text_section.push_str(&format!("    ; Load float: {} (scaled as integer)\n", f));
            text_section.push_str(&format!("    mov rax, {}\n", int_val));
        },
        Expr::Boolean(b) => {
            text_section.push_str(&format!("    ; Load boolean: {}\n", b));
            if *b {
                text_section.push_str("    mov rax, 1\n");
            } else {
                text_section.push_str("    mov rax, 0\n");
            }
        },
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                match value {
                    ConstValue::Number(n) => {
                        text_section.push_str(&format!("    ; Load numeric constant: {}\n", name));
                        text_section.push_str(&format!("    mov rax, {}\n", n));
                    },
                    ConstValue::Float(f) => {
                        // Similar to above use integers for simplicity
                        let int_val = (*f * 100.0) as i64;
                        text_section.push_str(&format!("    ; Load float constant: {} (scaled as integer)\n", name));
                        text_section.push_str(&format!("    mov rax, {}\n", int_val));
                    },
                    ConstValue::Boolean(b) => {
                        text_section.push_str(&format!("    ; Load boolean constant: {}\n", name));
                        if *b {
                            text_section.push_str("    mov rax, 1\n");
                        } else {
                            text_section.push_str("    mov rax, 0\n");
                        }
                    },
                    _ => panic!("Cannot use non-numeric constant in expression: {}", name),
                }
            } else if let Some(value) = variables.get(name) {
                match value {
                    ConstValue::Number(_) => {
                        text_section.push_str(&format!("    ; Load numeric variable: {}\n", name));
                        text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                    },
                    ConstValue::Float(_) => {
                        text_section.push_str(&format!("    ; Load float variable: {} (scaled as integer)\n", name));
                        text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                    },
                    ConstValue::Boolean(_) => {
                        text_section.push_str(&format!("    ; Load boolean variable: {}\n", name));
                        text_section.push_str(&format!("    mov rax, [var_mem_{}]\n", name));
                    },
                    _ => panic!("Cannot use non-numeric variable in expression: {}", name),
                }
            } else {
                panic!("Undefined variable: {}", name);
            }
        },
        Expr::BinaryOp { op, left, right } => {
            // Check if this is a string operation
            if *op == BinOp::Add && (is_string_expr(left, constants, variables) || is_string_expr(right, constants, variables)) {
                panic!("String operations should be handled by generate_string_concat");
            }
            
            // First, evaluate the right expression and push result to stack
            generate_expression_code(right, text_section, constants, variables);
            text_section.push_str("    push rax\n");
            
            // Then, evaluate the left expression (result in RAX)
            generate_expression_code(left, text_section, constants, variables);
            
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
        _ => panic!("Unsupported expression type"),
    }
}

// Evaluate constant expressions at compile time
fn evaluate_constant_expr(expr: &Expr, constants: &HashMap<String, ConstValue>, variables: &HashMap<String, ConstValue>) -> ConstValue {
    match expr {
        Expr::Number(n) => ConstValue::Number(*n),
        Expr::Float(f) => ConstValue::Float(*f),
        Expr::StringLiteral(s) => ConstValue::String(s.clone()),
        Expr::Boolean(b) => ConstValue::Boolean(*b),
        Expr::Null => ConstValue::Null,
        Expr::Array(elements) => {
            let evaluated_elements = elements.iter()
                .map(|e| evaluate_constant_expr(e, constants, variables))
                .collect::<Vec<_>>();
            ConstValue::Array(evaluated_elements)
        },
        Expr::Variable(name) => {
            if let Some(value) = constants.get(name) {
                value.clone()
            } else if let Some(value) = variables.get(name) {
                value.clone()
            } else {
                panic!("Undefined variable in constant expression: {}", name);
            }
        },
        Expr::BinaryOp { op, left, right } => {
            let left_val = evaluate_constant_expr(left, constants, variables);
            let right_val = evaluate_constant_expr(right, constants, variables);
            
            match (op, &left_val, &right_val) {
                // Integer arithmetic
                (BinOp::Add, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Number(a + b),
                (BinOp::Sub, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Number(a - b),
                (BinOp::Mul, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Number(a * b),
                (BinOp::Div, ConstValue::Number(a), ConstValue::Number(b)) => {
                    if *b == 0 {
                        panic!("Division by zero in constant expression");
                    }
                    ConstValue::Number(a / b)
                },
                
                // Float arithmetic
                (BinOp::Add, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Float(a + b),
                (BinOp::Sub, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Float(a - b),
                (BinOp::Mul, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Float(a * b),
                (BinOp::Div, ConstValue::Float(a), ConstValue::Float(b)) => {
                    if *b == 0.0 {
                        panic!("Division by zero in constant expression");
                    }
                    ConstValue::Float(a / b)
                },
                
                // Mixed float-integer arithmetic
                (BinOp::Add, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Float(*a as f64 + b),
                (BinOp::Add, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Float(a + *b as f64),
                (BinOp::Sub, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Float(*a as f64 - b),
                (BinOp::Sub, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Float(a - *b as f64),
                (BinOp::Mul, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Float(*a as f64 * b),
                (BinOp::Mul, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Float(a * *b as f64),
                (BinOp::Div, ConstValue::Number(a), ConstValue::Float(b)) => {
                    if *b == 0.0 {
                        panic!("Division by zero in constant expression");
                    }
                    ConstValue::Float(*a as f64 / b)
                },
                (BinOp::Div, ConstValue::Float(a), ConstValue::Number(b)) => {
                    if *b == 0 {
                        panic!("Division by zero in constant expression");
                    }
                    ConstValue::Float(a / *b as f64)
                },
                
                // String concatenation
                (BinOp::Add, ConstValue::String(a), ConstValue::String(b)) => {
                    ConstValue::String(format!("{}{}", a, b))
                },
                (BinOp::Add, ConstValue::String(a), _) => {
                    let b_str = match &right_val {
                        ConstValue::Number(n) => n.to_string(),
                        ConstValue::Float(f) => f.to_string(),
                        ConstValue::Boolean(b) => b.to_string(),
                        ConstValue::Null => "null".to_string(),
                        _ => panic!("Cannot convert to string"),
                    };
                    ConstValue::String(format!("{}{}", a, b_str))
                },
                (BinOp::Add, _, ConstValue::String(b)) => {
                    let a_str = match &left_val {
                        ConstValue::Number(n) => n.to_string(),
                        ConstValue::Float(f) => f.to_string(),
                        ConstValue::Boolean(b) => b.to_string(),
                        ConstValue::Null => "null".to_string(),
                        _ => panic!("Cannot convert to string"),
                    };
                    ConstValue::String(format!("{}{}", a_str, b))
                },
                
                // Comparison operators for integers
                (BinOp::Equal, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a == b),
                (BinOp::NotEqual, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a != b),
                (BinOp::Lt, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a < b),
                (BinOp::Gt, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a > b),
                (BinOp::Lte, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a <= b),
                (BinOp::Gte, ConstValue::Number(a), ConstValue::Number(b)) => ConstValue::Boolean(a >= b),
                
                // Comparison operators for floats
                (BinOp::Equal, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a == b),
                (BinOp::NotEqual, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a != b),
                (BinOp::Lt, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a < b),
                (BinOp::Gt, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a > b),
                (BinOp::Lte, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a <= b),
                (BinOp::Gte, ConstValue::Float(a), ConstValue::Float(b)) => ConstValue::Boolean(a >= b),
                
                // Mixed type comparisons (float and int)
                (BinOp::Equal, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) == *b),
                (BinOp::Equal, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a == (*b as f64)),
                
                (BinOp::NotEqual, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) != *b),
                (BinOp::NotEqual, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a != (*b as f64)),
                
                (BinOp::Lt, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) < *b),
                (BinOp::Lt, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a < (*b as f64)),
                
                (BinOp::Gt, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) > *b),
                (BinOp::Gt, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a > (*b as f64)),
                
                (BinOp::Lte, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) <= *b),
                (BinOp::Lte, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a <= (*b as f64)),
                
                (BinOp::Gte, ConstValue::Number(a), ConstValue::Float(b)) => ConstValue::Boolean((*a as f64) >= *b),
                (BinOp::Gte, ConstValue::Float(a), ConstValue::Number(b)) => ConstValue::Boolean(*a >= (*b as f64)),
                
                // String comparisons
                (BinOp::Equal, ConstValue::String(a), ConstValue::String(b)) => ConstValue::Boolean(a == b),
                (BinOp::NotEqual, ConstValue::String(a), ConstValue::String(b)) => ConstValue::Boolean(a != b),
                
                // Boolean comparisons
                (BinOp::Equal, ConstValue::Boolean(a), ConstValue::Boolean(b)) => ConstValue::Boolean(a == b),
                (BinOp::NotEqual, ConstValue::Boolean(a), ConstValue::Boolean(b)) => ConstValue::Boolean(a != b),
                
                _ => panic!("Invalid operation on types in constant expression"),
            }
        },
        _ => panic!("Unsupported expression in constant evaluation"),
    }
}