use crate::user::api::user_syscalls::exit;
use crate::user::apps::devices::pci_scan::pci_scan;
use crate::user::apps::ip::udp_tx::udp_tx_server;
use crate::user::c_api::c_add_numbers;
use crate::user::multithreading::scheduler::{init_scheduler, run_scheduler, spawn};
use crate::utils::qemu::QemuExitCode;
use crate::debug;

const TARGET: &str = "MAIN";

#[unsafe(no_mangle)]
pub extern "C" fn user_entry() -> ! {
    init_scheduler();
    main();
    exit(QemuExitCode::Success);

    loop {}
}

async fn test() {
    debug!("TTT");
}

fn main() {
    //crate::apps::user::benchmark::benchmark();

    let task_0 = async {
        let result = unsafe { c_add_numbers(10, 20) };
        debug!("Result from C: {}", result);
    };
    let task_1 = test();
    let task_2 = async {
        let (mut device, mut iface) = pci_scan();
        udp_tx_server(&mut device, &mut iface);
    };

    spawn(task_0);
    spawn(task_1);
    spawn(task_2);
    run_scheduler();
}