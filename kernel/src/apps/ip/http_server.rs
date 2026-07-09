use crate::println;
use crate::utils::compat::DeviceWrapper;
use crate::utils::time::now;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::str::FromStr;
use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::socket::tcp;
use virtio_drivers::transport::pci::PciTransport;
use crate::apps::ip::utils::{create_sockets, SocketType};

const PORT: u16 = 80;

const SOCKET_NUMBER: u16 = 64;

pub fn http_server(mut device: DeviceWrapper<PciTransport>, mut iface: Interface) {
    let mut sockets = SocketSet::new(vec![]);
    let mut handles: Vec<SocketHandle> = Vec::new();

    create_sockets(&mut sockets, &mut handles, SocketType::Tcp, SOCKET_NUMBER);

    let mut tcp_handles: Vec<(SocketHandle, bool)> = handles.drain(..).map(|s| (s, false)).collect();

    println!("===== HTTP server started =====\n");

    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut device, &mut sockets);

        for (index, (tcp_handle, tcp_active)) in tcp_handles.iter_mut().enumerate() {
            let socket = sockets.get_mut::<tcp::Socket>(*tcp_handle);
            if !socket.is_open() {
                println!("s{index} | listening");
                socket.listen(PORT).unwrap();
            }

            if socket.is_active() && tcp_active == &false {
                println!("s{index} | connected");
            }
            else if !socket.is_active() && tcp_active == &true {
                println!("s{index} | disconnected");
            }

            *tcp_active = socket.is_active();

            if socket.may_recv() {
                if let Ok(data) = socket.recv(|buff| recv(buff)) {
                    if socket.can_send() && !data.is_empty() {
                        let text = String::from_str("hello world").unwrap();
                        let len = text.len();
                        let content = format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nConnection: keep-alive\r\n\r\n{text}");
                        socket.send_slice(content.as_bytes()).unwrap();
                    }
                }
            }
            else if socket.may_send() {
                println!("s{index} | close");
                socket.close();
            }
        }
    }
}

fn recv(buffer: &mut [u8]) -> (usize, Vec<u8>) {
    let recvd_len = buffer.len();

    if !buffer.is_empty() {
        //println!("tcp:{} recv {} bytes: {:?}", PORT, recvd_len, buffer);

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