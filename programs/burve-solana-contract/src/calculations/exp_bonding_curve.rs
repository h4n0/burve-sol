// src/calculations/exp_bonding_curve.rs
use crate::bonding_curve::*;

pub struct ExpMixedBondingSwap;

impl BondingCurve for ExpMixedBondingSwap {
    // x => tokenAmount, y => raisingTokenAmount
    // y = (a) e**(x/b)
    // tokenAmount = b * ln(e ^ (tokenCurrentSupply / b) + raisingTokenAmount / a / b) - tokenCurrentSupply
    fn calculate_mint_amount_from_bonding_curve(
        raising_token_amount: u64,
        token_current_supply: u64,
        parameters: Parameters,
    ) -> (u64, u64) {
        let (a, b) = (parameters.a as f64, parameters.b as f64);
        // FIXME: Boundary check!!
        let e_index = token_current_supply as f64 / b;
        let e_mod = raising_token_amount as f64 / b / a * 1e9f64;
        let exp_val = (e_index.exp() + e_mod).ln();
        assert!(exp_val >= 0f64);
        let token_amount = (exp_val * b).round() as u64 - token_current_supply;

        (token_amount, raising_token_amount)
    }

    // x => tokenAmount, y => raisingTokenAmount
    // y = (a) e**(x/b)
    // raisingTokenAmount = ab * (e ^ (tokenCurrentSupply / b) - e ^ ((tokenCurrentSupply - tokenAmount) / b))
    fn calculate_burn_amount_from_bonding_curve(
        token_amount: u64,
        token_current_supply: u64,
        parameters: Parameters,
    ) -> (u64, u64) {
        let (a, b) = (parameters.a as f64, parameters.b as f64);
        let e_index_1 = token_current_supply as f64 / b;
        let e_index_0 = (token_current_supply - token_amount) as f64 / b;
        let exp_val1 = e_index_1.exp();
        let exp_val0 = e_index_0.exp();
        let y = exp_val1 - exp_val0;
        assert!(y >= 0f64);
        let raising_token_amount = (y * a * b / 1e9f64).round() as u64;
        (token_amount, raising_token_amount)
    }

    // price = a  * e ^ (tokenCurrentSupply / b)
    fn price(token_current_supply: u64, parameters: Parameters) -> u64 {
        let (a, b) = (parameters.a as f64, parameters.b as f64);
        let e_index = token_current_supply as f64 / b;
        let exp_val = e_index.exp();
        assert!(exp_val >= 0f64);
        (exp_val * a).round() as u64
    }
}
