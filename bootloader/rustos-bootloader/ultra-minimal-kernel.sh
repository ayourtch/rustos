cd test-kernel

# Create an even simpler kernel
cat > ultra-minimal.s << 'EOF'
.section .text
.global _start

_start:
    # Immediately write "HI!" to VGA text buffer
    movq $0xb8000, %rax
    movw $0x0F48, (%rax)     # 'H' in white
    movw $0x0F49, 2(%rax)    # 'I' in white  
    movw $0x0F21, 4(%rax)    # '!' in white
    
    # Simple infinite loop - don't even use hlt
spin:
    jmp spin
EOF

# Build with very basic linker script
cat > simple.ld << 'EOF'
ENTRY(_start)
SECTIONS {
    . = 0x100000;
    .text : { *(.text*) }
}
EOF

as --64 -o ultra-minimal.o ultra-minimal.s
ld -T simple.ld -o ultra-minimal ultra-minimal.o

cd ..
cp test-kernel/ultra-minimal esp/kernel.elf
