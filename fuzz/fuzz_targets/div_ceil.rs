#![no_main]

use alloy_primitives::U256;
use bobcat_maths::U;
use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
    fuzz_target,
};

#[derive(Arbitrary, Debug)]
struct DivCeil {
    x: U,
    y: U,
}

macro_rules! assert_eq_u {
    ($expected:expr, $actual:expr, $x:expr, $y:expr) => {
        assert_eq!(
            $expected.to_be_bytes::<32>(),
            $actual.0,
            "div_ceil mismatch for ({}, {})",
            $x,
            $y,
        )
    };
}

fuzz_target!(|data: DivCeil| {
    let DivCeil { x, y } = data;
    if y.is_zero() {
        assert_eq!(x.wrapping_ceil_div(&y), U::ZERO);
        assert_eq!(x.checked_ceil_div_opt(&y), Some(U::ZERO));
        assert_eq!(x.checked_ceil_div(&y), U::ZERO);
        return;
    }
    let exp = U256::from_be_bytes(x.0).div_ceil(U256::from_be_bytes(y.0));
    assert_eq_u!(exp, x.wrapping_ceil_div(&y), x, y);
    let checked = x
        .checked_ceil_div_opt(&y).unwrap();
    assert_eq_u!(exp, checked, x, y);
    assert_eq_u!(exp, x.checked_ceil_div(&y), x, y);
});
