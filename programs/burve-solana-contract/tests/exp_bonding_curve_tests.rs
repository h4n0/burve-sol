// tests/exp_mixed_bonding_swap_tests.rs
use burve_solana_contract::calculations::*;

#[cfg(test)]
mod tests {
    use super::*;
    use burve_solana_contract::Parameters;
    use rand::Rng;

    // FIXME This does NOT work right now, because currently we can't handle a decimal of 18 digits.
    // In solana, a standard of decimal is 9 digits, so the balance storage is using u64.
    // The max value of u64 is 18,446,744,073,709,551,615, which is around 10^19.
    // So, a token with decimal of 9 digits and max supply of 10^10 is the maximum we can handle.
    fn setup() -> (u64, u64, u64, Parameters) {
        let supply = 10_000_000 * 1_000_000_000; // 1e7 * 1e18
        let px = 1_000 * 1_000; // 0.001 * 1e18
        let tvl = 2_000 * 1_000_000_000; // 2000 * 1e18
        let round = 100;

        let a = 10_000_000; // 0.01 * 1e18
        let b = (1_000 * 1_000_000_000 / a as u128) as u64; // (1000 * 1e18) / a
        let parameters = Parameters { a, b };

        (supply, px, tvl, parameters)
    }

    #[test]
    fn test_calculation() {
        let (_supply, _px, tvl, parameters) = setup();
        let native_asset = tvl.clone();

        let (token_amount1, raising_token_amount1) =
            ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                native_asset.clone(),
                0,
                parameters.clone(),
            );
        let (token_amount2, raising_token_amount2) =
            ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
                token_amount1.clone(),
                token_amount1.clone(),
                parameters.clone(),
            );
        let price = ExpMixedBondingSwap::price(token_amount1.clone(), parameters.clone());

        assert_eq!(raising_token_amount1, native_asset);
        assert!(token_amount1 > 0);
        assert!(price > 0);
    }

    #[test]
    fn test_estimate_mint_burn() {
        let (_supply, _px, _tvl, parameters) = setup();
        let user1 = 1_000_000_000u64; // 1 * 1e18

        let (received_amount, _raising_token_amount) =
            ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                user1,
                0,
                parameters.clone(),
            );
        assert!(received_amount > 0);

        let erc20_balance = received_amount.clone();
        let (_, amount_return) = ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
            erc20_balance.clone(),
            erc20_balance.clone(),
            parameters.clone(),
        );
        assert!(amount_return > 0);
    }

    #[test]
    fn test_multi_mint() {
        let (_supply, _px, _tvl, parameters) = setup();
        let user1 = 1_000_000_000u64; // 1 * 1e18
        let round = 100;

        for _ in 0..round {
            let (minted_amount, _raising_token_amount) =
                ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                    user1,
                    0,
                    parameters.clone(),
                );
            assert!(minted_amount > 0);
        }
    }

    #[test]
    fn test_multi_burn() {
        let (_supply, _px, _tvl, parameters) = setup();
        let user1 = 5_000_000_000u64; // 5 * 1e18
        let round = 100;

        let (initial_minted_amount, _raising_token_amount) =
            ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                user1,
                0,
                parameters.clone(),
            );
        assert!(initial_minted_amount > 0);

        for _ in 0..round {
            let (minted_amount, _raising_token_amount) =
                ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                    1_000_000_000u64,
                    0,
                    parameters.clone(),
                ); // 1000 * 1e18
            assert!(minted_amount > 0);

            let (_, amount_return) = ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
                initial_minted_amount,
                initial_minted_amount,
                parameters.clone(),
            );
            assert!(amount_return > 0);
        }
    }

    #[test]
    fn test_random_mint_and_burn() {
        let (_supply, _px, _tvl, parameters) = setup();
        let round = 100;

        for _ in 0..round {
            let mut rng = rand::thread_rng();
            let amount1 = rng.gen_range(1..=49) * 100_000_000_000_000_000u64; // (1..49) * 1e17
            let amount2 = rng.gen_range(1..=999) * 100_000_000_000_000_000u64; // (1..999) * 1e17
            let amount3 = rng.gen_range(100..=999) * 100_000_000_000_000_000u64; // (100..999) * 1e17

            let (minted_amount1, _raising_token_amount1) =
                ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                    amount1,
                    0,
                    parameters.clone(),
                );
            assert!(minted_amount1 > 0);

            let (minted_amount2, _raising_token_amount2) =
                ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                    amount2,
                    0,
                    parameters.clone(),
                );
            assert!(minted_amount2 > 0);

            let (_, burn_amount_return) =
                ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
                    minted_amount2,
                    minted_amount2,
                    parameters.clone(),
                );
            assert!(burn_amount_return > 0);

            let (minted_amount3, _raising_token_amount3) =
                ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                    amount3,
                    0,
                    parameters.clone(),
                );
            assert!(minted_amount3 > 0);

            if rng.gen_bool(0.2) {
                let (_, burn_amount_return) =
                    ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
                        minted_amount3,
                        minted_amount3,
                        parameters.clone(),
                    );
                assert!(burn_amount_return > 0);
            }
        }
    }

    #[test]
    fn test_fuzz() {
        let (_supply, _px, _tvl, parameters) = setup();
        let amount = 979_999_999_999_999_107u64; // large number * 1e18
        assert!(amount > 0);

        let (minted_amount, _raising_token_amount) =
            ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
                amount,
                0,
                parameters.clone(),
            );
        assert!(minted_amount > 0);

        let (_, burn_amount_return) = ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
            minted_amount,
            minted_amount,
            parameters.clone(),
        );
        assert!(burn_amount_return > 0);
    }
}
