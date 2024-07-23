use std::mem::size_of;

use anchor_lang::prelude::*; 

use crate::Errors;

const MAX_PLATFORM_TAX_RATE: u16 = 100;

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct InitializeArgs {
    pub admin: Pubkey,
	pub treasury: Pubkey,
}
pub fn initialize(ctx: Context<BurveInitialize>, args: InitializeArgs) -> Result<()> {
    ctx.accounts.burve_base.admin = args.admin;
	ctx.accounts.burve_base.treasury = args.treasury;
    ctx.accounts.burve_base.mint_tax = 100;
    ctx.accounts.burve_base.burn_tax = 100;
    Ok(())
}

pub fn set_burve_admin(ctx: Context<SetBurveAdmin>, new_admin: Pubkey) -> Result<()> {
    ctx.accounts.burve_base.admin = new_admin;
    Ok(())
}

#[derive(Accounts)]
pub struct SetBurveTreasury<'info> {
	#[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
	pub burve_base: Account<'info, BurveBase>,
	pub admin: Signer<'info>,
}

pub fn set_burve_treasury(ctx: Context<SetBurveTreasury>, new_treasury: Pubkey) -> Result<()> {
	ctx.accounts.burve_base.treasury = new_treasury;
	Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SetBurveTaxArgs {
    pub new_mint_tax: u16,
    pub new_burn_tax: u16,
}

pub fn set_burve_tax(ctx: Context<SetBurveTax>, args: SetBurveTaxArgs) -> Result<()> {
    require!(
        args.new_mint_tax <= MAX_PLATFORM_TAX_RATE,
        Errors::TaxRateNotValid
    );
    require!(
        args.new_burn_tax <= MAX_PLATFORM_TAX_RATE,
        Errors::TaxRateNotValid
    );
    ctx.accounts.burve_base.mint_tax = args.new_mint_tax;
    ctx.accounts.burve_base.burn_tax = args.new_burn_tax;
    Ok(())
}


#[account]
pub struct BurveBase {
    pub admin: Pubkey,
	pub treasury: Pubkey,
    pub mint_tax: u16,
    pub burn_tax: u16,
}

#[derive(Accounts)]
pub struct BurveInitialize<'info> {
    #[account(
		init, 
		payer = signer, 
		space = size_of::<BurveBase>() + 8, 
		seeds = [b"burve"], 
		bump 
	)]
    pub burve_base: Account<'info, BurveBase>,
    #[account(mut)]
    pub signer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetBurveAdmin<'info> {
    #[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
    pub burve_base: Account<'info, BurveBase>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetBurveTax<'info> {
    #[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
    pub burve_base: Account<'info, BurveBase>,
    pub admin: Signer<'info>,
}
