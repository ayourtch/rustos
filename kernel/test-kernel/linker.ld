ENTRY(_start)

SECTIONS
{
    /* Load at 1MB physical address - safe area */
    . = 0x100000;
    
    .text : {
        *(.text*)
    }
    
    .rodata : {
        *(.rodata*)
    }
    
    .data : {
        *(.data*)
    }
    
    .bss : {
        *(.bss*)
    }
}
