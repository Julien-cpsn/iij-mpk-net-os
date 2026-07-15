use crate::drivers::mmio::{MMCONFIG_PHYS_BASE, MMCONFIG_SIZE};
use crate::user::apps::devices::compat::DeviceWrapper;
use crate::user::apps::devices::dma::HalImpl;
use crate::user::api::user_syscalls::allocate;
use crate::user::apps::devices::nic::init_interface;
use crate::{error, info, trace};
use smoltcp::iface::Interface;
use virtio_drivers::transport::pci::bus::{Cam, Command, MmioCam, PciRoot};
use virtio_drivers::transport::pci::{virtio_device_type, PciTransport};
use virtio_drivers::transport::DeviceType;

const TARGET: &str = "PCI";

pub fn pci_scan() -> (DeviceWrapper<PciTransport>, Interface) {
    info!("Starting scan...");

    trace!("\tMapping MMIO space...");
    let ecam = allocate(MMCONFIG_PHYS_BASE.get().unwrap(), MMCONFIG_SIZE);
    trace!("\tMMIO space mapped");

    let cam = unsafe { MmioCam::new(ecam.as_mut_ptr(), Cam::Ecam) };
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

    if let Some(virtio_transport) = virtio_net_pci_transport {
        init_interface(virtio_transport)
    }
    else {
        error!("No virtio-net-pci card found");
        panic!();
    }
}