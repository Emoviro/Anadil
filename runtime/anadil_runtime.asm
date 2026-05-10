; Anadil native runtime for Windows x64.
; This file is assembled separately and linked with generated user programs.

option casemap:none

STD_INPUT_HANDLE equ -10
STD_OUTPUT_HANDLE equ -11

extrn GetStdHandle:proc
extrn WriteFile:proc
extrn ReadFile:proc
extrn ExitProcess:proc

.data
newline db 10
text_dogru db 100, 111, 196, 159, 114, 117, 0
text_yanlis db 121, 97, 110, 108, 196, 177, 197, 159, 0
io_bytes_written dd 0
io_bytes_read dd 0
io_wait_buffer db 0
number_buffer db 32 dup(0)

.code
anadil_runtime_write_bytes PROC
    push rbp
    mov rbp, rsp
    sub rsp, 64

    mov qword ptr [rbp - 8], rcx
    mov qword ptr [rbp - 16], rdx

    mov ecx, STD_OUTPUT_HANDLE
    call GetStdHandle

    mov rcx, rax
    mov rdx, qword ptr [rbp - 8]
    mov r8d, dword ptr [rbp - 16]
    lea r9, io_bytes_written
    mov qword ptr [rsp + 32], 0
    call WriteFile

    add rsp, 64
    pop rbp
    ret
anadil_runtime_write_bytes ENDP

anadil_runtime_print_newline PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    lea rcx, newline
    mov edx, 1
    call anadil_runtime_write_bytes

    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_newline ENDP

anadil_runtime_print_sayi PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov rax, rcx
    xor r10d, r10d
    test rax, rax
    jge anadil_runtime_print_sayi_abs_ready
    mov r10d, 1
    neg rax

anadil_runtime_print_sayi_abs_ready:
    lea r8, number_buffer + 31
    xor r9d, r9d
    mov r11, 10
    test rax, rax
    jne anadil_runtime_print_sayi_loop

    dec r8
    mov byte ptr [r8], 48
    inc r9
    jmp anadil_runtime_print_sayi_sign

anadil_runtime_print_sayi_loop:
    xor edx, edx
    div r11
    add dl, 48
    dec r8
    mov byte ptr [r8], dl
    inc r9
    test rax, rax
    jne anadil_runtime_print_sayi_loop

anadil_runtime_print_sayi_sign:
    test r10d, r10d
    je anadil_runtime_print_sayi_write
    dec r8
    mov byte ptr [r8], 45
    inc r9

anadil_runtime_print_sayi_write:
    mov rcx, r8
    mov rdx, r9
    call anadil_runtime_write_bytes
    call anadil_runtime_print_newline

    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_sayi ENDP

anadil_runtime_print_metin PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov r8, rcx
    xor edx, edx
anadil_runtime_print_metin_len:
    cmp byte ptr [r8 + rdx], 0
    je anadil_runtime_print_metin_write
    inc rdx
    jmp anadil_runtime_print_metin_len

anadil_runtime_print_metin_write:
    call anadil_runtime_write_bytes
    call anadil_runtime_print_newline

    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_metin ENDP

anadil_runtime_print_mantik PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    cmp rcx, 0
    je anadil_runtime_print_mantik_false
    lea rcx, text_dogru
    jmp anadil_runtime_print_mantik_write
anadil_runtime_print_mantik_false:
    lea rcx, text_yanlis
anadil_runtime_print_mantik_write:
    call anadil_runtime_print_metin

    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_mantik ENDP

anadil_runtime_strcmp PROC
anadil_runtime_strcmp_loop:
    mov al, byte ptr [rcx]
    mov r8b, byte ptr [rdx]
    cmp al, r8b
    jne anadil_runtime_strcmp_diff
    test al, al
    je anadil_runtime_strcmp_equal
    inc rcx
    inc rdx
    jmp anadil_runtime_strcmp_loop
anadil_runtime_strcmp_diff:
    movzx eax, al
    movzx r8d, r8b
    sub eax, r8d
    ret
anadil_runtime_strcmp_equal:
    xor eax, eax
    ret
anadil_runtime_strcmp ENDP

anadil_runtime_wait_before_exit PROC
    push rbp
    mov rbp, rsp
    sub rsp, 48

    mov ecx, STD_INPUT_HANDLE
    call GetStdHandle

    mov rcx, rax
    lea rdx, io_wait_buffer
    mov r8d, 1
    lea r9, io_bytes_read
    mov qword ptr [rsp + 32], 0
    call ReadFile

    add rsp, 48
    pop rbp
    ret
anadil_runtime_wait_before_exit ENDP

anadil_runtime_panic PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    call anadil_runtime_print_metin
    call anadil_runtime_wait_before_exit
    mov ecx, 1
    call ExitProcess
anadil_runtime_panic ENDP

END
