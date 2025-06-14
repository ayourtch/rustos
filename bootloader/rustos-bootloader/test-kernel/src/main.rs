#![no_std]
#![no_main]

#[repr(C)]
pub struct BootInfo {
    pub memory_map: MemoryMapInfo,
    pub framebuffer: FramebufferInfo,
    pub rsdp_addr: Option<u64>,
}

#[repr(C)]
pub struct MemoryMapInfo {
    pub entries: *const u8,
    pub entry_count: usize,
    pub entry_size: usize,
}

#[repr(C)]
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
        // Draw a yellow rectangle IMMEDIATELY when kernel starts
        // This should appear before any other kernel code
        let fb_addr = 0x80000000 as *mut u32; // Use hardcoded framebuffer address for now
        let fb_width = 2048u32; // Use hardcoded width
        
        // Draw yellow rectangle to show kernel started
        for y in 300..350 {
            for x in 0..200 {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0xFFFF00; // Yellow
            }
        }
        
        // Now try to use the boot_info parameter
        let boot_info = &*boot_info;
        
        // Get proper framebuffer info
        let fb_addr = boot_info.framebuffer.addr as *mut u32;
        let fb_width = boot_info.framebuffer.width;
        let fb_height = boot_info.framebuffer.height;
        
        // Draw a green rectangle in top-right corner to show kernel started with boot_info
        for y in 0..50 {
            for x in (fb_width - 100)..fb_width {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0x00FF00; // Green
            }
        }
        
        // Draw animated blue dots
        let mut counter = 0u32;
        loop {
            let x = 200 + (counter / 100000) % 200;
            let y = 100 + ((counter / 200000) % 100);
            
            // Draw blue dot
            for dy in 0..10 {
                for dx in 0..10 {
                    let px = x + dx;
                    let py = y + dy;
                    if px < fb_width && py < fb_height {
                        let pixel_offset = (py * fb_width + px) as isize;
                        *fb_addr.offset(pixel_offset) = 0x0000FF; // Blue
                    }
                }
            }
            
            counter = counter.wrapping_add(1);
            
            // Clear previous dots periodically
            if counter % 50000 == 0 {
                for y in 100..(fb_height.min(200)) {
                    for x in 200..(fb_width.min(400)) {
                        let pixel_offset = (y * fb_width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0x000000; // Black
                    }
                }
            }
            
            // Small delay
            for _ in 0..1000 {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
