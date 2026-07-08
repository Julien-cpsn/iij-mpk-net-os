use crate::cpu::protection::pk::{PkPermission, PK_MASK};


pub fn read_pkru_key(pkey: u8) -> PkPermission {
    let pkru = read_pkru();

    let result = ((pkru >> (pkey * 2)) & PK_MASK) as u8;

    PkPermission::from_repr(result).unwrap()
}

pub fn write_pkru_key(pkey: u8, new_permission: PkPermission) {
    let shift = pkey * 2;
    let mask = PK_MASK << shift;

    let mut pkru = read_pkru();
    pkru &= !mask;
    pkru |= (new_permission as u32) << shift;

    write_pkru(pkru);
}

#[inline]
fn read_pkru() -> u32 {
    unsafe { rdpkru() }
}

#[inline]
fn write_pkru(pkru: u32) {
    unsafe { wrpkru(pkru) }
}

#[allow(improper_ctypes)]
unsafe extern "unadjusted" {
    #[link_name = "llvm.x86.rdpkru"]
    fn rdpkru() -> u32;
    #[link_name = "llvm.x86.wrpkru"]
    fn wrpkru(pkru: u32);
}
