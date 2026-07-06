use std::env;
use std::process::{exit, Command};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-machine").arg("q35");
    qemu.arg("-m").arg("8G");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));

    qemu.arg("-serial").arg("mon:stdio");
    qemu.arg("-display").arg("none");

    qemu.arg("-netdev").arg("tap,id=u0,ifname=tap0,script=no,downscript=no,vhost=on");
    qemu.arg("-device").arg("virtio-net-pci,mac=7E-EF-41-A1-C9-D3,netdev=u0");

    qemu.arg("--device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    qemu.arg("-no-reboot");

    let exit_status = qemu.status().unwrap();
    match exit_status.code() {
        None => exit(-1),
        Some(code) => exit(code),
    }
}