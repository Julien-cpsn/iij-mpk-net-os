use crate::apps::interface::init_interface;
use crate::drivers::dma::HalImpl;
use crate::drivers::mmio::{MemoryMapper, MMCONFIG_PHYS_BASE, MMCONFIG_SIZE};
use accessor::Mapper;
use goolog::{debug, error, info};
use virtio_drivers::transport::pci::bus::{Cam, Command, MmioCam, PciRoot};
use virtio_drivers::transport::pci::{virtio_device_type, PciTransport};
use virtio_drivers::transport::DeviceType;

const GOOLOG_TARGET: &str = "PCI";

#[allow(unused)]
pub fn enumerate_pci() {
    info!("Starting scan...");

    debug!("\tMapping MMIO space...");
    let mut mapper = MemoryMapper;
    let ecam = unsafe { mapper.map(MMCONFIG_PHYS_BASE.get().unwrap().as_u64() as usize, MMCONFIG_SIZE) };
    debug!("\tMMIO space mapped");
    debug!("");

    let cam = unsafe { MmioCam::new(ecam.get() as *mut u8, Cam::Ecam) };
    let mut pci_root = PciRoot::new(cam);

    let mut virtio_net_pci_transport = None;

    info!("\tDevices:");

    for (df, info) in pci_root.enumerate_bus(0) {
        info!("\t\t- Vendor: 0x{:4>0X}, Device: 0x{:4>0X}", info.vendor_id, info.device_id);

        let Some(virtio_type) = virtio_device_type(&info) else {
            continue;
        };

        if !matches!(virtio_type, DeviceType::Network) {
            continue;
        }

        // If not QEMU virtio-net-pci
        if info.vendor_id != 0x1AF4 && info.device_id != 0x1000 {
            continue;
        }

        pci_root.set_command(df, Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER);

        info!("\t\t\t^ Found virtio-net-pci NIC");
        virtio_net_pci_transport = Some(PciTransport::new::<HalImpl, _>(&mut pci_root, df).unwrap());
    }
    info!("PCI complete!");
    info!("");

    if let Some(virtio_transport) = virtio_net_pci_transport {
        init_interface(virtio_transport);
    }
    else {
        error!("No virtio-net-pci card found");
        panic!();
    }
}