use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::Transport;
use crate::apps::tcp_echo_server::tcp_echo_server;
use crate::drivers::dma::HalImpl;
use crate::println;

const NET_BUFFER_LEN: usize = 2048;
pub const NET_QUEUE_SIZE: usize = 16;

pub fn init_virtio_net<T: Transport>(transport: T) {
    println!("Initializing virtio-net-pci...");

    let net = VirtIONet::<HalImpl, T, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN).expect("failed to create net driver");
    let mac = net.mac_address();

    println!("MAC address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    tcp_echo_server(net);
}