use alloc::vec;
use alloc::vec::Vec;
use crate::user::apps::ip::udp_tx::{UDP_PACKET_METADATA_BUFFER_SIZE, UDP_PACKET_PAYLOAD_BUFFER_SIZE};
use crate::user::apps::ip::udp_rx::{RAW_PACKET_METADATA_BUFFER_SIZE, RAW_PACKET_PAYLOAD_BUFFER_SIZE};
use crate::info;
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::socket::{raw, tcp, udp};


const TARGET: &str = "IP UTILS";

pub enum SocketType {
    Raw,
    Tcp,
    Udp
}

pub fn create_sockets(sockets: &mut SocketSet, handles: &mut Vec<SocketHandle>, socket_type: SocketType, number: u16) {
    info!("Creating {number} sockets...");

    for _ in 0..number {
        let handle = match socket_type {
            SocketType::Raw => {
                let socket = create_raw_socket();
                sockets.add(socket)
            }
            SocketType::Tcp => {
                let socket = create_tcp_socket();
                sockets.add(socket)
            }
            SocketType::Udp => {
                let socket = create_udp_socket();
                sockets.add(socket)
            }
        };

        handles.push(handle);
    }

    info!("Sockets created!");
}

fn create_raw_socket<'a>() -> raw::Socket<'a> {
    let raw_rx_buffer = raw::PacketBuffer::new(
        vec![raw::PacketMetadata::EMPTY; RAW_PACKET_METADATA_BUFFER_SIZE],
        vec![0u8; RAW_PACKET_PAYLOAD_BUFFER_SIZE]
    );
    let raw_tx_buffer = raw::PacketBuffer::new(
        vec![raw::PacketMetadata::EMPTY; RAW_PACKET_METADATA_BUFFER_SIZE],
        vec![0u8; RAW_PACKET_PAYLOAD_BUFFER_SIZE]
    );
    let raw_socket = raw::Socket::new(None, None, raw_rx_buffer, raw_tx_buffer);

    raw_socket
}

fn create_tcp_socket<'a>() -> tcp::Socket<'a> {
    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);

    tcp_socket
}

fn create_udp_socket<'a>() -> udp::Socket<'a> {
    let udp_rx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; UDP_PACKET_METADATA_BUFFER_SIZE],
        vec![0u8; UDP_PACKET_PAYLOAD_BUFFER_SIZE]
    );
    let udp_tx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; UDP_PACKET_METADATA_BUFFER_SIZE],
        vec![0u8; UDP_PACKET_PAYLOAD_BUFFER_SIZE]
    );
    let udp_socket = udp::Socket::new(udp_rx_buffer, udp_tx_buffer);

    udp_socket
}