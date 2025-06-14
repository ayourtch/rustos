.section .text
.global _start

_start:
    # Write "OK!" to VGA text buffer at 0xb8000
    mov $0xb8000, %rdi
    movw $0x0F4F, (%rdi)     # 'O' in white
    movw $0x0F4B, 2(%rdi)    # 'K' in white  
    movw $0x0F21, 4(%rdi)    # '!' in white
    
    # Infinite loop
halt_loop:
    hlt
    jmp halt_loop
