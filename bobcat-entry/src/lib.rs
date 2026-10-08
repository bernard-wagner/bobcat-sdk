#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

pub use bobcat_maths::U;

type Address = [u8; 20];

pub use bobcat_cd::{EvmCdDeserialise, EvmCdError, read_words};

pub use bobcat_host as host;

use array_concat::concat_arrays;

pub fn balance(addr: Address) -> U {
    let mut out = U::ZERO;
    unsafe { host::account_balance(addr.as_ptr(), out.as_mut_ptr()) }
    out
}

#[unsafe(no_mangle)]
#[cfg(all(
    target_family = "wasm",
    target_os = "unknown",
    not(feature = "dont-define-symbols")
))]
pub unsafe fn mark_used() {
    unsafe { host::pay_for_memory_grow(0) }
    panic!();
}

/// Write a result slice.
pub fn write_slice(s: &[u8]) {
    unsafe { host::write_result(s.as_ptr(), s.len()) }
}

#[deprecated = "Replaced by write_slice"]
pub fn write_result_slice(x: &[u8]) {
    write_slice(x)
}

pub fn write_word(s: &U) {
    write_slice(&s.0)
}

#[deprecated = "Renamed to write_word"]
pub fn write_result_word(x: &U) {
    write_word(x)
}

pub fn write_bool(v: bool) {
    write_slice(&U::from(v).0)
}

const OFFSET_ARR: [u8; 32] = U::from_u32(32).0;

/// Helper function that create a fresh array with the length and offset
/// by concatinating arrays. Only writes in the end the array length
/// given. CD_LEN is the total length, so must be 32 * 2 + ARR_LEN.
pub fn write_array_slice_len<const ARR_LEN: usize, const CD_LEN: usize>(
    arr: [u8; ARR_LEN],
    len_arr: usize,
) {
    assert!(
        ARR_LEN + 32 * 2 == CD_LEN,
        "bad array length, need: {}",
        ARR_LEN + 32 * 2
    );
    assert!(
        ARR_LEN >= len_arr,
        "array length {len_arr} larger than buffer {ARR_LEN}"
    );
    let U(len_arr) = U::from_usize(ARR_LEN);
    let x: [u8; CD_LEN] = concat_arrays!(OFFSET_ARR, len_arr, arr);
    write_slice(&x)
}

/// Helper function that create a fresh array with the length and offset
/// by concatinating arrays. CD_LEN is the total length, so must be 32 * 2 + ARR_LEN.
pub fn write_array_slice<const ARR_LEN: usize, const CD_LEN: usize>(arr: [u8; ARR_LEN]) {
    write_array_slice_len::<ARR_LEN, CD_LEN>(arr, ARR_LEN)
}

#[cfg(feature = "alloc")]
pub fn write_array_vec(arr: Vec<u8>) {
    let mut v = Vec::with_capacity(32 * 2 + arr.len());
    v.extend_from_slice(&U::from_usize(32).0);
    v.extend_from_slice(&U::from_usize(arr.len()).0);
    v.extend(arr);
    write_slice(&v)
}

/// Write the slice given as Ethereum's String/Bytes type, with the length
/// and offset prefixed.
#[cfg(feature = "alloc")]
pub fn write_bytes(x: &[u8]) {
    let mut v = Vec::with_capacity(32 * 2 + x.len());
    v.extend_from_slice(&U::from_usize(32).0);
    v.extend_from_slice(&U::from_usize(x.len()).0);
    v.extend_from_slice(x);
    write_slice(&v)
}

#[cfg(feature = "alloc")]
pub fn write_string(x: String) {
    write_array_vec(x.into_bytes())
}

/// String buffer size we use for `write_str`.
pub const STR_BUFFER_SIZE: usize = 128 + 32 * 2;

/// Write a small string, allocating a small arena (192 bytes, 128
/// characters max to use here) for the writing.
pub fn write_str(x: &str) {
    let len = 64 + x.len().div_ceil(32) * 32;
    assert!(len <= STR_BUFFER_SIZE, "str too large");
    let mut buf = [0u8; STR_BUFFER_SIZE];
    // 0x20 for the first word:
    buf[31] = 0x20;
    buf[32..32 * 2].copy_from_slice(U::from(x.len()).as_slice());
    buf[32 * 2..32 * 2 + x.len()].copy_from_slice(x.as_bytes());
    write_slice(&buf[..len])
}

pub fn return_data_size() -> usize {
    unsafe { host::return_data_size() }
}

pub use bobcat_cd::leftpad_addr;

/// Like write_exit_call, except it only reverts with the
/// returndata if the underlying call reverted. If it doesn't, then it
/// just returns the slice.
#[macro_export]
macro_rules! revert_if_bad_call_vec {
    ($e:expr) => {{
        let (rc, rd) = $e;
        if !rc {
            $crate::write_slice(&rd);
            return 1;
        }
        rd
    }};
}

/// Reverts if the underlying call failed, using the vector that was
/// returned as the third argument as slice.
#[macro_export]
macro_rules! revert_if_bad_call_unit_vec {
    ($e:expr) => {{
        let (rc, revertdata) = $e;
        match (rc, revertdata) {
            (true, _) => (),
            (false, Some(v)) => {
                $crate::write_slice(&v);
                return 1;
            }
            (false, _) => return 1,
        }
    }};
}

/// Reverts with a message if the revertdata is Some, and if the rc is false.
#[macro_export]
macro_rules! revert_if_bad_call_slice_vec {
    ($e:expr) => {{
        let (rc, returndata, revertdata) = $e;
        match (rc, revertdata) {
            (true, _) => returndata,
            (false, Some(v)) => {
                $crate::write_slice(&v);
                return 1;
            }
            (false, _) => return 1,
        }
    }};
}

#[macro_export]
macro_rules! write_exit_res {
    ($ident:expr) => {{
        match $ident {
            Ok(v) => {
                $crate::write_slice(&v);
                0
            }
            Err(v) => {
                $crate::write_slice(&v);
                1
            }
        }
    }};
}

#[macro_export]
macro_rules! write_exit_create {
    ($ident:expr) => {{
        let (addr, b, i) = $ident;
        if addr != [0u8; 20] {
            $crate::write_slice(&leftpad_addr(addr));
            0
        } else {
            $crate::write_slice(&b[..i]);
            1
        }
    }};
}

#[macro_export]
macro_rules! write_exit_call {
    ($ident:expr) => {{
        let (rc, l, v) = $ident;
        $crate::write_slice(&v[..l]);
        if rc { 0 } else { 1 }
    }};
}

/// Read arguments from the Stylus VM (calldata), using unsafe
/// MaybeUninit code if we're on the wasm host. If we're on a non wasm
/// host, zero it out then return it. We also return the length.
pub fn read_args<const CAP: usize>(len: usize) -> ([u8; CAP], usize) {
    assert!(CAP >= len, "cap not enough");
    // SAFETY: This is safe since the host will write over this, and on
    // the Stylus host this will always be zeroed out anyway.
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    let mut b = unsafe { core::mem::MaybeUninit::<[u8; CAP]>::uninit().assume_init() };
    // On other hosts, we don't make the assumption about this and zero it out:
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    let mut b = [0u8; CAP];
    unsafe { host::read_args(b.as_mut_ptr()) };
    (b, len)
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub fn args_len() -> usize {
    unsafe { host::args_len() }
}

/// Read args and deserialise them into W, returning an error when the
/// generated or built-in buffer cannot hold the calldata.
pub fn read_cd_res<W: EvmCdDeserialise>(len: usize) -> Result<W, EvmCdError> {
    let mut args = W::new_buffer(len)?;
    unsafe { host::read_args(args.as_mut().as_mut_ptr()) };
    W::deserialise(&args.as_ref()[..len])
}

/// Read args and deserialise them into W, panicking on invalid calldata.
pub fn read_cd<W: EvmCdDeserialise>(len: usize) -> W {
    read_cd_res::<W>(len).unwrap()
}

#[macro_export]
macro_rules! read_args_safe {
    ($len:expr, $max_len:expr) => {{
        core::assert!($max_len >= $len, "{} < {}", $max_len, $len);
        $crate::read_args::<$max_len>($len).0
    }};
}

#[cfg(feature = "alloc")]
pub fn read_args_vec(len: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(len);
    unsafe {
        host::read_args(b.as_mut_ptr());
        b.set_len(len);
    };
    b
}

pub fn msg_sender() -> Address {
    let mut b = [0u8; 20];
    unsafe { host::msg_sender(b.as_mut_ptr()) }
    b
}

pub fn contract_address() -> Address {
    let mut b = [0u8; 20];
    unsafe { host::contract_address(b.as_mut_ptr()) }
    b
}

pub fn msg_value() -> U {
    let mut b = [0u8; 32];
    unsafe { host::msg_value(b.as_mut_ptr()) }
    U(b)
}

pub fn code_size(addr: Address) -> usize {
    unsafe { host::account_code_size(addr.as_ptr()) }
}

pub fn code_slice<const CAP: usize>(
    addr: Address,
    size: usize,
    offset: usize,
) -> ([u8; CAP], usize) {
    let mut b = [0u8; CAP];
    assert!(CAP >= size, "not enough size: {size}, capacity: {CAP}");
    let rd = unsafe { host::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    (b, rd)
}

#[cfg(feature = "alloc")]
pub fn code_vec_size(addr: Address, offset: usize, size: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(size);
    let rd = unsafe { host::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    unsafe { b.set_len(rd) };
    b
}

#[cfg(feature = "alloc")]
pub fn code_vec(addr: Address, offset: usize) -> Vec<u8> {
    code_vec_size(addr, offset, code_size(addr))
}

pub fn code_hash(addr: Address) -> U {
    let mut b = U::ZERO;
    unsafe { host::account_codehash(addr.as_ptr(), b.as_mut_ptr()) };
    b
}

pub fn chain_id() -> u64 {
    unsafe { host::chainid() }
}

pub fn block_timestamp() -> u64 {
    unsafe { host::block_timestamp() }
}

pub fn block_basefee() -> U {
    let mut out = U::ZERO;
    unsafe { host::block_basefee(out.as_mut_ptr()) }
    out
}

pub fn evm_gas_left() -> u64 {
    unsafe { host::evm_gas_left() }
}

pub fn evm_ink_left() -> u64 {
    unsafe { host::evm_ink_left() }
}

pub unsafe fn exit_early(code: usize) -> ! {
    unsafe { host::exit_early(code as i32) }
}

pub unsafe fn storage_flush_cache(clear: bool) {
    unsafe { host::storage_flush_cache(clear) }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn read_cd_uses_the_deserialiser_buffer() {
        let mut args = vec![0u8; 32];
        args[28..].copy_from_slice(&7u32.to_be_bytes());
        host::set_args(args);

        assert_eq!(read_cd_res::<u32>(32).unwrap(), 7);
        assert!(read_cd_res::<u32>(33).is_err());
    }
}
