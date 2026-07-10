use crate::cpu::interrupt::DOUBLE_FAULT_IST_INDEX;
use core::ptr::addr_of;
use spin::{LazyLock, RwLock};
use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;


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
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
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