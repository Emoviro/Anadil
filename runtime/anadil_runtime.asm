; Anadil native runtime for Windows x64.
; This file is assembled separately and linked with generated user programs.

option casemap:none

STD_INPUT_HANDLE equ -10
STD_OUTPUT_HANDLE equ -11
ANADIL_STATIC_REFCOUNT_MIN equ 4000000000000000h
ANADIL_TIP_METIN equ 1

extrn GetStdHandle:proc
extrn WriteFile:proc
extrn ReadFile:proc
extrn ExitProcess:proc
extrn GetProcessHeap:proc
extrn HeapAlloc:proc
extrn HeapFree:proc

.data
newline db 10
runtime_error_prefix db "Anadil runtime hatasi: ", 0
heap_alloc_error db "Bellek tahsisi basarisiz", 0
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

anadil_runtime_write_cstr PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov r8, rcx
    xor edx, edx
anadil_runtime_write_cstr_len:
    cmp byte ptr [r8 + rdx], 0
    je anadil_runtime_write_cstr_write
    inc rdx
    jmp anadil_runtime_write_cstr_len

anadil_runtime_write_cstr_write:
    mov rcx, r8
    call anadil_runtime_write_bytes

    add rsp, 32
    pop rbp
    ret
anadil_runtime_write_cstr ENDP

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

    call anadil_runtime_write_cstr
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

; Length-prefixed metin ABI hazirligi.
; Layout: [refcount][tip_id=ANADIL_TIP_METIN][len: u64][bytes...]
; Bu helper'lar henuz compiler tarafindan emit edilmez.
anadil_runtime_metin_uzunluk PROC
    mov rax, qword ptr [rcx]
    ret
anadil_runtime_metin_uzunluk ENDP

anadil_runtime_print_metin_nesne PROC
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov rdx, qword ptr [rcx]
    lea rcx, [rcx + 8]
    call anadil_runtime_write_bytes
    call anadil_runtime_print_newline

    add rsp, 32
    pop rbp
    ret
anadil_runtime_print_metin_nesne ENDP

anadil_runtime_metin_esit PROC
    mov r8, qword ptr [rcx]
    cmp r8, qword ptr [rdx]
    jne anadil_runtime_metin_esit_false

    lea rcx, [rcx + 8]
    lea rdx, [rdx + 8]
    xor r9, r9
anadil_runtime_metin_esit_loop:
    cmp r9, r8
    je anadil_runtime_metin_esit_true
    mov al, byte ptr [rcx + r9]
    cmp al, byte ptr [rdx + r9]
    jne anadil_runtime_metin_esit_false
    inc r9
    jmp anadil_runtime_metin_esit_loop

anadil_runtime_metin_esit_true:
    mov eax, 1
    ret
anadil_runtime_metin_esit_false:
    xor eax, eax
    ret
anadil_runtime_metin_esit ENDP

anadil_runtime_metin_birlestir PROC
    push rbp
    mov rbp, rsp
    sub rsp, 80

    mov qword ptr [rbp - 8], rcx      ; left
    mov qword ptr [rbp - 16], rdx     ; right

    mov r8, qword ptr [rcx]
    mov r9, qword ptr [rdx]
    mov qword ptr [rbp - 24], r8      ; left_len
    mov qword ptr [rbp - 32], r9      ; right_len

    mov rcx, r8
    add rcx, r9
    mov qword ptr [rbp - 40], rcx     ; total_len
    add rcx, 8
    mov edx, ANADIL_TIP_METIN
    call anadil_runtime_tahsis

    mov qword ptr [rbp - 48], rax     ; result
    mov r10, qword ptr [rbp - 40]
    mov qword ptr [rax], r10

    mov r10, qword ptr [rbp - 8]
    add r10, 8
    mov r11, qword ptr [rbp - 48]
    add r11, 8
    mov r8, qword ptr [rbp - 24]
anadil_runtime_metin_birlestir_left_loop:
    test r8, r8
    je anadil_runtime_metin_birlestir_right_start
    mov al, byte ptr [r10]
    mov byte ptr [r11], al
    inc r10
    inc r11
    dec r8
    jmp anadil_runtime_metin_birlestir_left_loop

anadil_runtime_metin_birlestir_right_start:
    mov r10, qword ptr [rbp - 16]
    add r10, 8
    mov r8, qword ptr [rbp - 32]
anadil_runtime_metin_birlestir_right_loop:
    test r8, r8
    je anadil_runtime_metin_birlestir_done
    mov al, byte ptr [r10]
    mov byte ptr [r11], al
    inc r10
    inc r11
    dec r8
    jmp anadil_runtime_metin_birlestir_right_loop

anadil_runtime_metin_birlestir_done:
    mov rax, qword ptr [rbp - 48]
    add rsp, 80
    pop rbp
    ret
anadil_runtime_metin_birlestir ENDP

; V0.2 heap primitive ABI.
; Nesne layout'u: [refcount: u64][tip_id: u64][data...]
; Kullaniciya donen pointer data baslangicini gosterir.
anadil_runtime_tahsis PROC
    push rbp
    mov rbp, rsp
    sub rsp, 48

    mov qword ptr [rbp - 8], rcx      ; data_size
    mov qword ptr [rbp - 16], rdx     ; tip_id

    call GetProcessHeap
    mov rcx, rax
    xor edx, edx
    mov r8, qword ptr [rbp - 8]
    add r8, 16
    call HeapAlloc

    test rax, rax
    jne anadil_runtime_tahsis_ok
    lea rcx, heap_alloc_error
    call anadil_runtime_panic

anadil_runtime_tahsis_ok:
    mov qword ptr [rax], 1
    mov rdx, qword ptr [rbp - 16]
    mov qword ptr [rax + 8], rdx
    add rax, 16

    add rsp, 48
    pop rbp
    ret
anadil_runtime_tahsis ENDP

; NOT: RC Faz 1 tek-thread varsayimiyla non-atomic sayac kullaniyor.
; Threading dile girdiginde bu PROC lock'lu versiyona gecer.
anadil_runtime_paylas PROC
    test rcx, rcx
    je anadil_runtime_paylas_done
    mov rax, qword ptr [rcx - 16]
    mov r10, ANADIL_STATIC_REFCOUNT_MIN
    cmp rax, r10
    jge anadil_runtime_paylas_done
    inc qword ptr [rcx - 16]
anadil_runtime_paylas_done:
    ret
anadil_runtime_paylas ENDP

; NOT: RC Faz 1 tek-thread varsayimiyla non-atomic sayac kullaniyor.
; Threading dile girdiginde bu PROC lock'lu versiyona gecer.
anadil_runtime_birak PROC
    push rbp
    mov rbp, rsp
    sub rsp, 48

    test rcx, rcx
    je anadil_runtime_birak_done

    mov rax, qword ptr [rcx - 16]
    mov r10, ANADIL_STATIC_REFCOUNT_MIN
    cmp rax, r10
    jge anadil_runtime_birak_done

    dec qword ptr [rcx - 16]
    jne anadil_runtime_birak_done

    mov qword ptr [rbp - 8], rcx
    call GetProcessHeap
    mov rcx, rax
    xor edx, edx
    mov r8, qword ptr [rbp - 8]
    sub r8, 16
    call HeapFree

anadil_runtime_birak_done:
    add rsp, 48
    pop rbp
    ret
anadil_runtime_birak ENDP

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
    sub rsp, 48

    mov qword ptr [rbp - 8], rcx
    lea rcx, runtime_error_prefix
    call anadil_runtime_write_cstr
    mov rcx, qword ptr [rbp - 8]
    call anadil_runtime_print_metin
    call anadil_runtime_wait_before_exit
    mov ecx, 1
    call ExitProcess
anadil_runtime_panic ENDP

END
