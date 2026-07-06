use alloc::borrow::ToOwned;
use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;
use byteorder::{ByteOrder, NetworkEndian};
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::Device;
use smoltcp::socket::{icmp, tcp};
use smoltcp::wire::{HardwareAddress, Icmpv4Packet, Icmpv4Repr, IpAddress, IpCidr, Ipv4Address};
use spin::LazyLock;
use virtio_drivers::transport::Transport;
use crate::apps::compat::{DeviceImpl, DeviceWrapper};
use crate::println;
use crate::utils::time::now;

const IP: LazyLock<IpAddress> = LazyLock::new(|| IpAddress::from_str("192.168.179.2").unwrap());
const GATEWAY: LazyLock<Ipv4Address> = LazyLock::new(|| Ipv4Address::from_str("192.168.179.1").unwrap());
const PORT: u16 = 5555;


pub fn tcp_echo_server<T: Transport>(dev: DeviceImpl<T>) {
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

    iface
        .routes_mut()
        .add_default_ipv4_route(*GATEWAY)
        .unwrap();

    let mut sockets = SocketSet::new(vec![]);

    println!("Replied to PING, entering TCP server...");

    // TCP

    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);

    let tcp_handle = sockets.add(tcp_socket);

    println!("Start the echo server...");
    let mut tcp_active = false;
    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut device, &mut sockets);

        // tcp:PORT: echo with reverse
        let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
        if !socket.is_open() {
            println!("Listening on port {}...", PORT);
            socket.listen(PORT).unwrap();
        }

        if socket.is_active() && !tcp_active {
            println!("tcp:{} connected", PORT);
        }
        else if !socket.is_active() && tcp_active {
            println!("tcp:{} disconnected", PORT);
        }

        tcp_active = socket.is_active();

        if socket.may_recv() {
            if let Ok(data) = socket.recv(|buff| recv(buff)) {
                if socket.can_send() && !data.is_empty() {
                    println!("tcp:{} send data: {:?}", PORT, data);
                    socket.send_slice(&data[..]).unwrap();
                }
            }
        }
        else if socket.may_send() {
            println!("tcp:{} close", PORT);
            socket.close();
            break;
        }
    }
}

fn recv(buffer: &mut [u8]) -> (usize, Vec<u8>) {
    let recvd_len = buffer.len();

    if !buffer.is_empty() {
        println!("tcp:{} recv {} bytes: {:?}", PORT, recvd_len, buffer);

        let mut lines = buffer
            .split(|&b| b == b'\n')
            .map(|b| b.to_owned())
            .collect::<Vec<_>>();
        for line in lines.iter_mut() {
            line.reverse();
        }

        let data = lines.join(&b'\n');

        (recvd_len, data)
    }
    else {
        (0, vec![])
    }
}