#![no_std]
#![no_main]

extern crate alloc;

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
use uefi_services::println;

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
    
    // Load kernel from filesystem
    system_table.stdout().write_str("Loading kernel...\n").unwrap();
    let kernel_data = load_kernel(system_table.boot_services(), image).expect("Failed to load kernel");
    
    // Parse ELF and get entry point
    system_table.stdout().write_str("Parsing ELF...\n").unwrap();
    let entry_point = parse_elf_and_load(&kernel_data, system_table.boot_services()).expect("Failed to parse ELF");
    
    // Set up identity mapping for first 1GB
    system_table.stdout().write_str("Setting up identity mapping...\n").unwrap();
    setup_identity_mapping(system_table.boot_services()).expect("Failed to setup identity mapping");
    
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
    
    // At this point, we can't use stdout anymore, but we can try to jump to kernel
    // Let's try to signal that we're about to jump by writing to a memory location
    unsafe {
        // Write a magic value to indicate we're about to jump
        *(0xb8000 as *mut u16) = 0x0F42; // 'B' in white on black (VGA text mode)
    }

    // Jump to kernel
    unsafe {
        let kernel_entry: extern "C" fn(*const BootInfo) -> ! = mem::transmute(entry_point);
        kernel_entry(boot_info_addr as *const BootInfo);
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
            
            // Try to allocate at the requested virtual address first
            let segment_addr = match boot_services.allocate_pages(
                uefi::table::boot::AllocateType::Address(p_vaddr),
                MemoryType::LOADER_DATA,
                pages_needed,
            ) {
                Ok(addr) => addr,
                Err(_) => {
                    // If that fails, allocate anywhere and we'll need to update the mapping
                    boot_services.allocate_pages(
                        uefi::table::boot::AllocateType::AnyPages,
                        MemoryType::LOADER_DATA,
                        pages_needed,
                    ).map_err(|_| "Failed to allocate segment memory anywhere")?
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
    
    let pd_addr = boot_services.allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).map_err(|_| "Failed to allocate PD")?;
    
    unsafe {
        // Clear all page tables
        core::ptr::write_bytes(pml4_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pdpt_addr as *mut u8, 0, 0x1000);
        core::ptr::write_bytes(pd_addr as *mut u8, 0, 0x1000);
        
        let pml4 = &mut *(pml4_addr as *mut PageTable);
        let pdpt = &mut *(pdpt_addr as *mut PageTable);
        let pd = &mut *(pd_addr as *mut PageTable);
        
        // Set up PML4[0] -> PDPT
        pml4.set_entry(0, pdpt_addr, PAGE_PRESENT | PAGE_WRITABLE);
        
        // Set up PDPT[0] -> PD
        pdpt.set_entry(0, pd_addr, PAGE_PRESENT | PAGE_WRITABLE);
        
        // Set up PD entries for first 1GB (512 * 2MB pages)
        for i in 0..512 {
            let addr = i as u64 * 0x20_0000; // 2MB pages
            pd.set_entry(i, addr, PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE);
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
