section .data
str_0: db 'Hello World!',0

section .text
global _start

_start:
push rbp
mov rbp, rsp
mov rax, 1
mov rdi, 1
mov rsi, str_0
mov rdx, 12
syscall
mov rax, 10
mov rdi, rax
mov rax, 60
syscall

mov rsp, rbp
pop rbp
mov rax, 60
mov rdi, 0
syscall
