use std::env;
use std::process::{exit, Command};

#[cfg(all(feature = "tap", feature = "dpdk-vhost"))]
compile_error!("Cannot use both \"tap\" and \"dpdk-vhost\" features at the same time");

const MEM_SIZE: &str = "6G";
const FREQUENCY_GHZ: f32 = 2.6;

fn main() {
    println!("{}", env!("BIOS_IMAGE"));

    #[cfg(feature = "dpdk-vhost")]
    let mut qemu = {
        println!("QEMU should be run as root to access hugepages");

        let mut qemu = Command::new("sudo");
        qemu.arg("qemu-system-x86_64");

        qemu
    };

    #[cfg(not(feature = "dpdk-vhost"))]
    let mut qemu = Command::new("qemu-system-x86_64");

    qemu.arg("-cpu").arg(format!("host,+pku,+invtsc,tsc-frequency={}", (FREQUENCY_GHZ * 1_000_000_000.0) as u64));
    qemu.arg("-machine").arg("q35,accel=kvm");
    qemu.arg("-m").arg(MEM_SIZE);
    qemu.arg("-enable-kvm");
    qemu.arg("--mem-prealloc");

    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));

    qemu.arg("-serial").arg("mon:stdio");
    qemu.arg("-display").arg("none");

    #[cfg(feature = "dpdk-vhost")]
    {
        qemu.arg("-chardev").arg("socket,id=char0,path=/tmp/vhost-user1,server=on");
        qemu.arg("-netdev").arg("type=vhost-user,id=u0,chardev=char0,vhostforce=on");
        qemu.arg("-object").arg(format!("memory-backend-file,id=mem0,size={MEM_SIZE},mem-path=/dev/hugepages,share=on"));
        qemu.arg("-numa").arg("node,memdev=mem0");
    }

    #[cfg(feature = "tap")]
    {
        qemu.arg("-netdev").arg("tap,id=u0,ifname=tap0,script=no,downscript=no,vhost=on");
    }

    #[cfg(any(feature = "dpdk-vhost", feature = "tap"))]
    qemu.arg("-device").arg("virtio-net-pci,mac=7E-EF-41-A1-C9-D3,netdev=u0");

    qemu.arg("--device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    qemu.arg("-no-reboot");

    let exit_status = qemu.status().unwrap();
    match exit_status.code() {
        None => exit(-1),
        Some(code) => exit(code),
    }
}