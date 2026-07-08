use crate::apps::interface::NET_QUEUE_SIZE;
use crate::drivers::dma::HalImpl;
use alloc::rc::Rc;
use core::cell::RefCell;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use virtio_drivers::device::net::{RxBuffer, VirtIONet};
use virtio_drivers::transport::Transport;
use virtio_drivers::Error;

pub type DeviceImpl<T> = VirtIONet<HalImpl, T, { NET_QUEUE_SIZE }>;
pub struct DeviceWrapper<T: Transport>(Rc<RefCell<DeviceImpl<T>>>);

impl<T: Transport> DeviceWrapper<T> {
    pub fn new(dev: DeviceImpl<T>) -> Self {
        DeviceWrapper(Rc::new(RefCell::new(dev)))
    }

    pub fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.0.borrow().mac_address())
    }
}

impl<T: Transport> Device for DeviceWrapper<T> {
    type RxToken<'a> = VirtIoRxToken<T> where Self: 'a;
    type TxToken<'a> = VirtIoTxToken<T> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self.0.borrow_mut().receive() {
            Ok(buf) => Some((
                VirtIoRxToken(self.0.clone(), buf),
                VirtIoTxToken(self.0.clone()),
            )),
            Err(Error::NotReady) => None,
            Err(err) => panic!("receive failed: {}", err),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(VirtIoTxToken(self.0.clone()))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.medium = Medium::Ethernet;
        caps
    }
}

pub struct VirtIoRxToken<T: Transport>(Rc<RefCell<DeviceImpl<T>>>, RxBuffer);
pub struct VirtIoTxToken<T: Transport>(Rc<RefCell<DeviceImpl<T>>>);

impl<T: Transport> RxToken for VirtIoRxToken<T> {
    fn consume<R, F>(self, f: F) -> R where F: FnOnce(&[u8]) -> R {
        let mut rx_buf = self.1;

        crate::println!("RECV {} bytes: {:02X?}", rx_buf.packet_len(), rx_buf.packet());

        let result = f(rx_buf.packet_mut());
        self.0.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();

        result
    }
}

impl<T: Transport> TxToken for VirtIoTxToken<T> {
    fn consume<R, F>(self, len: usize, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        let mut dev = self.0.borrow_mut();
        let mut tx_buf = dev.new_tx_buffer(len);
        let result = f(tx_buf.packet_mut());

        crate::println!("SEND {} bytes: {:02X?}", len, tx_buf.packet());

        dev.send(tx_buf).unwrap();

        result
    }
}