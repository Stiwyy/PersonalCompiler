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

## Features

Currently, the SPP language supports:

### Comments
- Single line comments (//)
- Mutli line comments (/* */)

### Data Types
- Integers
- Floating-point numbers
- String literals (With both single- and double quotes)
- Booleans (true/false)
- Arrays (mixed types supported)
- Null values

### Constants
- Constant declarations with compile-time evaluation
- Constants of any supported data type
- Complex expressions in constant initialization
- Constants can reference other constants

### Operations
- Arithmetic operations (+, -, *, /)
- Comparison operations (==, !=, <, >, <=, >=)
- String concatenation
- Mixed-type operations (e.g., adding strings and numbers)

### Control Flow
- Program termination: `exit(code);`

### Input/Output
- Console output: `console.print("Text");`
- String interpolation: `console.print("Value: " + variable);`

### Assembly Generation
- Generates x86_64 NASM assembly
- Optimized constant handling
- Proper memory management

### Example
```spp
const username = "Stiwyy";
const currentDate = "2025-08-14";
const currentTime = "23:53:15";

console.print("SPP Demo");
console.print("User: " + username);
console.print("Date: " + currentDate + " " + currentTime);

const pi = 3.14159;
const radius = 5;
const area = pi * radius * radius;
console.print("Circle area: " + area);

exit(0);
```
---

## Project Structure

- `src/lexer.rs`: Tokenizes the source code  
- `src/parser.rs`: Parses tokens into an AST  
- `src/ast.rs`: Definitions for the abstract syntax tree  
- `src/codegen.rs`: Generates NASM assembly code  
- `src/main.rs`: Main program that connects all components  
