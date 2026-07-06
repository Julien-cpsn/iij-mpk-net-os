use crate::drivers::mmio::MMCONFIG_PHYS_BASE;
use acpi::{AcpiHandler, AcpiTables, PciConfigRegions, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::{PhysAddr, VirtAddr};


#[derive(Clone)]
pub struct AcpiHandlerImpl {
    physical_memory_offset: VirtAddr,
}

impl AcpiHandlerImpl {
    pub fn new(physical_memory_offset: VirtAddr) -> Self {
        Self {
            physical_memory_offset,
        }
    }
}

impl AcpiHandler for AcpiHandlerImpl {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let phys_addr = PhysAddr::new(physical_address as u64);
        let virt_addr = self.physical_memory_offset + phys_addr.as_u64();

        unsafe {
            PhysicalMapping::new(
                physical_address,
                NonNull::new(virt_addr.as_mut_ptr()).expect("Failed to get virtual address"),
                size,
                size,
                self.clone(),
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // No unmapping necessary as we didn't create any new mappings
    }
}

pub fn init_acpi(rsdp: u64, physical_memory_offset: VirtAddr) {
    let handler = AcpiHandlerImpl::new(physical_memory_offset);
    let acpi = unsafe { AcpiTables::from_rsdp(handler, rsdp as usize) }.expect("ACPI tables failed");


    let mcfg = PciConfigRegions::new(&acpi).expect("No PCI MCFG table found");

    let region = mcfg
        .iter()
        .next()
        .expect("MCFG contains no regions");

    MMCONFIG_PHYS_BASE.call_once(|| PhysAddr::new(region.physical_address as u64));
}