#![no_std]
#![no_main]

// BootInfo structures from bootloader (simplified)
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
pub extern "C" fn _start(_boot_info: *const BootInfo) -> ! {
    // Write to VGA text mode buffer to show we're alive
    unsafe {
        let vga_buffer = 0xb8000 as *mut u16;
        
        // Write "KERNEL OK!" to VGA text buffer
        let message = b"KERNEL OK!";
        for (i, &byte) in message.iter().enumerate() {
            *vga_buffer.offset(i as isize) = (byte as u16) | 0x0F00; // White on black
        }
        
        // Flash dots to show continuous execution
        let mut counter = 0u32;
        loop {
            // Write a dot that changes position
            let pos = 10 + (counter / 100000) % 70;
            *vga_buffer.offset(pos as isize) = 0x0F2E; // White dot
            
            counter = counter.wrapping_add(1);
            
            // Clear previous dot
            if counter % 100000 == 0 {
                for i in 10..80 {
                    if i != pos {
                        *vga_buffer.offset(i as isize) = 0x0F20; // Space
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
    unsafe {
        // Write "PANIC!" to VGA buffer
        let vga_buffer = 0xb8000 as *mut u16;
        let message = b"PANIC!";
        for (i, &byte) in message.iter().enumerate() {
            *vga_buffer.offset(i as isize) = (byte as u16) | 0x0C00; // Red on black
        }
    }
    
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
