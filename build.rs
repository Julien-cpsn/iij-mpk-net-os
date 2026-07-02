use std::env;
use std::path::PathBuf;
use bootloader::DiskImageBuilder;

fn main() {
    let disk_builder = DiskImageBuilder::new(PathBuf::from(env::var("CARGO_BIN_FILE_KERNEL").unwrap()));

    // specify output paths
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bios_path = out_dir.join("bios.img");

    // create the disk images
    disk_builder.create_bios_image(&bios_path).unwrap();

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}