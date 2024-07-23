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
        instructions::initialize(ctx, args)
    }

    pub fn set_burve_admin(ctx: Context<SetBurveAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::set_burve_admin(ctx, new_admin)
    }

    pub fn set_burve_tax(ctx: Context<SetBurveTax>, args: SetBurveTaxArgs) -> Result<()> {
        instructions::set_burve_tax(ctx, args)
    }

    pub fn create_new_project(ctx: Context<CreateProject>, args: CreateProjectArgs) -> Result<()> {
        instructions::create_project(ctx, args)
    }

    pub fn set_project_admin(ctx: Context<SetProjectAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::set_project_admin(ctx, new_admin)
    }

    pub fn set_project_tax(ctx: Context<SetProjectTax>, args: SetProjectTaxArgs) -> Result<()> {
        instructions::set_project_tax(ctx, args)
    }

    pub fn set_project_treasury(
        ctx: Context<SetProjectTreasury>,
        new_treasury: Pubkey,
    ) -> Result<()> {
        instructions::set_project_treasury(ctx, new_treasury)
    }

    pub fn mint_token_with_spl(
        ctx: Context<MintTokenWithSPL>,
        args: MintTokenWithSPLArgs,
    ) -> Result<()> {
        instructions::mint_token_with_spl(ctx, args)
    }

    pub fn burn_token_to_spl(ctx: Context<BurnTokenToSPL>, args: BurnTokenToSPLArgs) -> Result<()> {
        instructions::burn_token_to_spl(ctx, args)
    }

    pub fn mint_token_with_sol(
        ctx: Context<MintTokenWithSOL>,
        args: MintTokenWithSOLArgs,
    ) -> Result<()> {
        instructions::mint_token_with_sol(ctx, args)
    }

    pub fn burn_token_to_sol(ctx: Context<BurnTokenToSOL>, args: BurnTokenToSOLArgs) -> Result<()> {
        instructions::burn_token_to_sol(ctx, args)
    }

    pub fn claim_burve_spl_tax(
        ctx: Context<ClaimBurveSPLTax>,
        args: ClaimBurveSPLTaxArgs,
    ) -> Result<()> {
        instructions::claim_burve_spl_tax(ctx, args)
    }

    pub fn claim_burve_sol_tax(
        ctx: Context<ClaimBurveSOLTax>,
        args: ClaimBurveSOLTaxArgs,
    ) -> Result<()> {
        instructions::claim_burve_sol_tax(ctx, args)
    }

    pub fn check_mint_extensions_constraints(
        _ctx: Context<CheckMintExtensionConstraints>,
    ) -> Result<()> {
        Ok(())
    }
}
