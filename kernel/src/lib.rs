#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![allow(internal_features)]

extern crate alloc;

pub mod utils;
pub mod drivers;
pub mod apps;
pub mod memory;
pub mod cpu;