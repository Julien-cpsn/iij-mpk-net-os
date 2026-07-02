use spin::{LazyLock, Mutex};
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550Tty};


pub static SERIAL: LazyLock<Mutex<Uart16550Tty<PioBackend>>> = LazyLock::new(|| Mutex::new(create_serial()));

pub fn create_serial() -> Uart16550Tty<PioBackend> {
    unsafe {
        Uart16550Tty::new_port(0x3F8, Config::default())
    }
        .expect("should initialize serial device from valid config and valid port")
}