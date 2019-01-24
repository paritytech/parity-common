#![feature(lang_items, start)]
#![no_std]

extern crate libc;
extern crate ethereum_types;
extern crate ethbloom;
extern crate fixed_hash;

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
   0
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        libc::abort();
    }
}
