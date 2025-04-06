use std::fs;
use std::path::Path;

//generates the assembly code for the out.asm
//Works only temporarily like this will change in the future
pub fn generate_nasm(exit_code: i32, output_path: &Path) {
    let asm_code = format!(
        "section .text
    global _start

_start:
    mov rax, 60
    mov rdi, {}
    syscall
", exit_code);

    fs::write(output_path, asm_code).expect("Failed to write ASM file");
}
