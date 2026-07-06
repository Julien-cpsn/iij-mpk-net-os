#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod utils;
mod drivers;
mod apps;
mod memory;
mod cpu;

use crate::cpu::gdt::init_gdt;
use crate::cpu::idt::init_idt;
use crate::drivers::acpi::init_acpi;
use crate::drivers::pci::enumerate_pci;
use crate::drivers::pic::init_pic;
use crate::memory::allocator::init_heap;
use crate::memory::tables::{MAPPER, MEMORY_REGIONS};
use crate::utils::qemu::{exit_qemu, QemuExitCode};
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use spin::{Mutex, RwLock};
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

    init(boot_info);

    enumerate_pci();

    exit_qemu(QemuExitCode::Success);
}

pub fn init(boot_info: &'static mut BootInfo) {
    println!("Initializing kernel...");

    println!("GDT");
    init_gdt();

    println!("IDT");
    init_idt();

    println!("Heap");
    init_heap();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset.take().expect("No physical memory"));
    let memory_regions = boot_info.memory_regions.to_vec();

    println!("Memory mapper");
    println!("\tPhysical offset: {:#X}", physical_memory_offset);
    MAPPER.call_once(|| RwLock::new(memory::tables::init(physical_memory_offset)));

    println!("Memory regions:");
    MEMORY_REGIONS.call_once(|| Mutex::new(memory_regions));

    for r in MEMORY_REGIONS.get().unwrap().lock().iter() {
        println!("\t{:?}: {:#x}..{:#x} ({} bytes)", r.kind, r.start, r.end, r.end - r.start);
    }

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