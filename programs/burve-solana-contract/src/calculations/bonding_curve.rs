#[derive(Debug, Clone)]
pub struct Parameters {
    pub a: u64,
    pub b: u64,
}

pub trait BondingCurve {
    fn calculate_mint_amount_from_bonding_curve(
        native_asset: u64,
        current_supply: u64,
        params: Parameters,
    ) -> (u64, u64);
    fn calculate_burn_amount_from_bonding_curve(
        token_amount: u64,
        current_supply: u64,
        params: Parameters,
    ) -> (u64, u64);
    fn price(current_supply: u64, params: Parameters) -> u64;
}
