mod parser;
mod codegen;

use std::process::Command;
use std::path::Path;

fn main() {
    let path = "examples/sample.spp";
    let source = std::fs::read_to_string(path)
        .expect("Could not read input file");

    let Some(exit_code) = parser::parse_exit_code(&source) else {
        eprintln!("Invalid syntax.");
        std::process::exit(1);
    };

    let asm_path = Path::new("build/out.asm");
    let bin_path = Path::new("build/out");

    std::fs::create_dir_all("build").unwrap();

    codegen::generate_nasm(exit_code, asm_path);

    let nasm_status = Command::new("nasm")
        .args(["-felf64", asm_path.to_str().unwrap(), "-o", "build/out.o"])
        .status()
        .expect("Failed to run NASM");

    if !nasm_status.success() {
        eprintln!("NASM failed");
        std::process::exit(1);
    }

    let ld_status = Command::new("ld")
        .args(["-o", bin_path.to_str().unwrap(), "build/out.o"])
        .status()
        .expect("Failed to run ld");

    if !ld_status.success() {
        eprintln!("Linking failed");
        std::process::exit(1);
    }

    println!("✅ Compilation successful! Output: build/out");
}
