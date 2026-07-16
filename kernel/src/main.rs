#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use kernel::cpu::gdt::init_gdt;
use kernel::cpu::idt::init_idt;
use kernel::cpu::protection::pk::init_pk;
use kernel::cpu::protection::user_mode::{init_user_mode, is_in_usermode};
use kernel::drivers::acpi::init_acpi;
use kernel::drivers::pic::init_pic;
use kernel::kprintln;
use kernel::memory::heap::init_heap;
use kernel::memory::tables::init_memory_mapping;
use kernel::utils::log::init_logger;
use kernel::utils::qemu::{exit_qemu, QemuExitCode};
use kernel::utils::time::init_time;
use x86_64::VirtAddr;

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
    init_user_mode(boot_info);

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

    kprintln!("Logger");
    init_logger();

    kprintln!("Kernel initialized!\n");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let frames = kernel::cpu::stack_trace::get_stacktrace();

    if is_in_usermode() {
        let text = alloc::format!("PANIC: {info}\n");
        kernel::user::api::user_syscalls::print(text.as_ptr(), text.len());
        let text = alloc::format!("STACK TRACE: {frames}\n");
        kernel::user::api::user_syscalls::print(text.as_ptr(), text.len());
        kernel::user::api::user_syscalls::exit(QemuExitCode::Failed);
        loop {}
    } else {
        kprintln!("PANIC: {info}");
        kprintln!("STACK TRACE: {frames}");
        exit_qemu(QemuExitCode::Failed);
    }
}