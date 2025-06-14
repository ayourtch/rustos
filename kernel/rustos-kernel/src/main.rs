#![no_std]
#![no_main]

use core::panic::PanicInfo;
use spin::Mutex;
use uart_16550::SerialPort;

// Boot info structures as specified
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
pub struct MemoryDescriptor {
    pub ty: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
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

// System call numbers
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum SyscallNumber {
    Exit = 0,
    Read = 1,
    Write = 2,
    Open = 3,
    Close = 4,
    Fork = 5,
    Execve = 6,
    Wait = 7,
    GetPid = 8,
    Kill = 9,
    Mmap = 10,
    Munmap = 11,
    Brk = 12,
    Access = 13,
    Dup = 14,
    Dup2 = 15,
    Pipe = 16,
    Chdir = 17,
    Getcwd = 18,
    Mkdir = 19,
    Rmdir = 20,
    Unlink = 21,
    Rename = 22,
    Stat = 23,
    Fstat = 24,
    Lstat = 25,
    Chmod = 26,
    Chown = 27,
    Lchown = 28,
    Utime = 29,
    Time = 30,
    GetTimeOfDay = 31,
    Invalid = 0xFFFFFFFFFFFFFFFF,
}

impl From<u64> for SyscallNumber {
    fn from(value: u64) -> Self {
        match value {
            0 => SyscallNumber::Exit,
            1 => SyscallNumber::Read,
            2 => SyscallNumber::Write,
            3 => SyscallNumber::Open,
            4 => SyscallNumber::Close,
            5 => SyscallNumber::Fork,
            6 => SyscallNumber::Execve,
            7 => SyscallNumber::Wait,
            8 => SyscallNumber::GetPid,
            9 => SyscallNumber::Kill,
            10 => SyscallNumber::Mmap,
            11 => SyscallNumber::Munmap,
            12 => SyscallNumber::Brk,
            13 => SyscallNumber::Access,
            14 => SyscallNumber::Dup,
            15 => SyscallNumber::Dup2,
            16 => SyscallNumber::Pipe,
            17 => SyscallNumber::Chdir,
            18 => SyscallNumber::Getcwd,
            19 => SyscallNumber::Mkdir,
            20 => SyscallNumber::Rmdir,
            21 => SyscallNumber::Unlink,
            22 => SyscallNumber::Rename,
            23 => SyscallNumber::Stat,
            24 => SyscallNumber::Fstat,
            25 => SyscallNumber::Lstat,
            26 => SyscallNumber::Chmod,
            27 => SyscallNumber::Chown,
            28 => SyscallNumber::Lchown,
            29 => SyscallNumber::Utime,
            30 => SyscallNumber::Time,
            31 => SyscallNumber::GetTimeOfDay,
            _ => SyscallNumber::Invalid,
        }
    }
}

// Global serial port for logging
static SERIAL1: Mutex<Option<SerialPort>> = Mutex::new(None);

// Global framebuffer info for panic handler
static FRAMEBUFFER: Mutex<Option<FramebufferInfo>> = Mutex::new(None);

// IDT structures
#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn new() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64, selector: u16) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = ((handler >> 32) & 0xFFFFFFFF) as u32;
        self.selector = selector;
        self.type_attr = 0x8E; // Present, Ring 0, Interrupt Gate
        self.ist = 0;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

// IDT with 256 entries
static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

// Assembly interrupt handler stubs
core::arch::global_asm!(
    r#"
.section .text

.macro SAVE_REGS
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
.endm

.macro RESTORE_REGS
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
.endm

.global divide_error_handler_asm
.global debug_handler_asm
.global nmi_handler_asm
.global breakpoint_handler_asm
.global overflow_handler_asm
.global bound_range_exceeded_handler_asm
.global invalid_opcode_handler_asm
.global device_not_available_handler_asm
.global double_fault_handler_asm
.global invalid_tss_handler_asm
.global segment_not_present_handler_asm
.global stack_segment_fault_handler_asm
.global general_protection_fault_handler_asm
.global page_fault_handler_asm
.global x87_floating_point_handler_asm
.global alignment_check_handler_asm
.global machine_check_handler_asm
.global simd_floating_point_handler_asm
.global virtualization_handler_asm
.global control_protection_handler_asm
.global hypervisor_injection_handler_asm
.global vmm_communication_handler_asm
.global security_handler_asm
.global generic_interrupt_handler_asm
.global syscall_handler_asm

divide_error_handler_asm:
    SAVE_REGS
    call divide_error_handler
    RESTORE_REGS
    iretq

debug_handler_asm:
    SAVE_REGS
    call debug_handler
    RESTORE_REGS
    iretq

nmi_handler_asm:
    SAVE_REGS
    call nmi_handler
    RESTORE_REGS
    iretq

breakpoint_handler_asm:
    SAVE_REGS
    call breakpoint_handler
    RESTORE_REGS
    iretq

overflow_handler_asm:
    SAVE_REGS
    call overflow_handler
    RESTORE_REGS
    iretq

bound_range_exceeded_handler_asm:
    SAVE_REGS
    call bound_range_exceeded_handler
    RESTORE_REGS
    iretq

invalid_opcode_handler_asm:
    SAVE_REGS
    call invalid_opcode_handler
    RESTORE_REGS
    iretq

device_not_available_handler_asm:
    SAVE_REGS
    call device_not_available_handler
    RESTORE_REGS
    iretq

double_fault_handler_asm:
    SAVE_REGS
    call double_fault_handler
1:
    hlt
    jmp 1b

invalid_tss_handler_asm:
    SAVE_REGS
    pop rdi
    call invalid_tss_handler
    RESTORE_REGS
    iretq

segment_not_present_handler_asm:
    SAVE_REGS
    pop rdi
    call segment_not_present_handler
    RESTORE_REGS
    iretq

stack_segment_fault_handler_asm:
    SAVE_REGS
    pop rdi
    call stack_segment_fault_handler
    RESTORE_REGS
    iretq

general_protection_fault_handler_asm:
    SAVE_REGS
    pop rdi
    call general_protection_fault_handler
    RESTORE_REGS
    iretq

page_fault_handler_asm:
    SAVE_REGS
    pop rdi
    call page_fault_handler
    RESTORE_REGS
    iretq

x87_floating_point_handler_asm:
    SAVE_REGS
    call x87_floating_point_handler
    RESTORE_REGS
    iretq

alignment_check_handler_asm:
    SAVE_REGS
    pop rdi
    call alignment_check_handler
    RESTORE_REGS
    iretq

machine_check_handler_asm:
    SAVE_REGS
    call machine_check_handler
1:
    hlt
    jmp 1b

simd_floating_point_handler_asm:
    SAVE_REGS
    call simd_floating_point_handler
    RESTORE_REGS
    iretq

virtualization_handler_asm:
    SAVE_REGS
    call virtualization_handler
    RESTORE_REGS
    iretq

control_protection_handler_asm:
    SAVE_REGS
    pop rdi
    call control_protection_handler
    RESTORE_REGS
    iretq

hypervisor_injection_handler_asm:
    SAVE_REGS
    call hypervisor_injection_handler
    RESTORE_REGS
    iretq

vmm_communication_handler_asm:
    SAVE_REGS
    pop rdi
    call vmm_communication_handler
    RESTORE_REGS
    iretq

security_handler_asm:
    SAVE_REGS
    pop rdi
    call security_handler
    RESTORE_REGS
    iretq

generic_interrupt_handler_asm:
    SAVE_REGS
    call generic_interrupt_handler
    RESTORE_REGS
    iretq

syscall_handler_asm:
    SAVE_REGS
    call syscall_handler
    RESTORE_REGS
    iretq
"#
);

// Exception handler function declarations
extern "C" {
    fn divide_error_handler_asm();
    fn debug_handler_asm();
    fn nmi_handler_asm();
    fn breakpoint_handler_asm();
    fn overflow_handler_asm();
    fn bound_range_exceeded_handler_asm();
    fn invalid_opcode_handler_asm();
    fn device_not_available_handler_asm();
    fn double_fault_handler_asm();
    fn invalid_tss_handler_asm();
    fn segment_not_present_handler_asm();
    fn stack_segment_fault_handler_asm();
    fn general_protection_fault_handler_asm();
    fn page_fault_handler_asm();
    fn x87_floating_point_handler_asm();
    fn alignment_check_handler_asm();
    fn machine_check_handler_asm();
    fn simd_floating_point_handler_asm();
    fn virtualization_handler_asm();
    fn control_protection_handler_asm();
    fn hypervisor_injection_handler_asm();
    fn vmm_communication_handler_asm();
    fn security_handler_asm();
    fn generic_interrupt_handler_asm();
    fn syscall_handler_asm();
}

// Exception handlers (called from assembly)
#[no_mangle]
extern "C" fn divide_error_handler() {
    log_error("EXCEPTION: Divide Error");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn debug_handler() {
    log_error("EXCEPTION: Debug");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn nmi_handler() {
    log_error("EXCEPTION: Non-Maskable Interrupt");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn breakpoint_handler() {
    log_info("EXCEPTION: Breakpoint");
}

#[no_mangle]
extern "C" fn overflow_handler() {
    log_error("EXCEPTION: Overflow");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn bound_range_exceeded_handler() {
    log_error("EXCEPTION: Bound Range Exceeded");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn invalid_opcode_handler() {
    log_error("EXCEPTION: Invalid Opcode");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn device_not_available_handler() {
    log_error("EXCEPTION: Device Not Available");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn double_fault_handler() -> ! {
    log_error("EXCEPTION: Double Fault");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn invalid_tss_handler(_error_code: u64) {
    log_error("EXCEPTION: Invalid TSS");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn segment_not_present_handler(_error_code: u64) {
    log_error("EXCEPTION: Segment Not Present");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn stack_segment_fault_handler(_error_code: u64) {
    log_error("EXCEPTION: Stack Segment Fault");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn general_protection_fault_handler(_error_code: u64) {
    log_error("EXCEPTION: General Protection Fault");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn page_fault_handler(_error_code: u64) {
    log_error("EXCEPTION: Page Fault");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn x87_floating_point_handler() {
    log_error("EXCEPTION: x87 Floating Point");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn alignment_check_handler(_error_code: u64) {
    log_error("EXCEPTION: Alignment Check");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn machine_check_handler() -> ! {
    log_error("EXCEPTION: Machine Check");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn simd_floating_point_handler() {
    log_error("EXCEPTION: SIMD Floating Point");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn virtualization_handler() {
    log_error("EXCEPTION: Virtualization");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn control_protection_handler(_error_code: u64) {
    log_error("EXCEPTION: Control Protection");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn hypervisor_injection_handler() {
    log_error("EXCEPTION: Hypervisor Injection");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn vmm_communication_handler(_error_code: u64) {
    log_error("EXCEPTION: VMM Communication");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn security_handler(_error_code: u64) {
    log_error("EXCEPTION: Security");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn generic_interrupt_handler() {
    log_info("Generic interrupt received");
}

#[no_mangle]
extern "C" fn syscall_handler() {
    log_info("System call received");
}

#[repr(C)]
struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

// Initialize IDT
fn init_idt() {
    unsafe {
        // Set exception handlers (0-31)
        IDT[0].set_handler(divide_error_handler_asm as u64, 0x08);
        IDT[1].set_handler(debug_handler_asm as u64, 0x08);
        IDT[2].set_handler(nmi_handler_asm as u64, 0x08);
        IDT[3].set_handler(breakpoint_handler_asm as u64, 0x08);
        IDT[4].set_handler(overflow_handler_asm as u64, 0x08);
        IDT[5].set_handler(bound_range_exceeded_handler_asm as u64, 0x08);
        IDT[6].set_handler(invalid_opcode_handler_asm as u64, 0x08);
        IDT[7].set_handler(device_not_available_handler_asm as u64, 0x08);
        IDT[8].set_handler(double_fault_handler_asm as u64, 0x08);
        IDT[9].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Coprocessor Segment Overrun (legacy)
        IDT[10].set_handler(invalid_tss_handler_asm as u64, 0x08);
        IDT[11].set_handler(segment_not_present_handler_asm as u64, 0x08);
        IDT[12].set_handler(stack_segment_fault_handler_asm as u64, 0x08);
        IDT[13].set_handler(general_protection_fault_handler_asm as u64, 0x08);
        IDT[14].set_handler(page_fault_handler_asm as u64, 0x08);
        IDT[15].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[16].set_handler(x87_floating_point_handler_asm as u64, 0x08);
        IDT[17].set_handler(alignment_check_handler_asm as u64, 0x08);
        IDT[18].set_handler(machine_check_handler_asm as u64, 0x08);
        IDT[19].set_handler(simd_floating_point_handler_asm as u64, 0x08);
        IDT[20].set_handler(virtualization_handler_asm as u64, 0x08);
        IDT[21].set_handler(control_protection_handler_asm as u64, 0x08);
        IDT[22].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[23].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[24].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[25].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[26].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[27].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved
        IDT[28].set_handler(hypervisor_injection_handler_asm as u64, 0x08);
        IDT[29].set_handler(vmm_communication_handler_asm as u64, 0x08);
        IDT[30].set_handler(security_handler_asm as u64, 0x08);
        IDT[31].set_handler(generic_interrupt_handler_asm as u64, 0x08); // Reserved

        // Set interrupt handlers (32-255)
        for i in 32..256 {
            if i == 0x80 {
                // System call interrupt
                IDT[i].set_handler(syscall_handler_asm as u64, 0x08);
            } else {
                IDT[i].set_handler(generic_interrupt_handler_asm as u64, 0x08);
            }
        }

        // Load IDT
        let idt_descriptor = IdtDescriptor {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: IDT.as_ptr() as u64,
        };

        core::arch::asm!("lidt [{}]", in(reg) &idt_descriptor, options(readonly, nostack, preserves_flags));
    }
}

// Logging functions
fn log_info(msg: &str) {
    if let Some(mut serial) = SERIAL1.lock().as_mut() {
        for byte in msg.bytes() {
            unsafe { serial.send(byte); }
        }
        unsafe { serial.send(b'\n'); }
    }
}

fn log_error(msg: &str) {
    if let Some(mut serial) = SERIAL1.lock().as_mut() {
        for byte in b"ERROR: ".iter() {
            unsafe { serial.send(*byte); }
        }
        for byte in msg.bytes() {
            unsafe { serial.send(byte); }
        }
        unsafe { serial.send(b'\n'); }
    }
}

// Framebuffer utilities for panic handler
fn write_to_framebuffer(msg: &str, color: u32) {
    if let Some(fb) = FRAMEBUFFER.lock().as_ref() {
        let fb_ptr = fb.addr as *mut u32;
        let bytes_per_pixel = fb.bpp / 8;
        
        if bytes_per_pixel == 4 {
            unsafe {
                // Simple text rendering - just fill some pixels with color to indicate panic
                for i in 0..1000 {
                    if (i as u64) < (fb.width as u64 * fb.height as u64) {
                        *fb_ptr.add(i) = color;
                    }
                }
            }
        }
    }
}

// Validate boot info structure
fn validate_boot_info(boot_info: &BootInfo) -> bool {
    // Check if memory map has valid entries
    if boot_info.memory_map.entry_count == 0 || boot_info.memory_map.entries.is_null() {
        return false;
    }
    
    // Check if framebuffer info is reasonable
    if boot_info.framebuffer.width == 0 || boot_info.framebuffer.height == 0 {
        return false;
    }
    
    if boot_info.framebuffer.bpp != 32 && boot_info.framebuffer.bpp != 24 && boot_info.framebuffer.bpp != 16 {
        return false;
    }
    
    true
}

// Kernel entry point
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize serial port for logging
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    *SERIAL1.lock() = Some(serial_port);
    
    log_info("RustOS Kernel Starting...");
    
    // Validate boot info
    if !validate_boot_info(boot_info) {
        log_error("Invalid boot info structure");
        panic!("Boot info validation failed");
    }
    
    log_info("Boot info validated successfully");
    
    // Store framebuffer info for panic handler
    *FRAMEBUFFER.lock() = Some(boot_info.framebuffer.clone());
    
    // Log boot info details
    log_info("Memory map entries:");
    unsafe {
        let entries = core::slice::from_raw_parts(
            boot_info.memory_map.entries,
            boot_info.memory_map.entry_count
        );
        
        for (i, entry) in entries.iter().enumerate() {
            if i < 10 { // Limit output for readability
                log_info("  Entry found in memory map");
            }
        }
    }
    
    log_info("Framebuffer info:");
    log_info("  Resolution and format validated");
    
    if let Some(rsdp_addr) = boot_info.rsdp_addr {
        log_info("ACPI RSDP found");
    } else {
        log_info("No ACPI RSDP provided");
    }
    
    // Initialize IDT
    log_info("Initializing IDT...");
    init_idt();
    log_info("IDT initialized successfully");
    
    // Enable interrupts
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
    log_info("Interrupts enabled");
    
    log_info("Kernel initialization complete");
    log_info("Kernel ready for system calls");
    
    // Main kernel loop
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
    
    // Write to serial
    if let Some(mut serial) = SERIAL1.lock().as_mut() {
        for byte in b"\n\nKERNEL PANIC!\n".iter() {
            unsafe { serial.send(*byte); }
        }
        
        if let Some(location) = info.location() {
            let loc_str = "Location: ";
            for byte in loc_str.bytes() {
                unsafe { serial.send(byte); }
            }
            for byte in location.file().bytes() {
                unsafe { serial.send(byte); }
            }
            unsafe { serial.send(b':'); }
            
            // Convert line number to string and send
            let line = location.line();
            let mut buffer = [0u8; 10];
            let mut len = 0;
            let mut temp = line;
            
            if temp == 0 {
                buffer[0] = b'0';
                len = 1;
            } else {
                while temp > 0 && len < buffer.len() {
                    buffer[len] = b'0' + (temp % 10) as u8;
                    temp /= 10;
                    len += 1;
                }
                // Reverse the digits
                for i in 0..len/2 {
                    buffer.swap(i, len - 1 - i);
                }
            }
            
            for i in 0..len {
                unsafe { serial.send(buffer[i]); }
            }
            unsafe { serial.send(b'\n'); }
        }
        
        if let Some(msg) = info.payload().downcast_ref::<&str>() {
            for byte in b"Message: ".iter() {
                unsafe { serial.send(*byte); }
            }
            for byte in msg.bytes() {
                unsafe { serial.send(byte); }
            }
            unsafe { serial.send(b'\n'); }
        }
    }
    
    // Write to framebuffer (red color to indicate panic)
    write_to_framebuffer("KERNEL PANIC", 0xFF0000);
    
    // Halt forever
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
