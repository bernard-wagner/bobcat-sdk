// chainlink-vrf-test: Mock Chainlink VRF with the help of arbos-foundry.
// Test to see if things would work under mocked circumstances for an
// interaction this way.

#![no_std]
#![no_main]

use bobcat_sdk::{
    alloc::bobcat_allocator, entry::write_bool, maths::U, storage::transient_exchange,
};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    write_bool(transient_exchange(&U::ONE, &U::ZERO, &U::ONE));
    0
}
