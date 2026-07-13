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
use crate::drivers::pic::init_pic;
use crate::memory::heap::init_heap;
use crate::memory::tables::init_memory_mapping;
use crate::utils::log::init_logger;
use crate::utils::qemu::{exit_qemu, QemuExitCode};
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::VirtAddr;
use crate::utils::time::init_time;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kprintln!("\n\nHello, world!");
    kprintln!("Entered kernel with boot info: {boot_info:?}");
    kprintln!();

    init_kernel(boot_info);

    cpu::protection::user_mode::init_user_mode(boot_info);
    //drivers::pci::enumerate_pci();
    //apps::benchmark::benchmark();

    exit_qemu(QemuExitCode::Success);
}

pub fn init_kernel(boot_info: &mut BootInfo) {
    kprintln!("Initializing kernel...");

    kprintln!("TSC");
    init_time();
    
    kprintln!("GDT");
    init_gdt();

    kprintln!("IDT");
    init_idt();

    kprintln!("Memory mapper");
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset.take().expect("No physical memory"));
    init_memory_mapping(physical_memory_offset);

    kprintln!("Heap");
    init_heap(&boot_info.memory_regions);

    kprintln!("PKU & PKS");
    init_pk();

    kprintln!("ACPI");
    let rsdp = boot_info.rsdp_addr.take().expect("Failed to get RSDP address");
    init_acpi(rsdp, physical_memory_offset);

    kprintln!("PIC");
    init_pic();

    kprintln!("Interrupts");
    //x86_64::instructions::interrupts::enable();

    kprintln!("Init logger");
    init_logger();

    kprintln!("Kernel initialized!\n");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kprintln!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}