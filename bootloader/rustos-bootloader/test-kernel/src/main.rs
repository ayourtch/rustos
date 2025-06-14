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

// Serial debugging functions
unsafe fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

unsafe fn serial_write_byte(byte: u8) {
    const SERIAL_PORT: u16 = 0x3F8;
    while (inb(SERIAL_PORT + 5) & 0x20) == 0 {}
    outb(SERIAL_PORT, byte);
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

unsafe fn serial_write_hex(value: u64) {
    serial_write_str("0x");
    let hex_chars = b"0123456789ABCDEF";
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as usize;
        serial_write_byte(hex_chars[digit]);
    }
}

unsafe fn serial_write_u32(value: u32) {
    if value == 0 {
        serial_write_byte(b'0');
        return;
    }
    
    let mut digits = [0u8; 10];
    let mut count = 0;
    let mut val = value;
    
    while val > 0 {
        digits[count] = (val % 10) as u8 + b'0';
        val /= 10;
        count += 1;
    }
    
    for i in (0..count).rev() {
        serial_write_byte(digits[i]);
    }
}

#[no_mangle]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        // Simple serial debug - just one message
        serial_write_str("KERNEL: Started\n");
        
        // Test 1: Draw purple rectangle to show we got here
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..50000 {
            *fb_addr.offset(i) = 0xFFFF00FF; // Purple - kernel started
        }
        
        // Test if boot_info is valid by checking if it's not null and not obviously bad
        if !boot_info.is_null() && (boot_info as u64) < 0xFFFF_FFFF_0000_0000 {
            serial_write_str("KERNEL: boot_info looks valid\n");
            
            let boot_info = &*boot_info;
            
            // Check if framebuffer address looks reasonable
            if boot_info.framebuffer.addr == 0x80000000 && 
               boot_info.framebuffer.width == 2048 && 
               boot_info.framebuffer.height == 2048 {
                
                serial_write_str("KERNEL: framebuffer info correct\n");
                
                // Draw cyan rectangle in top-right to show boot_info works
                let fb_addr = boot_info.framebuffer.addr as *mut u32;
                let width = boot_info.framebuffer.width;
                
                for y in 0..100 {
                    for x in (width-200)..width {
                        let pixel_offset = (y * width + x) as isize;
                        *fb_addr.offset(pixel_offset) = 0xFF00FFFF; // Cyan
                    }
                }
                
                serial_write_str("KERNEL: drew cyan rectangle\n");
            } else {
                serial_write_str("KERNEL: framebuffer info wrong\n");
            }
        } else {
            serial_write_str("KERNEL: boot_info invalid\n");
        }
        
        // Simple animation without using boot_info
        let fb_addr = 0x80000000 as *mut u32;
        let mut time = 0u32;
        loop {
            for y in 100..300 {
                for x in 0..400 {
                    let pixel_offset = (y * 2048 + x) as isize;
                    
                    let red = ((x + time) & 0xFF) as u32;
                    let green = ((y + time) & 0xFF) as u32;
                    let blue = (((x + y + time) / 2) & 0xFF) as u32;
                    
                    let color = (255 << 24) | (red << 16) | (green << 8) | blue;
                    *fb_addr.offset(pixel_offset) = color;
                }
            }
            
            time = time.wrapping_add(1);
            
            for _ in 0..10000 {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        serial_write_str("KERNEL: PANIC occurred!\n");
        if let Some(location) = info.location() {
            serial_write_str("KERNEL: Panic at line: ");
            serial_write_u32(location.line());
            serial_write_str("\n");
        }
        
        // Paint screen red if panic
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..2048*2048 {
            *fb_addr.offset(i) = 0xFFFF0000; // Red
        }
    }
    loop {}
}
