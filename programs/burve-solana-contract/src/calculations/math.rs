use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::ops::{Add, Div, Mul};

pub const PRECISION: usize = 18; // Precision for calculations

pub fn e() -> BigUint {
    BigUint::from(2718281828459045235u128) // Scaled e to PRECISION
}

pub fn ln(value: BigUint) -> BigUint {
    if value == Zero::zero() {
        return Zero::zero();
    }

    let mut result = BigUint::zero();
    let mut n = value;

    while n > One::one() {
        n = n.div(e());
        result = result.add(BigUint::one());
    }

    result
}
