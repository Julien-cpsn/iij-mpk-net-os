use crate::apps::user::user_entry;
use crate::cpu::interrupt::DOUBLE_FAULT_IST_INDEX;
use crate::memory::tables::addr_frame_set_or_flags;
use crate::println;
use core::arch::asm;
use core::ops::Add;
use core::ptr::addr_of;
use spin::{LazyLock, RwLock};
use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::tlb;
use x86_64::registers::segmentation::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::paging::PageTableFlags;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

/// Task State Segment.
/// Structure on x86-based computers which holds information about a task
pub static TSS: LazyLock<TaskStateSegment> = LazyLock::new(|| {
    let mut tss = TaskStateSegment::new();

    const KERNEL_STACK_SIZE: usize = 4096 * 5;
    static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

    let stack_start = VirtAddr::from_ptr(addr_of!(KERNEL_STACK));
    let stack_end = stack_start + KERNEL_STACK_SIZE as u64;

    // Used when entering the kernel from ring 3
    tss.privilege_stack_table[0] = stack_end;

    // Used only for the double-fault IST
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

    tss
});

/// Global Descriptor Table.
/// Construct used by the x86 processor to configure segmented virtual memory
pub static GDT: LazyLock<(GlobalDescriptorTable, RwLock<Selectors>)> = LazyLock::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

    let selectors = Selectors {
        kernel_code_selector,
        kernel_data_selector,
        user_code_selector,
        user_data_selector,
        tss_selector
    };

    (gdt, RwLock::new(selectors))
});

pub struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init_gdt() {
    GDT.0.load();
    let selectors = GDT.1.read();
    unsafe {
        CS::set_reg(selectors.kernel_code_selector);
        SS::set_reg(selectors.kernel_data_selector);
        DS::set_reg(selectors.kernel_data_selector);
        ES::set_reg(selectors.kernel_data_selector);
        FS::set_reg(selectors.kernel_data_selector);
        GS::set_reg(selectors.kernel_data_selector);
        load_tss(selectors.tss_selector);
    }
}

#[allow(unused)]
pub fn init_user_mode() {
    println!("Entering user mode...");

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

    {
        addr_frame_set_or_flags(user_entry, PageTableFlags::USER_ACCESSIBLE, true);
        tlb::flush(user_entry);
    }

    {
        addr_frame_set_or_flags(user_stack - 1, PageTableFlags::USER_ACCESSIBLE, true);
        tlb::flush(user_stack - 1);
    }

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