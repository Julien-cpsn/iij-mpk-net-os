use std::{env, fs};

fn main() {
    let current_exe = env::current_exe().unwrap();
    let bios_target = current_exe.with_file_name("bios.img");

    fs::copy(env!("BIOS_IMAGE"), &bios_target).unwrap();

    println!("BIOS disk image at {}", bios_target.display());
}
