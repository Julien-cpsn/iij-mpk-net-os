use x86_64::registers::model_specific::Msr;
use crate::cpu::protection::pk::{PkPermission, PK_MASK};
use crate::kdebug;

const TARGET: &str = "PKRS";

const IA32_PKRS: u32 = 0x6E1;

pub fn read_pkrs_key(pkey: u8) -> PkPermission {
    let pkrs = read_pkrs();

    let result = ((pkrs >> (pkey * 2)) & PK_MASK) as u8;
    let permission = PkPermission::from_repr(result).unwrap();

    kdebug!("PKRS key {} permission is {}", pkey, permission);

    permission
}

pub fn write_pkrs_key(pkey: u8, new_permission: PkPermission) {
    let shift = pkey * 2;
    let mask = PK_MASK << shift;

    kdebug!("PKRS key {} permission set to {}", pkey, new_permission);

    let mut pkru = read_pkrs();
    pkru &= !mask;
    pkru |= (new_permission as u32) << shift;

    write_pkrs(pkru);
}

fn read_pkrs() -> u32 {
    let msr = Msr::new(IA32_PKRS);
    unsafe { msr.read() as u32 }
}

fn write_pkrs(pkrs: u32) {
    let mut msr = Msr::new(IA32_PKRS);
    unsafe { msr.write(pkrs as u64) }
}