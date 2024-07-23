// src/calculations/linear_bonding_curve.rs
use crate::bonding_curve::*;
use std::ops::{Add, Div, Mul, Sub};

pub struct LinearMixedBondingSwap;

impl BondingCurve for LinearMixedBondingSwap {
    // x => spl, y => native
    // p(x) = (kx^2)/2+px-Δy
    fn calculate_mint_amount_from_bonding_curve(
        raising_token_amount: u64,
        token_current_supply: u64,
        parameters: Parameters,
    ) -> (u64, u64) {
        let (k, p) = (parameters.a, parameters.b);
        if k == 0u64 {
            let token_amount = raising_token_amount.mul(1e9 as u64).div(p);
            (token_amount, raising_token_amount)
        } else {
            let token_current_price = (token_current_supply as f64)
                .mul(k as f64)
                .div(1e9 as f64)
                .add(p as f64);
            let token_amount = (token_current_price
                .mul(token_current_price)
                .add(raising_token_amount.mul(2u64).mul(k) as f64)
                .sqrt()
                .sub(token_current_price))
            .mul(1e9 as f64)
            .div(k as f64)
            .round() as u64;
            (token_amount, raising_token_amount)
        }
    }

    fn calculate_burn_amount_from_bonding_curve(
        token_amount: u64,
        token_current_supply: u64,
        parameters: Parameters,
    ) -> (u64, u64) {
        let (k, p) = (parameters.a, parameters.b);
        let native_token_amount = token_current_supply
            .mul(k)
            .add(p.mul(1e9 as u64))
            .mul(token_amount)
            .sub(token_amount.mul(token_amount).mul(k).div(2u64))
            .div(1e36 as u64);
        (token_amount, native_token_amount)
    }

    fn price(token_current_supply: u64, parameters: Parameters) -> u64 {
        let (k, p) = (parameters.a, parameters.b);
        token_current_supply.mul(k).div(1e9 as u64).add(p)
    }
}
