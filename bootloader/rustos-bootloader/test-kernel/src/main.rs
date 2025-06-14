#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // No parameters - just like the assembly kernel
    unsafe {
        // Paint framebuffer green to show Rust kernel without params works
        let fb_addr = 0x80000000 as *mut u32;
        let pixel_count = 2048 * 2048;
        
        for i in 0..pixel_count {
            *fb_addr.offset(i) = 0x00FF00; // Green
        }
    }
    
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Paint screen red if panic occurs
    unsafe {
        let fb_addr = 0x80000000 as *mut u32;
        let pixel_count = 2048 * 2048;
        
        for i in 0..pixel_count {
            *fb_addr.offset(i) = 0xFF0000; // Red
        }
    }
    
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
