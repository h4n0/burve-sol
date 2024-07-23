use std::mem::size_of;

use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
         token_metadata_initialize, Mint,
        Token2022, TokenAccount, TokenMetadataInitialize,
    },
};


use crate::Errors;

use crate::{
    update_account_lamports_to_minimum_balance,  MINT_ACCOUNT_SEED,
    PROJECT_METADATA_SEED,
};

const MAX_PLATFORM_TAX_RATE: u16 = 5000;


#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, PartialEq)]
pub enum BondingCurveType {
	Linear{a:u64, b:u64},
	Exponential{a:u64, b:u64},
}

#[account]
pub struct ProjectMetadata {
    pub admin: Pubkey,
	pub treasury: Pubkey,
	pub symbol: String,
    pub mint_tax: u16,
    pub burn_tax: u16,
	pub raising_token: Option<Pubkey>,
	pub bonding_curve_type: BondingCurveType,
	pub burve_tax_counter: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateProjectArgs {
	// Token metadata
    pub name: String,
    pub symbol: String,
    pub uri: String,
	// Project related metadata
	pub admin: Pubkey,
	pub treasury: Pubkey,
	pub mint_tax: u16,
	pub burn_tax: u16,
	pub bonding_curve_type: BondingCurveType,
}

#[derive(Accounts)]
#[instruction(args: CreateProjectArgs)]
pub struct CreateProject<'info> {
    #[account(
		init, 
		payer = payer, 
		space = size_of::<ProjectMetadata>() + 8, 
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
    pub project_metadata: Account<'info, ProjectMetadata>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account()]
	pub raising_token: Option<InterfaceAccount<'info, Mint>>,
	#[account(
		init,
		payer = payer,
		//space = size_of::<TokenAccount>() + 8,
		seeds = [b"vault", mint.key().as_ref()],
		bump,
		token::mint = raising_token,
		token::token_program = token_program,
		token::authority = mint,
	)]
	pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
		// CHECK: mint account is created with the symbol as the seed
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = mint,
        mint::freeze_authority = mint,
        extensions::metadata_pointer::authority = mint,
        extensions::metadata_pointer::metadata_address = mint,
        extensions::group_member_pointer::authority = mint,
        extensions::group_member_pointer::member_address = mint,
        extensions::transfer_hook::authority = mint,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::close_authority::authority = mint,
        extensions::permanent_delegate::delegate = mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    // #[account(
    //     init,
    //     payer = payer,
    //     associated_token::token_program = token_program,
    //     associated_token::mint = mint,
    //     associated_token::authority = receiver,
    // )]
    // pub mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: This account's data is a buffer of TLV data
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> CreateProject<'info> {
    fn initialize_token_metadata(
        &self,
        name: String,
        symbol: String,
        uri: String,
    ) -> ProgramResult {
        let cpi_accounts = TokenMetadataInitialize {
            token_program_id: self.token_program.to_account_info(),
            mint: self.mint.to_account_info(),
            metadata: self.mint.to_account_info(), // metadata account is the mint, since data is stored in mint
            mint_authority: self.mint.to_account_info(),
            update_authority: self.mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        token_metadata_initialize(cpi_ctx, name, symbol, uri)?;
        Ok(())
    }
}

pub fn create_project(
    ctx: Context<CreateProject>,
    args: CreateProjectArgs,
) -> Result<()> {

	require!(args.mint_tax <= MAX_PLATFORM_TAX_RATE, Errors::TaxRateNotValid);
	require!(args.burn_tax <= MAX_PLATFORM_TAX_RATE, Errors::TaxRateNotValid);

	ctx.accounts.project_metadata.admin = args.admin;
	ctx.accounts.project_metadata.treasury = args.treasury;
	ctx.accounts.project_metadata.symbol = args.symbol.clone();
    ctx.accounts.project_metadata.mint_tax = args.mint_tax;
    ctx.accounts.project_metadata.burn_tax = args.burn_tax;
	ctx.accounts.project_metadata.raising_token = ctx.accounts.raising_token.as_ref().map_or(None, |mint| Some(mint.key()));
	ctx.accounts.project_metadata.bonding_curve_type = args.bonding_curve_type;
	ctx.accounts.project_metadata.burve_tax_counter = 0;

    //let vault = &mut ctx.accounts.vault; 
//	vault.mint = ctx.accounts.mint.key();

    ctx.accounts.initialize_token_metadata(
        args.name.clone(),
        args.symbol.clone(),
        args.uri.clone(),
    )?;

    ctx.accounts.mint.reload()?;

    update_account_lamports_to_minimum_balance(
        ctx.accounts.project_metadata.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;
    update_account_lamports_to_minimum_balance(
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;
    update_account_lamports_to_minimum_balance(
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct CheckMintExtensionConstraints<'info> {
    #[account(mut)]
    /// CHECK: can be any account
    pub authority: Signer<'info>,
    #[account(
        extensions::metadata_pointer::authority = authority,
        extensions::metadata_pointer::metadata_address = mint,
        extensions::group_member_pointer::authority = authority,
        extensions::group_member_pointer::member_address = mint,
        extensions::transfer_hook::authority = authority,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::close_authority::authority = authority,
        extensions::permanent_delegate::delegate = authority,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
}

#[derive(Accounts)]
pub struct SetProjectAdmin<'info> {
	#[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	pub admin: Signer<'info>,
}

pub fn set_project_admin(ctx: Context<SetProjectAdmin>, new_admin: Pubkey) -> Result<()> {
	ctx.accounts.project_metadata.admin = new_admin;
	Ok(())
}

#[derive(Accounts)]
pub struct SetProjectTreasury<'info> {
	#[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	pub admin: Signer<'info>,
}

pub fn set_project_treasury(ctx: Context<SetProjectTreasury>, new_treasury: Pubkey) -> Result<()> {
	ctx.accounts.project_metadata.treasury = new_treasury;
	Ok(())
}

#[derive(Accounts)]
pub struct SetProjectTax<'info> {
	#[account(mut, has_one = admin @ Errors::SignerIsNotAdmin)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	pub admin: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SetProjectTaxArgs {
	pub new_mint_tax: u16,
	pub new_burn_tax: u16,
}

pub fn set_project_tax(ctx: Context<SetProjectTax>, args: SetProjectTaxArgs) -> Result<()> {
	require!(args.new_mint_tax <= MAX_PLATFORM_TAX_RATE, Errors::TaxRateNotValid);
	require!(args.new_burn_tax <= MAX_PLATFORM_TAX_RATE, Errors::TaxRateNotValid);
	ctx.accounts.project_metadata.mint_tax = args.new_mint_tax;
	ctx.accounts.project_metadata.burn_tax = args.new_burn_tax;
	Ok(())
}
