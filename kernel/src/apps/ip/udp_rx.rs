use crate::apps::ip::utils::{create_sockets, SocketType};
use crate::println;
use crate::utils::compat::DeviceWrapper;
use crate::utils::time::now;
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::raw;
use virtio_drivers::transport::pci::PciTransport;

const SOCKET_NUMBER: u16 = 1;
pub const RAW_PACKET_METADATA_BUFFER_SIZE: usize = 2;
pub const RAW_PACKET_PAYLOAD_BUFFER_SIZE: usize = 65_535;

pub fn udp_rx_server(mut device: DeviceWrapper<PciTransport>, mut iface: Interface) {
    let mut sockets = SocketSet::new(vec![]);
    let mut udp_handles: Vec<SocketHandle> = Vec::new();
    
    create_sockets(&mut sockets, &mut udp_handles, SocketType::Raw, SOCKET_NUMBER);

    println!("===== UDP RX server started =====\n");

    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut device, &mut sockets);

        for (index, udp_handle) in udp_handles.iter_mut().enumerate() {
            let socket = sockets.get_mut::<raw::Socket>(*udp_handle);

            if socket.can_recv() {
                if let Ok(_) = socket.recv() {
                    println!("s{index} | packet");
                }
            }
        }
    }
}