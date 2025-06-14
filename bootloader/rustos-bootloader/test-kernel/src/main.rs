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
        let mut width:usize = 2048u32 as usize;
        let mut height:usize = 2048u32 as usize;
        
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
            
            // Let's visualize the actual values by drawing patterns
            // We'll draw rectangles whose size represents the values
            
            // Show fb_info.addr by drawing rectangles (each bit represented)
            if fb_info.addr != 0 {
                // Draw cyan rectangle to show addr is non-zero
                for y in 300..350 {
                    for x in 0..100 {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFF00FFFF; // Cyan - addr non-zero
                    }
                }
            }
            
            // Visualize width by drawing dots - each dot represents 256 pixels of width
            let width_dots = (fb_info.width / 256).min(50); // Max 50 dots
            for i in 0..width_dots {
                for y in 350..370 {
                    for x in (i*4)..(i*4+3) {
                        if x < 200 {
                            let pixel_offset = (y * width + x as usize) as isize;
                            *fb_addr.offset(pixel_offset) = 0xFFFF8000; // Orange dots for width
                        }
                    }
                }
            }
            
            // Visualize height by drawing dots - each dot represents 256 pixels of height  
            let height_dots = (fb_info.height / 256).min(50) as usize; // Max 50 dots
            for i in 0..height_dots {
                for y in 380..400 {
                    for x in (i*4)..(i*4+3) {
                        if x < 200 {
                            let pixel_offset = (y * width + x) as isize;
                            *fb_addr.offset(pixel_offset) = 0xFFFF80FF; // Pink dots for height
                        }
                    }
                }
            }
            
            // Show raw bytes of width and height as patterns
            // Draw width bytes as colored squares
            let width_bytes = fb_info.width.to_le_bytes();
            for (i, &byte) in width_bytes.iter().enumerate() {
                for bit in 0..8 {
                    if (byte >> bit) & 1 != 0 {
                        let x = 210 + i * 10 + bit;
                        for y in 350..360 {
                            if x < width {
                                let pixel_offset = (y * width + x) as isize;
                                *fb_addr.offset(pixel_offset) = 0xFFFFFF00; // Yellow bits for width
                            }
                        }
                    }
                }
            }
            
            // Draw height bytes as colored squares  
            let height_bytes = fb_info.height.to_le_bytes();
            for (i, &byte) in height_bytes.iter().enumerate() {
                for bit in 0..8 {
                    if (byte >> bit) & 1 != 0 {
                        let x = 210 + i * 10 + bit;
                        for y in 370..380 {
                            if x < width {
                                let pixel_offset = (y * width + x) as isize;
                                *fb_addr.offset(pixel_offset) = 0xFF00FF00; // Green bits for height
                            }
                        }
                    }
                }
            }
            
            // If all values look good, try using them
            if fb_info.addr != 0 && fb_info.width > 0 && fb_info.height > 0 
               && fb_info.width < 10000 && fb_info.height < 10000 {
                // Update our values with boot_info values
                fb_addr = fb_info.addr as *mut u32;
                width = fb_info.width as usize;
                height = fb_info.height as usize;
                
                // Draw magenta rectangle to show we're using boot_info framebuffer
                for y in 450..500 {
                    for x in 0..200 {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFFFF00FF; // Magenta - using boot_info fb
                    }
                }
            }
        }
        
        // Now do the normal gradient animation
        let mut time: usize = 0u32 as usize;
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
