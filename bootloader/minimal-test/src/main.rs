#![no_std]
#![no_main]

use uefi::prelude::*;
use core::fmt::Write;

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.stdout().write_str("Hello from minimal UEFI app!\n").unwrap();
    
    // Wait a bit so we can see the message
    system_table.boot_services().stall(3_000_000); // 3 seconds
    
    Status::SUCCESS
}
