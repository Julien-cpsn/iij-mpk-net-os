use crate::cpu::gdt::GDT;
use crate::memory::heap::ALLOCATOR;
use crate::memory::tables::add_flags_to_frame;
use crate::user::main::user_entry;
use crate::kprintln;
use bootloader_api::BootInfo;
use core::arch::asm;
use core::ops::Add;
use core::ptr::addr_of;
use x86_64::instructions::tlb;
use x86_64::registers::segmentation::SegmentSelector;
use x86_64::structures::paging::{PageSize, PageTableFlags, Size4KiB};
use x86_64::{PrivilegeLevel, VirtAddr};


pub fn init_user_mode(boot_info: &BootInfo) {
    kprintln!("Entering user mode...");

    let (user_cs, user_ss) = {
        let selectors = GDT.1.read();

        let user_cs = SegmentSelector::new(
            selectors.user_code_selector.index(),
            PrivilegeLevel::Ring3,
        );

        let user_ss = SegmentSelector::new(
            selectors.user_data_selector.index(),
            PrivilegeLevel::Ring3,
        );

        (user_cs, user_ss)
    };


    const USER_STACK_SIZE: usize = 4096 * 4;
    static mut USER_STACK: [u8; USER_STACK_SIZE] = [0; USER_STACK_SIZE];

    let user_entry = VirtAddr::from_ptr(user_entry as *const ());
    let user_stack = VirtAddr::from_ptr(addr_of!(USER_STACK)).add(USER_STACK_SIZE as u64);

    // USER ENTRY
    {
        add_flags_to_frame(user_entry, PageTableFlags::USER_ACCESSIBLE, true);
        tlb::flush(user_entry);
    }

    // USER STACK
    {
        add_flags_to_frame(user_stack - 1, PageTableFlags::USER_ACCESSIBLE, true);
        tlb::flush(user_stack - 1);
    }

    // KERNEL
    {
        let start = boot_info.kernel_image_offset;
        let end = start + boot_info.kernel_len;

        let mut addr = start;

        while addr < end {
            let virt_addr = VirtAddr::new(addr);
            add_flags_to_frame(virt_addr, PageTableFlags::USER_ACCESSIBLE, true);
            tlb::flush(virt_addr);

            addr += Size4KiB::SIZE;
        }
    }


    // HEAP
    {
        let (start, end) = {
            let allocator = ALLOCATOR.lock();
            (allocator.bottom().addr() as u64, allocator.top().addr() as u64)
        };

        let mut addr = start;

        while addr < end {
            let virt_addr = VirtAddr::new(addr);
            add_flags_to_frame(virt_addr, PageTableFlags::USER_ACCESSIBLE, true);
            tlb::flush(virt_addr);

            addr += Size4KiB::SIZE;
        }
    }

    kprintln!("User mode entered");
    kprintln!("================================================================================");
    kprintln!();

    unsafe {
        asm!(
            "push {ss}",
            "push {rsp}",
            "pushfq",
            "push {cs}",
            "push {rip}",
            "iretq",
            ss = in(reg) user_ss.0 as u64,
            rsp = in(reg) user_stack.as_u64(),
            cs = in(reg) user_cs.0 as u64,
            rip = in(reg) user_entry.as_u64(),
            options(noreturn),
        );
    }
}