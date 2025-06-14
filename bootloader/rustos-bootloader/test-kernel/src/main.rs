#![no_std]
#![no_main]

// Let's start with just testing the framebuffer info directly
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub addr: u64,     // 8 bytes
    pub width: u32,    // 4 bytes  
    pub height: u32,   // 4 bytes
    pub pitch: u32,    // 4 bytes
    pub bpp: u32,      // 4 bytes
    pub red_mask: u32,   // 4 bytes
    pub green_mask: u32, // 4 bytes  
    pub blue_mask: u32,  // 4 bytes
}

#[no_mangle]
pub extern "C" fn _start(framebuffer_info: *const FramebufferInfo) -> ! {
    unsafe {
        // Use hardcoded values as fallback
        let fb_addr = 0x80000000 as *mut u32;
        let width = 2048u32;
        
        // Draw blue rectangle to show we started
        for y in 0..100 {
            for x in 0..200 {
                let pixel_offset = (y * width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0xFF0000FF; // Blue
            }
        }
        
        if !framebuffer_info.is_null() {
            let fb_info = &*framebuffer_info;
            
            // Draw green if not null
            for y in 100..150 {
                for x in 0..100 {
                    let pixel_offset = (y * width + x) as isize;
                    *fb_addr.offset(pixel_offset) = 0xFF00FF00; // Green
                }
            }
            
            // Test each field individually
            // Show addr as cyan dots (each dot = 1GB)
            let addr_gb = (fb_info.addr / 0x40000000) as u32; // Divide by 1GB
            for i in 0..addr_gb.min(10) {
                for y in 150..160 {
                    for x in (i*10)..(i*10+8) {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFF00FFFF; // Cyan dots
                    }
                }
            }
            
            // Show width as orange dots (each dot = 256 pixels)
            let width_dots = (fb_info.width / 256).min(20);
            for i in 0..width_dots {
                for y in 170..180 {
                    for x in (i*10)..(i*10+8) {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFFFF8000; // Orange dots
                    }
                }
            }
            
            // Show height as pink dots (each dot = 256 pixels)  
            let height_dots = (fb_info.height / 256).min(20);
            for i in 0..height_dots {
                for y in 190..200 {
                    for x in (i*10)..(i*10+8) {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFFFF00FF; // Pink dots
                    }
                }
            }
            
            // Show the raw bytes of each field
            // Width bytes
            let width_bytes = fb_info.width.to_le_bytes();
            for (i, &byte) in width_bytes.iter().enumerate() {
                let color = match byte {
                    0 => 0xFF000000,      // Black for 0
                    1..=63 => 0xFF0000FF,   // Blue for low values
                    64..=127 => 0xFF00FF00, // Green for mid values  
                    128..=191 => 0xFFFF8000, // Orange for high values
                    192..=255 => 0xFFFF0000, // Red for very high values
                };
                
                for y in 210..220 {
                    for x in (i*20)..(i*20+18) {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = color;
                    }
                }
            }
            
            // Height bytes  
            let height_bytes = fb_info.height.to_le_bytes();
            for (i, &byte) in height_bytes.iter().enumerate() {
                let color = match byte {
                    0 => 0xFF000000,      // Black for 0
                    1..=63 => 0xFF0000FF,   // Blue for low values
                    64..=127 => 0xFF00FF00, // Green for mid values
                    128..=191 => 0xFFFF8000, // Orange for high values  
                    192..=255 => 0xFFFF0000, // Red for very high values
                };
                
                for y in 230..240 {
                    for x in (i*20)..(i*20+18) {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = color;
                    }
                }
            }
        }
        
        // Normal gradient at bottom
        let mut time = 0u32;
        loop {
            for y in 400..600 {
                for x in 0..400 {
                    let pixel_offset = (y * width + x) as isize;
                    let red = ((x + time) & 0xFF) as u32;
                    let green = ((y + time) & 0xFF) as u32;
                    let blue = (((x + y + time) / 2) & 0xFF) as u32;
                    let color = (255 << 24) | (red << 16) | (green << 8) | blue;
                    *fb_addr.offset(pixel_offset) = color;
                }
            }
            time = time.wrapping_add(1);
            for _ in 0..50000 {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
