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
