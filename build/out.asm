section .data
msg0 db "Hello World!", 10, 0
msg1 db "test2", 10, 0

section .text
global _start

_start:
    ; Print: Hello World!
    mov rax, 1          ; sys_write
    mov rdi, 1          ; stdout
    mov rsi, msg0
    mov rdx, 13
    syscall

    ; Exit program
    mov rax, 60         ; sys_exit
    mov rdi, 10
    syscall

    ; Print: test2
    mov rax, 1          ; sys_write
    mov rdi, 1          ; stdout
    mov rsi, msg1
    mov rdx, 6
    syscall

