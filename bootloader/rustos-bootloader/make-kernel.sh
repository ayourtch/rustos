mkdir test-kernel
cd test-kernel

# Create a minimal C kernel
cat > kernel.c << 'EOF'
void _start(void) {
    // Simple infinite loop
    while(1) {
        asm volatile("hlt");
    }
}
EOF

# Compile as a standalone ELF
gcc -ffreestanding -nostdlib -o kernel.elf kernel.c -Wl,--entry=_start

cd ..
cp test-kernel/kernel.elf esp/kernel.elf
