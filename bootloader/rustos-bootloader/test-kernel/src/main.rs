#![no_std]
#![no_main]

// BootInfo structures matching the bootloader
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
    pub entries: *const u8,
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
        // Test 1: Draw purple rectangle to show we got here
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..50000 {
            *fb_addr.offset(i) = 0xFFFF00FF; // Purple - kernel started
        }
        
        // Small delay
        for _ in 0..50000000 {
            core::arch::asm!("nop");
        }
        
        // Test 2: Try to use boot_info parameter
        if !boot_info.is_null() {
            let boot_info = &*boot_info;
            
            // Use framebuffer info from boot_info
            let fb_addr = boot_info.framebuffer.addr as *mut u32;
            let width = boot_info.framebuffer.width;
            let height = boot_info.framebuffer.height;
            
            // Draw cyan rectangle in top-right to show boot_info works
            for y in 0..100 {
                for x in (width-200)..width {
                    let pixel_offset = (y * width + x) as isize;
                    *fb_addr.offset(pixel_offset) = 0xFF00FFFF; // Cyan
                }
            }
            
            // Create animated gradient using proper framebuffer info
            let mut time = 0u32;
            loop {
                for y in 100..300 {
                    for x in 0..400 {
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
                
                for _ in 0..10000 {
                    core::arch::asm!("nop");
                }
            }
        } else {
            // boot_info was null - draw red error
            for i in 0..100000 {
                *fb_addr.offset(i) = 0xFFFF0000; // Red - null boot_info
            }
            
            loop {
                core::arch::asm!("hlt");
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
