#![no_main]
#![no_std]

use bobcat_sdk::{
    alloc::bobcat_allocator,
    cd::read_words,
    create::create1_unit,
    entry::{read_args_safe, write_word},
    maths::U,
    proxy::make_eip1967_proxy,
};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { 32 + 4 });
    let impl_ = read_words!(&args[4..], 1);
    let addr = create1_unit(&make_eip1967_proxy(impl_.into()), U::ZERO);
    write_word(&addr.unwrap().into());
    0
}
