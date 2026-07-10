#[inline]
pub fn read_pkru() -> u32 {
    unsafe { rdpkru() }
}

#[inline]
pub fn write_pkru(pkru: u32) {
    unsafe { wrpkru(pkru) }
}

#[allow(improper_ctypes)]
unsafe extern "unadjusted" {
    #[link_name = "llvm.x86.rdpkru"]
    fn rdpkru() -> u32;
    #[link_name = "llvm.x86.wrpkru"]
    fn wrpkru(pkru: u32);
}
