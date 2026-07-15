use crate::debug;
use crate::user::apps::devices::pci_scan::pci_scan;
use crate::user::api::user_syscalls::exit;
use crate::user::apps::ip::udp_tx::udp_tx_server;
use crate::user::c_api::c_add_numbers;
use crate::utils::qemu::QemuExitCode;


const TARGET: &str = "MAIN";

#[unsafe(no_mangle)]
pub extern "C" fn user_entry() -> ! {
    main();
    exit(QemuExitCode::Success);

    loop {}
}

fn main() {
    let result = unsafe { c_add_numbers(10, 20) };
    debug!("Result from C: {}", result);

    //crate::apps::user::benchmark::benchmark();

    let (mut device, mut iface) = pci_scan();

    udp_tx_server(&mut device, &mut iface);
}