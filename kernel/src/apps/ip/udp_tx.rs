use crate::apps::interface::GATEWAY;
use crate::apps::ip::utils::{create_sockets, SocketType};
use crate::utils::compat::DeviceWrapper;
use crate::utils::time::now;
use alloc::vec;
use alloc::vec::Vec;
use goolog::info;
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::udp;
use smoltcp::wire::{IpAddress, IpEndpoint};
use virtio_drivers::transport::pci::PciTransport;


const GOOLOG_TARGET: &str = "UDP TX";

const SOCKET_NUMBER: u16 = 256;
pub const UDP_PACKET_METADATA_BUFFER_SIZE: usize = 2;
pub const UDP_PACKET_PAYLOAD_BUFFER_SIZE: usize = 65_535;
const CONTENT: &[u8] = b"Hello world";

pub fn udp_tx_server(mut device: DeviceWrapper<PciTransport>, mut iface: Interface) {
    let mut sockets = SocketSet::new(vec![]);
    let mut udp_handles: Vec<SocketHandle> = Vec::new();

    create_sockets(&mut sockets, &mut udp_handles, SocketType::Udp, SOCKET_NUMBER);

    info!("===== UDP TX server started =====");

    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut device, &mut sockets);

        for udp_handle in udp_handles.iter_mut() {
            let socket = sockets.get_mut::<udp::Socket>(*udp_handle);

            if !socket.is_open() {
                socket.bind(9).unwrap();
            }

            if socket.can_send() {
                let _ = socket.send_slice(CONTENT, IpEndpoint::new(IpAddress::Ipv4(*GATEWAY), 9));
            }
        }
    }
}