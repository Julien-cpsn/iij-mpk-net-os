use crate::cpu::protection::pk::set_frame_protection_key;
use crate::memory::tables::find_page_table_entry;
use crate::println;
use x86_64::VirtAddr;

pub fn pkey_mprotect(virt_addr: VirtAddr, length: u64, pkey: u8) {
    assert!(pkey < 16);

    let mut addr_offset = 0;

    while addr_offset < length {
        let iter_addr = virt_addr + addr_offset;
        let (entry, entry_size) = find_page_table_entry(iter_addr).unwrap();

        set_frame_protection_key(entry, pkey);

        println!("{entry_size} | {entry:?}");

        addr_offset += entry_size;
    }
}
