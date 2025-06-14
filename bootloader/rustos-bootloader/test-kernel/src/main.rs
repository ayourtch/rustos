#![no_std]
#![no_main]

// BootInfo structures (must match bootloader)
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

// Simple character patterns (just a few characters to keep it small)
const FONT_0: [u8; 8] = [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00]; // 0
const FONT_1: [u8; 8] = [0x18, 0x18, 0x38, 0x18, 0x18, 0x18, 0x7E, 0x00]; // 1
const FONT_2: [u8; 8] = [0x3C, 0x66, 0x06, 0x0C, 0x30, 0x60, 0x7E, 0x00]; // 2
const FONT_3: [u8; 8] = [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00]; // 3
const FONT_4: [u8; 8] = [0x06, 0x0E, 0x1E, 0x66, 0x7F, 0x06, 0x06, 0x00]; // 4
const FONT_5: [u8; 8] = [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00]; // 5
const FONT_6: [u8; 8] = [0x3C, 0x66, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00]; // 6
const FONT_7: [u8; 8] = [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00]; // 7
const FONT_8: [u8; 8] = [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00]; // 8
const FONT_9: [u8; 8] = [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x66, 0x3C, 0x00]; // 9
const FONT_O: [u8; 8] = [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00]; // O
const FONT_K: [u8; 8] = [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00]; // K

fn get_digit_font(digit: u8) -> &'static [u8; 8] {
    match digit {
        0 => &FONT_0,
        1 => &FONT_1,
        2 => &FONT_2,
        3 => &FONT_3,
        4 => &FONT_4,
        5 => &FONT_5,
        6 => &FONT_6,
        7 => &FONT_7,
        8 => &FONT_8,
        9 => &FONT_9,
        _ => &FONT_0,
    }
}

unsafe fn draw_char_pattern(fb_addr: *mut u32, width: u32, x: u32, y: u32, pattern: &[u8; 8], color: u32) {
    for row in 0..8 {
        let byte = pattern[row];
        for col in 0..8 {
            if (byte >> (7 - col)) & 1 != 0 {
                let px = x + col;
                let py = y + row as u32;
                let pixel_offset = (py * width + px) as isize;
                *fb_addr.offset(pixel_offset) = color;
            }
        }
    }
}

unsafe fn draw_number(fb_addr: *mut u32, width: u32, x: u32, y: u32, mut num: u32, color: u32) {
    if num == 0 {
        draw_char_pattern(fb_addr, width, x, y, get_digit_font(0), color);
        return;
    }
    
    // Draw up to 8 digits
    let mut digits = [10u8; 8]; // Invalid digit means empty
    let mut digit_count = 0;
    
    while num > 0 && digit_count < 8 {
        digits[digit_count] = (num % 10) as u8;
        num /= 10;
        digit_count += 1;
    }
    
    // Draw digits from left to right
    for i in 0..digit_count {
        let digit = digits[digit_count - 1 - i];
        draw_char_pattern(fb_addr, width, x + (i as u32 * 9), y, get_digit_font(digit), color);
    }
}

#[no_mangle]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        // Get boot info
        let boot_info = &*boot_info;
        let fb_addr = boot_info.framebuffer.addr as *mut u32;
        let fb_width = boot_info.framebuffer.width;
        
        // Clear top part of screen
        for y in 0..200 {
            for x in 0..fb_width {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0xFF000000; // Black
            }
        }
        
        // Draw "OK" using available font patterns
        draw_char_pattern(fb_addr, fb_width, 50, 50, &FONT_O, 0xFF00FF00); // Green O
        draw_char_pattern(fb_addr, fb_width, 70, 50, &FONT_K, 0xFF00FF00); // Green K
        
        let mut counter = 0u32;
        
        loop {
            // Clear the counter area
            for y in 100..120 {
                for x in 50..200 {
                    let pixel_offset = (y * fb_width + x) as isize;
                    *fb_addr.offset(pixel_offset) = 0xFF000000; // Black
                }
            }
            
            // Draw the counter
            draw_number(fb_addr, fb_width, 50, 100, counter, 0xFFFFFFFF); // White
            
            counter += 1;
            
            // Delay
            for _ in 0..50000000 {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // Just set a few pixels to red to indicate panic
        let fb_addr = 0x80000000 as *mut u32;
        for i in 0..1000 {
            *fb_addr.offset(i) = 0xFFFF0000; // Red
        }
    }
    loop {}
}
