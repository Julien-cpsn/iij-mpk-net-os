use crate::println;
use crate::utils::compat::DeviceWrapper;
use crate::utils::time::now;
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::udp;
use virtio_drivers::transport::pci::PciTransport;

const PORT: u16 = 5555;
const SOCKET_NUMBER: u8 = 64;

pub fn udp_rx_server(mut device: DeviceWrapper<PciTransport>, mut iface: Interface) {
    let mut sockets = SocketSet::new(vec![]);

    let mut udp_handles: Vec<SocketHandle> = Vec::new();

    println!("Creating {SOCKET_NUMBER} sockets...");

    for _ in 0..SOCKET_NUMBER {
        let udp_rx_buffer = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY, udp::PacketMetadata::EMPTY], vec![0; 65535]);
        let udp_tx_buffer = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY, udp::PacketMetadata::EMPTY], vec![0; 65535]);
        let udp_socket = udp::Socket::new(udp_rx_buffer, udp_tx_buffer);

        let udp_handle = sockets.add(udp_socket);
        udp_handles.push(udp_handle);
    }

    println!("Sockets created!");
    println!("===== UDP RX server started =====\n");

    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut device, &mut sockets);

        for (index, udp_handle) in udp_handles.iter_mut().enumerate() {
            let socket = sockets.get_mut::<udp::Socket>(*udp_handle);

            if !socket.is_open() {
                socket.bind(PORT).unwrap();
            }

            if socket.can_recv() {
                if let Ok((data, _)) = socket.recv() {
                    println!("s{index} | {:#?}", data);
                }
            }
        }
    }
}