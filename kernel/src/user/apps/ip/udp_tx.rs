use crate::info;
use crate::user::apps::devices::virtio_net::net::VirtioNetQueue;

const TARGET: &str = "UDP TX";

pub const UDP_PACKET_METADATA_BUFFER_SIZE: usize = 2;
pub const UDP_PACKET_PAYLOAD_BUFFER_SIZE: usize = 65_535;

const PACKET_SIZE: usize = 10;
const BATCH_SIZE: usize = 128;

pub fn udp_tx_server(virt_net_queue: &mut VirtioNetQueue) {
    info!("===== UDP TX server started =====");

    loop {
        virt_net_queue.transmit_batch(
            BATCH_SIZE,
            10
        );
    }
}