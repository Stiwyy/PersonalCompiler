# SPP Compiler

This repository contains a compiler written in Rust for a simple programming language (SPP).  

---

## Prerequisites

To use this compiler, you need:

- **Rust compiler (`rustc`) and Cargo**  
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- **NASM (Netwide Assembler)**  
- **ld (GNU Linker)**  

---

## Running the Compiler

### Build the Compiler
```bash
cargo build
```
### Compile Example Code
```bash
cargo run examples/sample.spp
```
### Convert Assembly Output to Executable (Optional)
```bash
nasm -f elf64 build/out.asm -o build/out.o && \
ld -m elf_x86_64 -o build/out build/out.o && \
./build/out
```
---

## SPP Language Features

Currently, the SPP language supports:

- Integers  
- String literals  
- Arithmetic operations (`+`, `-`, `*`, `/`)  
- Console output: `console.print("Text");`  
- Program termination: `exit(code);`  

---

## Example

    console.print("Hello World!");
    exit(10);

---

## Project Structure

- `src/lexer.rs`: Tokenizes the source code  
- `src/parser.rs`: Parses tokens into an AST  
- `src/ast.rs`: Definitions for the abstract syntax tree  
- `src/codegen.rs`: Generates NASM assembly code  
- `src/main.rs`: Main program that connects all components  
