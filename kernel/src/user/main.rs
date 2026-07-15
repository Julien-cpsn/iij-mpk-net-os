use crate::user::apps::devices::pci_scan::pci_scan;
use crate::user::api::user_syscalls::exit;
use crate::user::apps::ip::udp_tx::udp_tx_server;
use crate::utils::qemu::QemuExitCode;

#[unsafe(no_mangle)]
pub extern "C" fn user_entry() -> ! {
    main();
    exit(QemuExitCode::Success);

    loop {}
}

fn main() {
    //crate::apps::user::benchmark::benchmark();

    let (mut device, mut iface) = pci_scan();

    udp_tx_server(&mut device, &mut iface);
}