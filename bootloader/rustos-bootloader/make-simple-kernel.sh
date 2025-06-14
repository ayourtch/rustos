cd test-kernel

# Create a simpler linker script with low physical addresses
cat > linker.ld << 'EOF'
ENTRY(_start)

SECTIONS
{
    /* Load at 1MB physical address - this should be safe */
    . = 0x100000;
    
    .text : {
        *(.text)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        *(.bss)
    }
}
EOF

# Rebuild
gcc -ffreestanding -nostdlib -o kernel.elf kernel.c -Wl,--script=linker.ld

cd ..
cp test-kernel/kernel.elf esp/kernel.elf

# Check what we created
readelf -l esp/kernel.elf
