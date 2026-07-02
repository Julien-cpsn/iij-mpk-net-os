use crate::drivers::dma::HalImpl;
use crate::drivers::mmio::{MemoryMapper, MMCONFIG_BASE, MMCONFIG_SIZE};
use crate::println;
use accessor::Mapper;
use virtio_drivers::transport::pci::bus::{Cam, Command, MmioCam, PciRoot};
use virtio_drivers::transport::pci::{virtio_device_type, PciTransport};
use virtio_drivers::transport::DeviceType;
use crate::drivers::virtio_net::init_virtio_net;

pub fn enumerate_pci() {
    println!("Starting PCI scan");

    let mmconfig = MMCONFIG_BASE.get().expect("ACPI not initialized");

    let mut mapper = MemoryMapper;
    let ecam = unsafe { mapper.map(mmconfig.as_u64() as usize, MMCONFIG_SIZE) };

    let cam = unsafe { MmioCam::new(ecam.get() as *mut u8, Cam::Ecam) };
    let mut pci_root = PciRoot::new(cam);

    let mut virtio_net_pci_transport = None;

    for (df, info) in pci_root.enumerate_bus(0) {
        println!("Found Vendor: {:4>0X}, Device: {:4>0X}", info.vendor_id, info.device_id);

        let Some(virtio_type) = virtio_device_type(&info) else {
            continue;
        };

        pci_root.set_command(df, Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER);

        if !matches!(virtio_type, DeviceType::Network) {
            continue;
        }

        // If not virtio-net-pci
        if info.vendor_id != 0x1AF4 && info.device_id != 0x1000 {
            continue;
        }

        println!("\t^ Found virtio-net-pci NIC");
        virtio_net_pci_transport = Some(PciTransport::new::<HalImpl, _>(&mut pci_root, df).unwrap());
    }
    println!("PCI scan complete\n");

    if let Some(virtio_transport) = virtio_net_pci_transport {
        init_virtio_net(virtio_transport);
    }
    else {
        panic!("No virtio-net-pci card found");
    }
}