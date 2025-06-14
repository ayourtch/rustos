#![no_std]
#![no_main]

// BootInfo structures - must match bootloader exactly
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BootInfo {
    pub memory_map: MemoryMapInfo,
    pub framebuffer: FramebufferInfo,
    pub rsdp_addr: Option<u64>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemoryMapInfo {
    pub entries: *const u8, // We'll just treat as opaque for now
    pub entry_count: usize,
    pub entry_size: usize,
}

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

#[no_mangle]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        // First, test if we can access the parameter at all
        // Start with hardcoded values as fallback
        let mut fb_addr = 0x80000000 as *mut u32;
        let mut width = 2048u32;
        let mut height = 2048u32;
        
        // Draw a blue rectangle to show we started
        for y in 0..100 {
            for x in 0..200 {
                let pixel_offset = (y * width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0xFF0000FF; // Blue - we started
            }
        }
        
        // Try to access boot_info parameter
        if !boot_info.is_null() {
            // Draw green rectangle to show boot_info is not null
            for y in 100..200 {
                for x in 0..200 {
                    let pixel_offset = (y * width + x) as isize;
                    *fb_addr.offset(pixel_offset) = 0xFF00FF00; // Green - boot_info not null
                }
            }
            
            // Try to dereference boot_info
            let boot_info_ref = &*boot_info;
            
            // Draw yellow rectangle to show we can dereference
            for y in 200..300 {
                for x in 0..200 {
                    let pixel_offset = (y * width + x) as isize;
                    *fb_addr.offset(pixel_offset) = 0xFFFFFF00; // Yellow - can dereference
                }
            }
            
            // Try to use framebuffer info from boot_info
            let fb_info = &boot_info_ref.framebuffer;
            if fb_info.addr != 0 && fb_info.width > 0 && fb_info.height > 0 {
                // Update our values with boot_info values
                fb_addr = fb_info.addr as *mut u32;
                width = fb_info.width;
                height = fb_info.height;
                
                // Draw magenta rectangle to show we're using boot_info framebuffer
                for y in 300..400 {
                    for x in 0..200 {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFFFF00FF; // Magenta - using boot_info fb
                    }
                }
            }
        }
        
        // Now do the normal gradient animation
        let mut time = 0u32;
        loop {
            // Create a moving color gradient using the framebuffer info
            for y in 500..1000 { // Use lower part of screen for gradient
                for x in 0..500 {
                    if x < width && y < height {
                        let pixel_offset = (y * width + x) as isize;
                        
                        let red = ((x + time) & 0xFF) as u32;
                        let green = ((y + time) & 0xFF) as u32;
                        let blue = (((x + y + time) / 2) & 0xFF) as u32;
                        
                        let color = (255 << 24) | (red << 16) | (green << 8) | blue;
                        *fb_addr.offset(pixel_offset) = color;
                    }
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
    // Paint screen red if panic
    unsafe {
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..2048*2048 {
            *fb_addr.offset(i) = 0xFFFF0000; // Red
        }
    }
    loop {}
}
