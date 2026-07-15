use crate::cpu::protection::pk::{PkPermission, PK_MASK};
use crate::cpu::protection::pkru::{read_pkru, write_pkru};
use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::kdebug;

const TARGET: &str = "PKRU";

pub fn read_pkru_key(regs: &SyscallRegs) {
    let pkey = regs.rdx as u8;

    let pkru = read_pkru();

    let result = ((pkru >> (pkey * 2)) & PK_MASK) as u8;
    let permission = PkPermission::from_repr(result).unwrap();

    kdebug!("PKRU key {} permission is {}", pkey, permission);

    // return here
    //permission
}

pub fn write_pkru_key(regs: &SyscallRegs) {
    let pkey = regs.rdx as u8;
    let new_permission = PkPermission::from_repr(regs.rdi as u8).unwrap();

    let shift = pkey * 2;
    let mask = PK_MASK << shift;

    kdebug!("PKRU key {} permission set to {}", pkey, new_permission);

    let mut pkru = read_pkru();
    pkru &= !mask;
    pkru |= (new_permission as u32) << shift;

    write_pkru(pkru);
}