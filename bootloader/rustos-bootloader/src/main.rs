#![no_std]
#![no_main]

extern crate alloc;
use uefi_services::println;


use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use core::slice;
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

const BOOT_INFO_ADDR: u64 = 0x8000_0000;
const KERNEL_ADDR: u64 = 0x4000_0000;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; 512],
}

impl PageTable {
    fn new() -> Self {
        Self {
            entries: [0; 512],
        }
    }
    
    fn set_entry(&mut self, index: usize, addr: u64, flags: u64) {
        self.entries[index] = addr | flags;
    }
}

const PAGE_PRESENT: u64 = 1 << 0;
const PAGE_WRITABLE: u64 = 1 << 1;
const PAGE_HUGE: u64 = 1 << 7;

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
    pub entries: *const MemoryDescriptor,
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

#[entry]
fn efi_main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    
    // Add debug output right at the start
    system_table.stdout().write_str("RustOS Bootloader Starting...\n").unwrap();
    
    // Set up graphics mode
    system_table.stdout().write_str("Setting up graphics...\n").unwrap();
    let framebuffer_info = setup_graphics(system_table.boot_services()).expect("Failed to setup graphics");
    
    // Debug: Print framebuffer info
    system_table.stdout().write_str("Framebuffer addr: 0x").unwrap();
    print_hex(&mut system_table, framebuffer_info.addr);
    system_table.stdout().write_str("\n").unwrap();
    system_table.stdout().write_str("Framebuffer width: ").unwrap();
    print_decimal(&mut system_table, framebuffer_info.width as u64);
    system_table.stdout().write_str("\n").unwrap();
    system_table.stdout().write_str("Framebuffer height: ").unwrap();
    print_decimal(&mut system_table, framebuffer_info.height as u64);
    system_table.stdout().write_str("\n").unwrap();
    
    // Load kernel from filesystem
    system_table.stdout().write_str("Loading kernel...\n").unwrap();
    let kernel_data = load_kernel(system_table.boot_services(), image).expect("Failed to load kernel");
    
    // Parse ELF and get entry point
    system_table.stdout().write_str("Parsing ELF...\n").unwrap();
    let entry_point = parse_elf_and_load(&kernel_data, system_table.boot_services()).expect("Failed to parse ELF");
    
    // Debug: Print the entry point address and where we loaded segments
    system_table.stdout().write_str("Entry point: 0x").unwrap();
    print_hex(&mut system_table, entry_point);
    system_table.stdout().write_str(" (jumping to this address)\n").unwrap();
    
    // Set up identity mapping for first 1GB
    system_table.stdout().write_str("Setting up identity mapping...\n").unwrap();
    setup_identity_mapping(system_table.boot_services()).expect("Failed to setup identity mapping");
    
    // Allocate kernel stack before exiting boot services
    system_table.stdout().write_str("Allocating kernel stack...\n").unwrap();
    let stack_pages = system_table.boot_services().allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        16, // 16 pages = 64KB
    ).expect("Failed to allocate kernel stack");
    let stack_top = stack_pages + (16 * 0x1000); // Stack grows downward
    
    // Get memory map before exiting boot services
    system_table.stdout().write_str("Getting memory map...\n").unwrap();
    system_table.stdout().write_str("About to call get_memory_map\n").unwrap();
    let memory_map_info = get_memory_map(system_table.boot_services()).expect("Failed to get memory map");
    system_table.stdout().write_str("Memory map obtained successfully\n").unwrap();
    system_table.stdout().write_str("Memory map has entries and continuing...\n").unwrap();
    system_table.stdout().write_str("About to proceed to RSDP search...\n").unwrap();
    
    // Find RSDP
    system_table.stdout().write_str("Finding RSDP...\n").unwrap();
    let rsdp_addr = find_rsdp(&mut system_table);
    system_table.stdout().write_str("RSDP search completed\n").unwrap();
    
    // Create BootInfo structure
    system_table.stdout().write_str("Creating BootInfo structure...\n").unwrap();
    let boot_info = BootInfo {
        memory_map: memory_map_info,
        framebuffer: framebuffer_info,
        rsdp_addr,
    };
    
    // Allocate memory for BootInfo through UEFI boot services
    system_table.stdout().write_str("Allocating memory for BootInfo...\n").unwrap();
    let boot_info_addr = system_table.boot_services().allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1, // 1 page should be enough for BootInfo
    ).expect("Failed to allocate BootInfo memory");
    
    system_table.stdout().write_str("Placing BootInfo at allocated address...\n").unwrap();
    unsafe {
        let boot_info_ptr = boot_info_addr as *mut BootInfo;
        system_table.stdout().write_str("About to write BootInfo to memory...\n").unwrap();
        *boot_info_ptr = boot_info;
        system_table.stdout().write_str("BootInfo write completed...\n").unwrap();
    }
    system_table.stdout().write_str("BootInfo placed successfully\n").unwrap();
    
    // Exit boot services - UEFI 0.26 API takes only MemoryType parameter
    system_table.stdout().write_str("Exiting boot services...\n").unwrap();
    let (_runtime_system_table, _memory_map) = system_table
        .exit_boot_services(MemoryType::LOADER_DATA);
    
    // At this point, we can't use stdout anymore
    // Now try to write to the framebuffer with expanded identity mapping
    unsafe {
        // Get framebuffer info from our boot_info
        let boot_info_ref = &*(boot_info_addr as *const BootInfo);
        let fb_addr = boot_info_ref.framebuffer.addr as *mut u32;
        let fb_width = boot_info_ref.framebuffer.width;
        
        // Draw a red rectangle in top-left corner
        for y in 0..100 {
            for x in 0..200 {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0xFF0000; // Red
            }
        }
        
        // Small delay to make it visible
        for _ in 0..50000000 {
            core::arch::asm!("nop");
        }
    }
    
    // Jump to kernel
    unsafe {
        // Draw a blue rectangle before jumping to show we're about to call kernel
        let boot_info_ref = &*(boot_info_addr as *const BootInfo);
        let fb_addr = boot_info_ref.framebuffer.addr as *mut u32;
        let fb_width = boot_info_ref.framebuffer.width;
        
        for y in 200..250 {
            for x in 0..200 {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0x0000FF; // Blue rectangle - "about to jump"
            }
        }
        
        // Small delay
        for _ in 0..10000000 {
            core::arch::asm!("nop");
        }
        
    // Jump to kernel
    unsafe {
        // Draw a blue rectangle before jumping to show we're about to call kernel
        let boot_info_ref = &*(boot_info_addr as *const BootInfo);
        let fb_addr = boot_info_ref.framebuffer.addr as *mut u32;
        let fb_width = boot_info_ref.framebuffer.width;
        
        for y in 200..250 {
            for x in 0..200 {
                let pixel_offset = (y * fb_width + x) as isize;
                *fb_addr.offset(pixel_offset) = 0x0000FF; // Blue rectangle - "about to jump"
            }
        }
        
        // Small delay
        for _ in 0..10000000 {
            core::arch::asm!("nop");
        }
        
        // Set up stack and jump to kernel
        core::arch::asm!(
            "mov rsp, {stack_top}",      // Set up stack pointer
            "push rbp",                   // Set up frame pointer
            "mov rbp, rsp",
            "call {entry_point}",         // Call kernel entry point
            stack_top = in(reg) stack_top,
            entry_point = in(reg) entry_point,
            in("rdi") boot_info_addr,     // Pass boot_info as first parameter
            options(noreturn)
        );
    }
    }
}

fn setup_graphics(boot_services: &BootServices) -> Result<FramebufferInfo, uefi::Error> {
    let gop_handle = boot_services
        .get_handle_for_protocol::<GraphicsOutput>()?;
    
    let mut gop = boot_services
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;
    
    // Find best mode (highest resolution with 32-bit color)
    let mut best_mode = None;
    let mut best_pixels = 0;
    
    for mode_idx in 0..gop.modes(boot_services).count() {
        if let Ok(mode) = gop.query_mode(mode_idx as u32, boot_services) {
            let info = mode.info();
            let pixels = info.resolution().0 * info.resolution().1;
            
            if matches!(info.pixel_format(), PixelFormat::Rgb | PixelFormat::Bgr) && pixels > best_pixels {
                best_mode = Some(mode_idx as u32);
                best_pixels = pixels;
            }
        }
    }
    
    if let Some(mode_idx) = best_mode {
        if let Ok(mode) = gop.query_mode(mode_idx, boot_services) {
            gop.set_mode(&mode)?;
        }
    }
    
    let mode_info = gop.current_mode_info();
    let mut framebuffer = gop.frame_buffer();
    
    let (red_mask, green_mask, blue_mask) = match mode_info.pixel_format() {
        PixelFormat::Rgb => (0x00_FF_00_00, 0x00_00_FF_00, 0x00_00_00_FF),
        PixelFormat::Bgr => (0x00_00_00_FF, 0x00_00_FF_00, 0x00_FF_00_00),
        _ => (0, 0, 0),
    };
    
    Ok(FramebufferInfo {
        addr: framebuffer.as_mut_ptr() as u64,
        width: mode_info.resolution().0 as u32,
        height: mode_info.resolution().1 as u32,
        pitch: mode_info.stride() as u32 * 4,
        bpp: 32,
        red_mask,
        green_mask,
        blue_mask,
    })
}

fn load_kernel(boot_services: &BootServices, image: Handle) -> Result<Vec<u8>, uefi::Error> {
    let loaded_image = boot_services.open_protocol_exclusive::<uefi::proto::loaded_image::LoadedImage>(image)?;
    let device_handle = loaded_image.device().unwrap();
    
    let mut fs = boot_services.open_protocol_exclusive::<SimpleFileSystem>(device_handle)?;
    let mut root = fs.open_volume()?;
    
    let mut kernel_file = root.open(
        cstr16!("kernel.elf"),
        FileMode::Read,
        FileAttribute::empty(),
    )?.into_regular_file().unwrap();
    
    let mut file_info_buf = [0u8; 512];
    let file_info = kernel_file.get_info::<uefi::proto::media::file::FileInfo>(&mut file_info_buf)
        .map_err(|e| e.status())?;
    let file_size = file_info.file_size() as usize;
    
    let mut buffer = vec![0u8; file_size];
    kernel_file.read(&mut buffer)?;
    
    Ok(buffer)
}

fn parse_elf_and_load(elf_data: &[u8], boot_services: &BootServices) -> Result<u64, &'static str> {
    if elf_data.len() < 64 {
        return Err("ELF too small");
    }
    
    // Check ELF magic
    if &elf_data[0..4] != b"\x7fELF" {
        return Err("Invalid ELF magic");
    }
    
    // Check 64-bit
    if elf_data[4] != 2 {
        return Err("Not 64-bit ELF");
    }
    
    // Get entry point (at offset 24 for 64-bit ELF)
    let entry_point = u64::from_le_bytes([
        elf_data[24], elf_data[25], elf_data[26], elf_data[27],
        elf_data[28], elf_data[29], elf_data[30], elf_data[31],
    ]);
    
    // Get program header info
    let ph_offset = u64::from_le_bytes([
        elf_data[32], elf_data[33], elf_data[34], elf_data[35],
        elf_data[36], elf_data[37], elf_data[38], elf_data[39],
    ]) as usize;
    
    let ph_entry_size = u16::from_le_bytes([elf_data[54], elf_data[55]]) as usize;
    let ph_num = u16::from_le_bytes([elf_data[56], elf_data[57]]) as usize;
    
    // Load program segments
    for i in 0..ph_num {
        let ph_start = ph_offset + i * ph_entry_size;
        if ph_start + 56 > elf_data.len() {
            continue;
        }
        
        let ph = &elf_data[ph_start..ph_start + 56];
        let p_type = u32::from_le_bytes([ph[0], ph[1], ph[2], ph[3]]);
        
        // PT_LOAD = 1
        if p_type == 1 {
            let p_offset = u64::from_le_bytes([
                ph[8], ph[9], ph[10], ph[11], ph[12], ph[13], ph[14], ph[15],
            ]) as usize;
            
            let p_vaddr = u64::from_le_bytes([
                ph[16], ph[17], ph[18], ph[19], ph[20], ph[21], ph[22], ph[23],
            ]);
            
            let p_filesz = u64::from_le_bytes([
                ph[32], ph[33], ph[34], ph[35], ph[36], ph[37], ph[38], ph[39],
            ]) as usize;
            
            let p_memsz = u64::from_le_bytes([
                ph[40], ph[41], ph[42], ph[43], ph[44], ph[45], ph[46], ph[47],
            ]) as usize;
            
            // Allocate pages for this segment
            let pages_needed = (p_memsz + 0xFFF) / 0x1000;
            
            // Debug: Print what we're trying to load
            println!("Loading segment: vaddr=0x{:x}, filesz=0x{:x}, memsz=0x{:x}", p_vaddr, p_filesz, p_memsz);
            
            // Try to allocate at the requested virtual address first
            let segment_addr = match boot_services.allocate_pages(
                uefi::table::boot::AllocateType::Address(p_vaddr),
                MemoryType::LOADER_DATA,
                pages_needed,
            ) {
                Ok(addr) => {
                    println!("Allocated segment at requested address: 0x{:x}", addr);
                    addr
                },
                Err(_) => {
                    // If that fails, allocate anywhere and we'll need to update the mapping
                    let addr = boot_services.allocate_pages(
                        uefi::table::boot::AllocateType::AnyPages,
                        MemoryType::LOADER_DATA,
                        pages_needed,
                    ).map_err(|_| "Failed to allocate segment memory anywhere")?;
                    println!("Allocated segment at fallback address: 0x{:x} (requested 0x{:x})", addr, p_vaddr);
                    addr
                }
            };
            
            // Copy segment data
            unsafe {
                let dest = segment_addr as *mut u8;
                if p_filesz > 0 && p_offset + p_filesz <= elf_data.len() {
                    core::ptr::copy_nonoverlapping(
                        elf_data.as_ptr().add(p_offset),
                        dest,
                        p_filesz,
                    );
                }
                // Zero remaining bytes
                if p_memsz > p_filesz {
                    core::ptr::write_bytes(dest.add(p_filesz), 0, p_memsz - p_filesz);
                }
            }
        }
    }
    
    Ok(entry_point)
}

fn setup_identity_mapping(boot_services: &BootServices) -> Result<(), &'static str> {
    // Allocate pages for page tables
    let pml4_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PML4")?;
    
    let pdpt_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PDPT")?;
    
    // Allocate more PD pages to cover more memory
    let pd0_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PD0")?;
    
    let pd1_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PD1")?;
    
    let pd2_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PD2")?;
    
    let pd3_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PD3")?;
    
    unsafe {
        // Clear all page tables
        core::ptr::write_bytes(pml4_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pdpt_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pd0_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pd1_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pd2_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pd3_addr as *mut u8, 0, 0x1000);
        
        let pml4 = &mut *(pml4_addr as *mut PageTable);
        let pdpt = &mut *(pdpt_addr as *mut PageTable);
        let pd0 = &mut *(pd0_addr as *mut PageTable);
        let pd1 = &mut *(pd1_addr as *mut PageTable);
        let pd2 = &mut *(pd2_addr as *mut PageTable);
        let pd3 = &mut *(pd3_addr as *mut PageTable);
        
        // Set up PML4[0] -> PDPT
        pml4.set_entry(0, pdpt_addr, PAGE_PRESENT | PAGE_WRITABLE);
        
        // Set up PDPT entries to cover first 4GB
        pdpt.set_entry(0, pd0_addr, PAGE_PRESENT | PAGE_WRITABLE); // 0-1GB
        pdpt.set_entry(1, pd1_addr, PAGE_PRESENT | PAGE_WRITABLE); // 1-2GB
        pdpt.set_entry(2, pd2_addr, PAGE_PRESENT | PAGE_WRITABLE); // 2-3GB
        pdpt.set_entry(3, pd3_addr, PAGE_PRESENT | PAGE_WRITABLE); // 3-4GB
        
        // Set up PD entries for first 4GB (4 * 512 * 2MB pages)
        let mut pds = [pd0, pd1, pd2, pd3];
        for (pdpt_entry, pd) in pds.iter_mut().enumerate() {
            for pd_entry in 0..512 {
                let addr = (pdpt_entry as u64 * 0x4000_0000) + (pd_entry as u64 * 0x20_0000); // 2MB pages
                pd.set_entry(pd_entry, addr, PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE);
            }
        }
        
        // Load CR3 register
        load_cr3(pml4_addr);
    }
    
    Ok(())
}

#[inline(always)]
unsafe fn load_cr3(pml4_addr: u64) {
    core::arch::asm!(
        "mov cr3, {}",
        in(reg) pml4_addr,
        options(nostack, preserves_flags)
    );
}

fn get_memory_map(boot_services: &BootServices) -> Result<MemoryMapInfo, uefi::Error> {
    let map_size = boot_services.memory_map_size().map_size + 8 * mem::size_of::<MemoryDescriptor>();
    
    let map_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        (map_size + 0xFFF) / 0x1000,
    )?;
    
    let map_buffer = unsafe {
        slice::from_raw_parts_mut(map_addr as *mut u8, map_size)
    };
    
    let _memory_map = boot_services.memory_map(map_buffer)?;
    
    // Don't count entries - just estimate based on size
    let entry_size = mem::size_of::<MemoryDescriptor>();
    let estimated_entry_count = map_size / entry_size;
    
    Ok(MemoryMapInfo {
        entries: map_addr as *const MemoryDescriptor,
        entry_count: estimated_entry_count,
        entry_size,
    })
}

fn find_rsdp(system_table: &mut SystemTable<Boot>) -> Option<u64> {
    for entry in system_table.config_table() {
        if entry.guid == uefi::table::cfg::ACPI2_GUID {
            return Some(entry.address as u64);
        }
        if entry.guid == uefi::table::cfg::ACPI_GUID {
            return Some(entry.address as u64);
        }
    }
    None
}

fn print_hex(system_table: &mut SystemTable<Boot>, value: u64) {
    let hex_chars = b"0123456789ABCDEF";
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as usize;
        let ch = hex_chars[digit] as char;
        system_table.stdout().write_char(ch).unwrap();
    }
}

fn print_decimal(system_table: &mut SystemTable<Boot>, mut value: u64) {
    if value == 0 {
        system_table.stdout().write_char('0').unwrap();
        return;
    }
    
    let mut digits = [0u8; 20];
    let mut count = 0;
    
    while value > 0 {
        digits[count] = (value % 10) as u8 + b'0';
        value /= 10;
        count += 1;
    }
    
    for i in (0..count).rev() {
        system_table.stdout().write_char(digits[i] as char).unwrap();
    }
}
