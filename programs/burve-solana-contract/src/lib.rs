use anchor_lang::prelude::*;

pub mod calculations;
pub mod instructions;
pub mod utils;

pub use calculations::*;
pub use instructions::*;
pub use utils::*;

declare_id!("3bZ4oQHBhP95DVdBteTG13meav1qEnKnUzxb5SjmSpQH");

#[program]
pub mod burve_solana_contract {
    use super::*;

    pub fn initialize(ctx: Context<BurveInitialize>, args: InitializeArgs) -> Result<()> {
        burve_initialize(ctx, args)
    }

    pub fn set_burve_admin(ctx: Context<SetBurveAdmin>, new_admin: Pubkey) -> Result<()> {
        burve_set_admin(ctx, new_admin)
    }

    pub fn set_burve_tax(ctx: Context<SetBurveTax>, args: SetBurveTaxArgs) -> Result<()> {
        burve_set_tax(ctx, args)
    }

    pub fn create_new_project_with_spl(
        ctx: Context<CreateProjectWithSPL>,
        args: CreateProjectArgs,
    ) -> Result<()> {
        factory_create_project_with_spl(ctx, args)
    }

    pub fn create_new_project_with_sol(
        ctx: Context<CreateProjectWithSOL>,
        args: CreateProjectArgs,
    ) -> Result<()> {
        factory_create_project_with_sol(ctx, args)
    }

    pub fn set_project_admin(ctx: Context<SetProjectAdmin>, new_admin: Pubkey) -> Result<()> {
        factory_set_project_admin(ctx, new_admin)
    }

    pub fn set_project_tax(ctx: Context<SetProjectTax>, args: SetProjectTaxArgs) -> Result<()> {
        factory_set_project_tax(ctx, args)
    }

    pub fn set_project_treasury(
        ctx: Context<SetProjectTreasury>,
        new_treasury: Pubkey,
    ) -> Result<()> {
        factory_set_project_treasury(ctx, new_treasury)
    }

    pub fn mint_token_with_spl(
        ctx: Context<MintTokenWithSPL>,
        args: MintTokenWithSPLArgs,
    ) -> Result<()> {
        route_mint_token_with_spl(ctx, args)
    }

    pub fn burn_token_to_spl(ctx: Context<BurnTokenToSPL>, args: BurnTokenToSPLArgs) -> Result<()> {
        route_burn_token_to_spl(ctx, args)
    }

    pub fn mint_token_with_sol(
        ctx: Context<MintTokenWithSOL>,
        args: MintTokenWithSOLArgs,
    ) -> Result<()> {
        route_mint_token_with_sol(ctx, args)
    }

    pub fn burn_token_to_sol(ctx: Context<BurnTokenToSOL>, args: BurnTokenToSOLArgs) -> Result<()> {
        route_burn_token_to_sol(ctx, args)
    }

    pub fn claim_burve_spl_tax(
        ctx: Context<ClaimBurveSPLTax>,
        args: ClaimBurveSPLTaxArgs,
    ) -> Result<()> {
        route_claim_burve_spl_tax(ctx, args)
    }

    pub fn claim_burve_sol_tax(
        ctx: Context<ClaimBurveSOLTax>,
        args: ClaimBurveSOLTaxArgs,
    ) -> Result<()> {
        route_claim_burve_sol_tax(ctx, args)
    }
}
