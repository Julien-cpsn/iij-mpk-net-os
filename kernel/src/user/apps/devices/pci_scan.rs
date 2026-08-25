use crate::user::apps::devices::virtio_net::net::{VirtioNetPci, VirtioNetQueue};
use crate::user::apps::devices::virtio_net::pci::{Ecam, VIRTIO_NET_DEVICE_ID_MODERN, VIRTIO_NET_DEVICE_ID_TRANSITIONAL, VIRTIO_VENDOR_ID};
use crate::{error, info, trace};

const TARGET: &str = "PCI";

const NUM_QUEUE_PAIRS: u16 = 1;

pub fn pci_scan() -> VirtioNetQueue {
    info!("Starting scan...");

    trace!("\tMapping MMIO space...");
    let ecam = Ecam::new();
    trace!("\tMMIO space mapped");


    let mut virtio_net_pci_transport = None;

    info!("\tDevices:");

    for (bdf, vendor, device) in ecam.enumerate_bus0() {
        info!("\t\t- Vendor: 0x{:4>0X}, Device: 0x{:4>0X}", vendor, device);

        if vendor != VIRTIO_VENDOR_ID {
            continue;
        }
        if device != VIRTIO_NET_DEVICE_ID_TRANSITIONAL && device != VIRTIO_NET_DEVICE_ID_MODERN {
            continue;
        }

        info!("\t\t\t^ Found virtio-net-pci NIC");

        virtio_net_pci_transport = Some(bdf);
    }
    info!("PCI complete!");

    let Some(bdf) = virtio_net_pci_transport else {
        error!("No virtio-net-pci card found");
        panic!();
    };

    let mut dev = VirtioNetPci::init(&ecam, bdf, NUM_QUEUE_PAIRS);
    info!("Device initialized");
    let queue = dev.take_queue(0);
    queue
}