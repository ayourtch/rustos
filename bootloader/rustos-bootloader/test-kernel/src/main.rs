#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        let fb_addr = 0x80000000 as *mut u32;
        let width = 2048u32;
        let height = 2048u32;
        
        let mut time = 0u32;
        
        loop {
            // Create a moving color gradient
            for y in 0..height {
                for x in 0..width {
                    let pixel_offset = (y * width + x) as isize;
                    
                    // Create RGB color based on position and time
                    let red = ((x + time) & 0xFF) as u32;
                    let green = ((y + time) & 0xFF) as u32;
                    let blue = (((x + y + time) / 2) & 0xFF) as u32;
                    
                    // Combine into 32-bit RGBA (Alpha=255 for opaque)
                    let color = (255 << 24) | (red << 16) | (green << 8) | blue;
                    
                    *fb_addr.offset(pixel_offset) = color;
                }
            }
            
            time = time.wrapping_add(1);
            
            // Small delay to control animation speed
            for _ in 0..10000 {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Paint screen red if panic
    unsafe {
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..2048*2048 {
            *fb_addr.offset(i) = 0xFFFF0000; // Red with alpha
        }
    }
    loop {}
}
