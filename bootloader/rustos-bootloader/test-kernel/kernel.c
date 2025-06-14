void _start(void) {
    // Simple infinite loop
    while(1) {
        asm volatile("hlt");
    }
}
