#![no_std]
#![no_main]

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

// Test framebuffer writes at different locations to find the boundary
unsafe fn test_memory_write(test_value: u32, x_offset: u32, y_offset: u32) -> bool {
    let fb_addr = 0x80000000 as *mut u32;
    let fb_width = 2048u32;
    
    // Try to write at a specific location
    let pixel_offset = (y_offset * fb_width + x_offset) as isize;
    
    // Try the write - if it causes a fault, we'll hang
    *fb_addr.offset(pixel_offset) = test_value;
    
    // If we get here, the write succeeded
    true
}

// Draw a single colored rectangle to show we're alive
unsafe fn draw_status_rect(color: u32, rect_id: u32) {
    let fb_addr = 0x80000000 as *mut u32;
    let fb_width = 2048u32;
    
    let start_y = rect_id * 20;
    let end_y = start_y + 15;
    
    for y in start_y..end_y {
        for x in 0..100 {
            if y < 2048 && x < fb_width {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = color;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start(framebuffer_info: *const FramebufferInfo) -> ! {
    unsafe {
        // Test 1: Can we write at all?
        draw_status_rect(0xFF0000FF, 0); // Blue - started
        
        // Test 2: Can we write to framebuffer at position 0,0?
        let fb_addr = 0x80000000 as *mut u32;
        *fb_addr = 0xFF00FF00; // Green pixel at 0,0
        draw_status_rect(0xFF00FF00, 1); // Green rect - basic write works
        
        // Test 3: Can we write to position 100,0?
        if test_memory_write(0xFFFF0000, 100, 0) {
            draw_status_rect(0xFFFF0000, 2); // Red rect - position 100,0 works
        }
        
        // Test 4: Can we write to position 1000,0?
        if test_memory_write(0xFFFFFF00, 1000, 0) {
            draw_status_rect(0xFFFFFF00, 3); // Yellow rect - position 1000,0 works
        }
        
        // Test 5: Can we write to position 2000,0?
        if test_memory_write(0xFFFF00FF, 2000, 0) {
            draw_status_rect(0xFFFF00FF, 4); // Magenta rect - position 2000,0 works
        }
        
        // Test 6: Can we write to position 0,100?
        if test_memory_write(0xFF00FFFF, 0, 100) {
            draw_status_rect(0xFF00FFFF, 5); // Cyan rect - position 0,100 works
        }
        
        // Test 7: Can we write to position 0,1000?
        if test_memory_write(0xFF808080, 0, 1000) {
            draw_status_rect(0xFF808080, 6); // Gray rect - position 0,1000 works
        }
        
        // Test 8: Can we write to position 0,2000?
        if test_memory_write(0xFFFFC0C0, 0, 2000) {
            draw_status_rect(0xFFFFC0C0, 7); // Light red rect - position 0,2000 works
        }
        
        // Test 9: Test parameter access
        if !framebuffer_info.is_null() {
            draw_status_rect(0xFF004000, 8); // Dark green - pointer not null
            
            // Try to read addr field (first field)
            let fb_info = &*framebuffer_info;
            let _addr = fb_info.addr; // This might cause the hang!
            draw_status_rect(0xFF008000, 9); // Brighter green - can read addr
            
            // Try to read width field  
            let _width = fb_info.width;
            draw_status_rect(0xFF00C000, 10); // Even brighter green - can read width
        }
        
        // If we get here, everything worked
        draw_status_rect(0xFFFFFFFF, 15); // White rect at bottom - all tests passed
        
        loop {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Fill entire screen with red to indicate panic
    unsafe {
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..(2048 * 2048) {
            *fb_addr.offset(i) = 0xFFFF0000;
        }
    }
    loop {}
}
