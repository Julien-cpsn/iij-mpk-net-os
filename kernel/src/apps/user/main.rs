use crate::apps::user::benchmark::benchmark;
use core::hint::spin_loop;

#[unsafe(no_mangle)]
pub extern "C" fn user_entry() -> ! {
    main();
    loop {
        spin_loop();
    }
}

fn main() {
    benchmark();

    /*
    const PKEY: u8 = 6;
    let msg = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    println!("start");
    println!("{:?}", msg);

    pkey_mprotect(VirtAddr::from_ptr(msg.as_ptr()), msg.len(), PKEY);
    println!("{:?}", msg);

    write_pkru(PKEY, PkPermission::ReadWrite);
    println!("{:?}", msg);

    write_pkru(PKEY, PkPermission::NoAccess);
    println!("{:?}", msg);
    println!("end");*/
}