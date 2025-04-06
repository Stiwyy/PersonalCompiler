use std::fs::{self, File};
use std::path::Path;
use std::io::Write;

pub fn generate_nasm(exit_code: i32, output_path: &Path) {
    // Ensure the directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let asm_code = format!(
        "section .text
    global _start

_start:
    mov rax, 60            ; syscall number for exit
    mov rdi, {}            ; exit code
    syscall
", exit_code);

    let mut file = File::create(output_path).expect("Failed to create ASM file");
    file.write_all(asm_code.as_bytes())
        .expect("Failed to write to ASM file");

    println!("ASM code successfully written to {}", output_path.display());
}
