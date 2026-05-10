; Anadil native runtime for Windows x64.
; This file is assembled separately and linked with generated user programs.

option casemap:none

extrn printf:proc
extrn strcmp:proc
extrn getchar:proc
extrn exit:proc

.data
fmt_sayi db "%lld", 10, 0
fmt_metin db "%s", 10, 0
fmt_runtime_error db "%s", 10, 0
text_dogru db 100, 111, 196, 159, 114, 117, 0
text_yanlis db 121, 97, 110, 108, 196, 177, 197, 159, 0

.code
anadil_runtime_print_sayi PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov rdx, rcx
    lea rcx, fmt_sayi
    call printf
    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_sayi ENDP

anadil_runtime_print_metin PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov rdx, rcx
    lea rcx, fmt_metin
    call printf
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
    lea rdx, text_dogru
    jmp anadil_runtime_print_mantik_done
anadil_runtime_print_mantik_false:
    lea rdx, text_yanlis
anadil_runtime_print_mantik_done:
    lea rcx, fmt_metin
    call printf
    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_mantik ENDP

anadil_runtime_strcmp PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32
    call strcmp
    add rsp, 32
    pop rbp
    ret
anadil_runtime_strcmp ENDP

anadil_runtime_wait_before_exit PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32
    call getchar
    add rsp, 32
    pop rbp
    ret
anadil_runtime_wait_before_exit ENDP

anadil_runtime_panic PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov rdx, rcx
    lea rcx, fmt_runtime_error
    call printf
    call getchar
    mov ecx, 1
    call exit
anadil_runtime_panic ENDP

END
