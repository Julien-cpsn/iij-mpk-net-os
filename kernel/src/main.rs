#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![allow(internal_features)]

extern crate alloc;

mod utils;
mod drivers;
mod apps;
mod memory;
mod cpu;

use crate::cpu::gdt::init_gdt;
use crate::cpu::idt::init_idt;
use crate::cpu::protection::pk::init_pk;
use crate::drivers::acpi::init_acpi;
use crate::drivers::pci::enumerate_pci;
use crate::drivers::pic::init_pic;
use crate::memory::allocator::init_heap;
use crate::memory::tables::init_memory_mapping;
use crate::utils::qemu::{exit_qemu, QemuExitCode};
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("\n\nHello, world!");
    println!("Entered kernel with boot info: {boot_info:?}");
    println!();

    init_kernel(boot_info);

    //init_user_mode();
    enumerate_pci();

    exit_qemu(QemuExitCode::Success);
}

pub fn init_kernel(boot_info: &mut BootInfo) {
    println!("Initializing kernel...");

    println!("GDT");
    init_gdt();

    println!("IDT");
    init_idt();

    println!("Memory mapper");
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset.take().expect("No physical memory"));
    init_memory_mapping(physical_memory_offset);

    println!("Heap");
    init_heap(&boot_info.memory_regions);

    println!("PKU & PKS");
    init_pk();

    println!("ACPI");
    let rsdp = boot_info.rsdp_addr.take().expect("Failed to get RSDP address");
    init_acpi(rsdp, physical_memory_offset);

    println!("PIC");
    init_pic();

    println!("Interrupts");
    //x86_64::instructions::interrupts::enable();

    println!("Kernel initialized!\n");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}