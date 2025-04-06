use crate::ast::Expr;
use std::path::Path;
use std::fs;

pub fn generate_nasm(exprs: &[Expr], output_path: &Path) -> String {
    let mut asm = String::new();
    let mut data_section = String::new();
    let mut text_section = String::new();
    let mut string_counter = 0;

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
                if let Expr::StringLiteral(s) = &**inner {
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
                }
            }
            Expr::Exit(code) => {
                if let Expr::Number(n) = &**code {
                    text_section.push_str("    ; Exit program\n");
                    text_section.push_str("    mov rax, 60         ; sys_exit\n");
                    text_section.push_str(&format!("    mov rdi, {}\n", n));
                    text_section.push_str("    syscall\n\n");
                }
            }
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

    asm.push_str(&text_section);

    // Write to file
    if let Err(e) = fs::write(output_path, &asm) {
        eprintln!("Error writing to file: {}", e);
    }

    asm
}