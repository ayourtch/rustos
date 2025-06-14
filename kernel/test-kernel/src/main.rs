#![no_std]
#![no_main]

// BootInfo structures from bootloader
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
    pub entries: *const u8, // We don't need to parse this for our simple kernel
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

static mut FRAMEBUFFER: Option<&'static mut [u32]> = None;
static mut FB_WIDTH: u32 = 0;
static mut FB_HEIGHT: u32 = 0;
static mut DOT_X: u32 = 0;
static mut DOT_Y: u32 = 50; // Start a bit down from the top

#[no_mangle]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        let boot_info = &*boot_info;
        
        // Set up framebuffer access
        let fb_addr = boot_info.framebuffer.addr as *mut u32;
        let fb_size = (boot_info.framebuffer.width * boot_info.framebuffer.height) as usize;
        FRAMEBUFFER = Some(core::slice::from_raw_parts_mut(fb_addr, fb_size));
        FB_WIDTH = boot_info.framebuffer.width;
        FB_HEIGHT = boot_info.framebuffer.height;
        
        // Clear screen to blue
        clear_screen(0x0000FF);
        
        // Print a startup message
        draw_text("RustOS Kernel Started! Dots every second:", 10, 10, 0xFFFFFF);
        
        // Main loop - print a dot every second
        loop {
            // Draw a white dot
            draw_dot(DOT_X, DOT_Y, 0xFFFFFF);
            
            // Move to next position
            DOT_X += 20;
            if DOT_X >= FB_WIDTH - 20 {
                DOT_X = 0;
                DOT_Y += 20;
                if DOT_Y >= FB_HEIGHT - 20 {
                    DOT_Y = 70; // Reset below the text
                }
            }
            
            // Wait approximately 1 second (very rough timing)
            busy_wait(1000000000); // Adjust this value as needed
        }
    }
}

unsafe fn clear_screen(color: u32) {
    if let Some(fb) = &mut FRAMEBUFFER {
        for pixel in fb.iter_mut() {
            *pixel = color;
        }
    }
}

unsafe fn draw_dot(x: u32, y: u32, color: u32) {
    if let Some(fb) = &mut FRAMEBUFFER {
        if x < FB_WIDTH && y < FB_HEIGHT {
            // Draw a 5x5 dot
            for dy in 0..5 {
                for dx in 0..5 {
                    let px = x + dx;
                    let py = y + dy;
                    if px < FB_WIDTH && py < FB_HEIGHT {
                        let index = (py * FB_WIDTH + px) as usize;
                        if index < fb.len() {
                            fb[index] = color;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn draw_text(text: &str, start_x: u32, start_y: u32, color: u32) {
    let mut x = start_x;
    let y = start_y;
    
    for ch in text.chars() {
        if ch == ' ' {
            x += 8;
            continue;
        }
        
        // Simple bitmap font for a few characters
        let bitmap = match ch {
            'R' => [0x7C, 0x44, 0x44, 0x7C, 0x48, 0x44, 0x44, 0x00],
            'u' => [0x00, 0x00, 0x44, 0x44, 0x44, 0x4C, 0x34, 0x00],
            's' => [0x00, 0x00, 0x38, 0x40, 0x30, 0x08, 0x70, 0x00],
            't' => [0x20, 0x20, 0x70, 0x20, 0x20, 0x24, 0x18, 0x00],
            'O' => [0x38, 0x44, 0x44, 0x44, 0x44, 0x44, 0x38, 0x00],
            'S' => [0x38, 0x44, 0x40, 0x38, 0x04, 0x44, 0x38, 0x00],
            'K' => [0x44, 0x48, 0x50, 0x60, 0x50, 0x48, 0x44, 0x00],
            'e' => [0x00, 0x00, 0x38, 0x44, 0x78, 0x40, 0x38, 0x00],
            'r' => [0x00, 0x00, 0x58, 0x64, 0x40, 0x40, 0x40, 0x00],
            'n' => [0x00, 0x00, 0x58, 0x64, 0x44, 0x44, 0x44, 0x00],
            'l' => [0x60, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
            'd' => [0x04, 0x04, 0x34, 0x4C, 0x44, 0x44, 0x3C, 0x00],
            '!' => [0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x20, 0x00],
            'D' => [0x78, 0x44, 0x44, 0x44, 0x44, 0x44, 0x78, 0x00],
            'o' => [0x00, 0x00, 0x38, 0x44, 0x44, 0x44, 0x38, 0x00],
            'v' => [0x00, 0x00, 0x44, 0x44, 0x44, 0x28, 0x10, 0x00],
            'y' => [0x00, 0x00, 0x44, 0x44, 0x2C, 0x14, 0x08, 0x10],
            'c' => [0x00, 0x00, 0x38, 0x44, 0x40, 0x44, 0x38, 0x00],
            ':' => [0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00],
            _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Unknown char
        };
        
        // Draw the character
        for row in 0..8 {
            let byte = bitmap[row];
            for col in 0..8 {
                if (byte >> (7 - col)) & 1 != 0 {
                    let px = x + col;
                    let py = y + row as u32;
                    if px < FB_WIDTH && py < FB_HEIGHT {
                        let index = (py * FB_WIDTH + px) as usize;
                        if let Some(fb) = &mut FRAMEBUFFER {
                            if index < fb.len() {
                                fb[index] = color;
                            }
                        }
                    }
                }
            }
        }
        
        x += 8; // Move to next character position
    }
}

fn busy_wait(cycles: u64) {
    for _ in 0..cycles {
        unsafe {
            core::arch::asm!("nop");
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
