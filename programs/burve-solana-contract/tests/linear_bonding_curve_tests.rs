// tests/linear_bonding_curve_tests.rs
use burve_solana_contract::calculations::linear_bonding_curve::LinearMixedBondingSwap;
use burve_solana_contract::calculations::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mint_amount_from_linear_bonding_curve() {
        let parameters = Parameters { a: 2, b: 3 };
        let raising_token_amount: u64 = 100;
        let token_current_supply: u64 = 50;

        let (token_amount, raising_token_amount) =
            LinearMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                raising_token_amount,
                token_current_supply,
                parameters,
            );

        assert!(token_amount > 0);
        assert_eq!(raising_token_amount, 100);
    }

    #[test]
    fn test_calculate_burn_amount_from_linear_bonding_curve() {
        let parameters = Parameters { a: 2, b: 3 };
        let token_amount: u64 = 50;
        let token_current_supply: u64 = 100;

        let (token_amount, raising_token_amount) =
            LinearMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
                token_amount,
                token_current_supply,
                parameters,
            );

        assert!(raising_token_amount > 0);
        assert_eq!(token_amount, 50);
    }

    #[test]
    fn test_price_from_linear_bonding_curve() {
        let parameters = Parameters { a: 2, b: 3 };
        let token_current_supply: u64 = 100;

        let price = LinearMixedBondingSwap::price(token_current_supply, parameters);

        assert!(price > 0);
    }
}
