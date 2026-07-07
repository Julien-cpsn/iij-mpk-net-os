use alloc::string::ToString;
use crate::println;
use crate::drivers::dma::HalImpl;
use crate::utils::compat::DeviceWrapper;
use crate::utils::time::now;
use core::str::FromStr;
use smoltcp::iface::{Config, Interface};
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use spin::LazyLock;
use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::pci::PciTransport;

const IP: LazyLock<IpAddress> = LazyLock::new(|| IpAddress::from_str("192.168.179.2").unwrap());
const GATEWAY: LazyLock<Ipv4Address> = LazyLock::new(|| Ipv4Address::from_str("192.168.179.1").unwrap());
const NET_BUFFER_LEN: usize = 2048;
pub const NET_QUEUE_SIZE: usize = 2;

pub fn init_interface(transport: PciTransport) {
    println!("Initializing virtio-net-pci interface...");

    let dev = VirtIONet::<HalImpl, PciTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN).expect("failed to create net driver");
    let mac = dev.mac_address();

    println!("\tMAC address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    let mut device = DeviceWrapper::new(dev);

    // Create interface
    let mut config = Config::new(HardwareAddress::Ethernet(device.mac_address()));
    config.random_seed = 0x2333;

    let mut iface = Interface::new(config, &mut device, now());

    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(*IP, 24))
            .unwrap();
    });

    println!("\tIP address: {}", IP.to_string());

    iface
        .routes_mut()
        .add_default_ipv4_route(*GATEWAY)
        .unwrap();

    println!("\tGateway: {}", GATEWAY.to_string());
    println!("Virtio-net-pci interface initialized!\n");

    //crate::apps::http_server::http_server(device, iface);
    crate::apps::udp_server::udp_rx_server(device, iface);
}