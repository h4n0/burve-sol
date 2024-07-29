use anchor_lang::{ prelude::*,  system_program};

use anchor_spl::{
    associated_token::AssociatedToken, token_2022::{mint_to,burn,
    Burn, MintTo}, token_interface::{
         Mint,
        Token2022, TokenAccount, 
    }
};
use anchor_spl::token_interface::{transfer_checked, TransferChecked};


use crate::{ calculations::*, BurveBase, Errors, MAX_TAX_RATE_DENOMINATOR};

use crate::{
	 MINT_ACCOUNT_SEED, PROJECT_METADATA_SEED
};

use crate::token_factory::*;


struct EstimateMintResult {
	calculated_receiving_amount: u64,
	actual_paid_amount: u64,
	project_fee: u64,
	burve_fee: u64,
}


#[inline(never)]
fn estimate_mint_amount_from_bonding_curve(
	bonding_curve_type: BondingCurveType,
	paid_amount: u64,
	mint_supply: u64,
	burve_tax: u16,
	project_tax: u16,
) -> EstimateMintResult {

	let project_fee = paid_amount * project_tax as u64 / MAX_TAX_RATE_DENOMINATOR ;
	let burve_fee = paid_amount * burve_tax as u64/ MAX_TAX_RATE_DENOMINATOR;

	let actual_paid_amount = paid_amount - project_fee - burve_fee;

	let (calculated_receiving_amount, _) = 
	match bonding_curve_type {
		BondingCurveType::Linear { a, b } => {
			 LinearMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
				actual_paid_amount,	
				mint_supply,
				crate::Parameters { a, b },
			)
		}
		BondingCurveType::Exponential { a, b } => {
			 ExpMixedBondingSwap::calculate_mint_amount_from_bonding_curve(
				actual_paid_amount,	
				mint_supply,
				crate::Parameters { a, b },
			)
		}

	};

	EstimateMintResult {
		calculated_receiving_amount,
		actual_paid_amount,
		project_fee,
		burve_fee,
	}
}

pub struct EstimateBurnResult {
	actual_received_amount: u64,
	project_fee: u64,
	burve_fee: u64,
}

#[inline(never)]
fn estimate_burn_amount_from_bonding_curve(
	bonding_curve_type: BondingCurveType,
	burning_amount: u64,
	mint_supply: u64,
	burve_tax: u16,
	project_tax: u16,
) -> EstimateBurnResult {


	let (calculated_receiving_amount, _) = 
		match bonding_curve_type {
			BondingCurveType::Linear { a, b } => {
				 LinearMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
					burning_amount,	
					mint_supply,
					crate::Parameters { a, b },
				)
			}
			BondingCurveType::Exponential { a, b } => {
				 ExpMixedBondingSwap::calculate_burn_amount_from_bonding_curve(
					burning_amount,	
					mint_supply,
					crate::Parameters { a, b },
				)
			}
		};

	let project_fee = calculated_receiving_amount * project_tax as u64 / MAX_TAX_RATE_DENOMINATOR;
	let burve_fee = calculated_receiving_amount * burve_tax as u64/ MAX_TAX_RATE_DENOMINATOR;

	let actual_received_amount = calculated_receiving_amount - project_fee - burve_fee;

	EstimateBurnResult {
		actual_received_amount,
		project_fee,
		burve_fee,
	}
}


#[derive(Accounts)]
#[instruction(args: MintTokenWithSPLArgs)]
pub struct MintTokenWithSPL<'info> {
    #[account(
		seeds = [b"burve"], 
		bump 
	)]
    pub burve_base: Box<Account<'info, BurveBase>>,
	#[account(
		constraint = project_metadata.raising_token == Some(raising_token.key()),
		constraint = project_metadata.symbol == args.symbol,
		constraint = project_metadata.treasury == project_treasury.key(),
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
    pub project_metadata: Box<Account<'info, ProjectMetadata>>,
	#[account(
		mut,
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
    pub mint: InterfaceAccount<'info, Mint>,
	#[account()]
	pub raising_token: Box<InterfaceAccount<'info, Mint>>, 
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump,
		token::mint = raising_token,
		token::token_program = token_program
	)]
	pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
	#[account(
		mut,
		token::mint = raising_token,
		token::token_program = token_program,
	)]
	pub project_treasury: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub signer: Signer<'info>,
	#[account(
		mut,
		token::mint = raising_token,
		token::token_program = token_program,
		token::authority = signer,
	)]
    pub from_ata: Box<InterfaceAccount<'info, TokenAccount>>,
	#[account(
		init_if_needed,
		payer = signer,
		associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = signer,
	)]
	pub mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct MintTokenWithSPLArgs {
	pub amount: u64,
	pub symbol: String,
	pub min_receive: u64,
}

pub fn route_mint_token_with_spl(
	ctx: Context<MintTokenWithSPL>,
	args: MintTokenWithSPLArgs,
) -> Result<()> {
	// Calculate how many tokens to mint
	let estimate_res = estimate_mint_amount_from_bonding_curve(
		ctx.accounts.project_metadata.bonding_curve_type.clone(),
		args.amount,
		ctx.accounts.mint.supply,
		ctx.accounts.burve_base.mint_tax,
		ctx.accounts.project_metadata.mint_tax,
	);

	assert!(estimate_res.calculated_receiving_amount >= args.min_receive, "min_receive not met");

	// Transfer SPL token to vault
	let token_program = ctx.accounts.token_program.to_account_info();
	let accounts = TransferChecked {
		from: ctx.accounts.from_ata.to_account_info().clone(),
		to: ctx.accounts.vault.to_account_info().clone(),
		authority: ctx.accounts.signer.to_account_info().clone(),
		mint: ctx.accounts.raising_token.to_account_info().clone(),
	};

	//let bump = &[ctx.bumps.mint];
	//let seeds: &[&[u8]] = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), bump];
	//let signer_seeds = &[&seeds[..]];
	let cpi_ctx = CpiContext::new(token_program, accounts);

	transfer_checked(cpi_ctx, estimate_res.actual_paid_amount + estimate_res.burve_fee, ctx.accounts.raising_token.decimals)?;

	ctx.accounts.project_metadata.burve_tax_counter += estimate_res.burve_fee;

	// Transfer project tax to project treasury
	let token_program = ctx.accounts.token_program.to_account_info();
	let accounts = TransferChecked {
		from: ctx.accounts.from_ata.to_account_info().clone(),
		to: ctx.accounts.project_treasury.to_account_info().clone(),
		authority: ctx.accounts.signer.to_account_info().clone(),
		mint: ctx.accounts.raising_token.to_account_info().clone(),
	};

	let cpi_ctx = CpiContext::new(token_program, accounts);

	transfer_checked(cpi_ctx, estimate_res.project_fee, ctx.accounts.raising_token.decimals)?;

	// Mint SPL token to mint token account
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];

	mint_to(
		CpiContext::new_with_signer(
			ctx.accounts.token_program.to_account_info(),
			MintTo {
				authority: ctx.accounts.mint.to_account_info(),
				to: ctx.accounts.mint_token_account.to_account_info(),
				mint: ctx.accounts.mint.to_account_info(),
			},
			&signer,
		),
		estimate_res.calculated_receiving_amount
	)?;

	Ok(())
}

#[derive(Accounts)]
#[instruction(args: BurnTokenToSPLArgs)]
pub struct BurnTokenToSPL<'info> {
	#[account(
		seeds = [b"burve"], 
		bump 
	)]
	pub burve_base: Account<'info, BurveBase>,
	#[account(
		constraint = project_metadata.raising_token == Some(raising_token.key()),
		constraint = project_metadata.symbol == args.symbol,
		constraint = project_metadata.treasury == project_treasury.key(),
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	#[account(
		mut,
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
	pub mint: InterfaceAccount<'info, Mint>,
	#[account()]
	pub raising_token: InterfaceAccount<'info, Mint>, 
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump
	)]
	pub vault: InterfaceAccount<'info, TokenAccount>,
	#[account(
		mut,
		token::mint = raising_token,
		token::token_program = token_program,
	)]
	pub project_treasury: InterfaceAccount<'info, TokenAccount>,
	#[account(mut)]
	pub signer: Signer<'info>,
	#[account(mut)]
	pub to_ata: InterfaceAccount<'info, TokenAccount>,
	#[account(
		mut,
		associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = signer,
	)]
	pub burn_token_account: InterfaceAccount<'info, TokenAccount>,
	pub system_program: Program<'info, System>,
	pub token_program: Program<'info, Token2022>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct BurnTokenToSPLArgs {
	pub amount: u64,
	pub symbol: String,
	pub min_receive: u64,
}

pub fn route_burn_token_to_spl(
	ctx: Context<BurnTokenToSPL>,
	args: BurnTokenToSPLArgs,
) -> Result<()> {
	// Calculate how many tokens to mint
	let estimate_res = estimate_burn_amount_from_bonding_curve(
		ctx.accounts.project_metadata.bonding_curve_type.clone(),
		args.amount,
		ctx.accounts.mint.supply,
		ctx.accounts.burve_base.mint_tax,
		ctx.accounts.project_metadata.mint_tax,
	);

	assert!(estimate_res.actual_received_amount >= args.min_receive, "min_receive not met");

	// Burn tokens
	burn(
		CpiContext::new(
			ctx.accounts.token_program.to_account_info(),
			Burn {
				authority: ctx.accounts.signer.to_account_info(),
				from: ctx.accounts.burn_token_account.to_account_info(),
				mint: ctx.accounts.mint.to_account_info(),
			},
		),
		args.amount
	)?;


	// Transfer SPL token from vault
	let token_program = ctx.accounts.token_program.to_account_info();
	let accounts = TransferChecked {
		from: ctx.accounts.vault.to_account_info().clone(),
		to: ctx.accounts.to_ata.to_account_info().clone(),
		authority: ctx.accounts.mint.to_account_info().clone(),
		mint: ctx.accounts.raising_token.to_account_info().clone(),
	};
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];

	let cpi_ctx = CpiContext::new_with_signer(token_program, accounts, &signer);

	transfer_checked(cpi_ctx, estimate_res.actual_received_amount, ctx.accounts.mint.decimals)?;

	// Transfer project tax to project treasury
	let token_program = ctx.accounts.token_program.to_account_info();
	let accounts = TransferChecked {
		from: ctx.accounts.vault.to_account_info().clone(),
		to: ctx.accounts.project_treasury.to_account_info().clone(),
		authority: ctx.accounts.mint.to_account_info().clone(),
		mint: ctx.accounts.raising_token.to_account_info().clone(),
	};

	let cpi_ctx = CpiContext::new_with_signer(token_program, accounts, &signer);

	transfer_checked(cpi_ctx, estimate_res.project_fee, ctx.accounts.raising_token.decimals)?;

	// Increment burve tax counter
	ctx.accounts.project_metadata.burve_tax_counter += estimate_res.burve_fee;

	Ok(())
}


#[derive(Accounts)]
#[instruction(args: MintTokenWithSOLArgs)]
pub struct MintTokenWithSOL<'info> {
    #[account(
		seeds = [b"burve"], 
		bump 
	)]
    pub burve_base: Box<Account<'info, BurveBase>>,
	#[account(
		constraint = project_metadata.raising_token == None,
		constraint = project_metadata.symbol == args.symbol,
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
    pub project_metadata: Box<Account<'info, ProjectMetadata>>,
	#[account(
		mut,
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
    pub mint: Box<InterfaceAccount<'info, Mint>>,
	#[account(mut)]
	pub from: Signer<'info>,
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump
	)]
	pub vault: SystemAccount<'info>,
	#[account(
		mut,
	)]
	pub project_treasury: SystemAccount<'info>,
	#[account(
		init_if_needed,
		payer = from,
		associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = from,
	)]
	pub mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	pub associated_token_program: Program<'info, AssociatedToken>,
	pub token_program: Program<'info, Token2022>,
	pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct MintTokenWithSOLArgs {
	pub amount: u64,
	pub symbol: String,
	pub min_receive: u64,
}

pub fn route_mint_token_with_sol(ctx: Context<MintTokenWithSOL>, args: MintTokenWithSOLArgs) -> Result<()> {
	// Calculate how many tokens to mint
	let estimate_res = estimate_mint_amount_from_bonding_curve(
		ctx.accounts.project_metadata.bonding_curve_type.clone(),
		args.amount,
		ctx.accounts.mint.supply,
		ctx.accounts.burve_base.mint_tax,
		ctx.accounts.project_metadata.mint_tax,
	);

	assert!(estimate_res.calculated_receiving_amount >= args.min_receive, "min_receive not met");

	// Transfer SOL token to vault
	let cpi_ctx = CpiContext::new(
	ctx.accounts.system_program.to_account_info(), 
	system_program::Transfer{
		from: ctx.accounts.from.to_account_info(),
		to: ctx.accounts.vault.to_account_info(),
	});

	system_program::transfer(cpi_ctx, estimate_res.actual_paid_amount + estimate_res.burve_fee)?;

	ctx.accounts.project_metadata.burve_tax_counter += estimate_res.burve_fee;

	// Transfer project tax to project treasury
	let cpi_ctx = CpiContext::new(
	ctx.accounts.system_program.to_account_info(),
	system_program::Transfer{
		from: ctx.accounts.from.to_account_info(),
		to: ctx.accounts.project_treasury.to_account_info(),
	});

	system_program::transfer(cpi_ctx, estimate_res.project_fee)?;

	// Mint the project SPL token to mint token account
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];

	mint_to(
		CpiContext::new_with_signer(
			ctx.accounts.token_program.to_account_info(),
			MintTo {
				authority: ctx.accounts.mint.to_account_info(),
				to: ctx.accounts.mint_token_account.to_account_info(),
				mint: ctx.accounts.mint.to_account_info(),
			},
			&signer,
		),
		estimate_res.calculated_receiving_amount
	)?;

	Ok(())
}

#[derive(Accounts)]
#[instruction(args: BurnTokenToSOLArgs)]
pub struct BurnTokenToSOL<'info> {
	#[account(
		seeds = [b"burve"], 
		bump 
	)]
	pub burve_base: Account<'info, BurveBase>,
	#[account(
		constraint = project_metadata.raising_token == None,
		constraint = project_metadata.symbol == args.symbol,
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	#[account(
		mut,
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
	pub mint: InterfaceAccount<'info, Mint>,
	#[account(mut)]
	pub from: Signer<'info>,
	#[account(
		mut,
		token::token_program = token_program,
		token::mint = mint,
		token::authority = from,
	)]
	pub burn_token_account: InterfaceAccount<'info, TokenAccount>,
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump
	)]
	pub vault: SystemAccount<'info>,
	#[account(
		mut,
	)]
	pub project_treasury: SystemAccount<'info>,
	pub token_program: Program<'info, Token2022>,
	pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct BurnTokenToSOLArgs {
	pub amount: u64,
	pub symbol: String,
	pub min_receive: u64,
}

pub fn route_burn_token_to_sol(ctx: Context<BurnTokenToSOL>, args: BurnTokenToSOLArgs) -> Result<()> {
	// Calculate how many tokens to burn
	let estimate_res = estimate_burn_amount_from_bonding_curve(
		ctx.accounts.project_metadata.bonding_curve_type.clone(),
		args.amount,
		ctx.accounts.mint.supply,
		ctx.accounts.burve_base.mint_tax,
		ctx.accounts.project_metadata.mint_tax,
	);

	assert!(estimate_res.actual_received_amount >= args.min_receive, "min_receive not met");

	// Burn tokens
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];
	burn(
		CpiContext::new_with_signer(
			ctx.accounts.token_program.to_account_info(),
			Burn {
				authority: ctx.accounts.mint.to_account_info(),
				from: ctx.accounts.burn_token_account.to_account_info(),
				mint: ctx.accounts.mint.to_account_info(),
			},
			&signer
		),
		args.amount
	)?;

	// Transfer SOL token from vault
	let mint_pubkey = ctx.accounts.mint.to_account_info().key;
	let seeds = &[b"vault", mint_pubkey.as_ref(), &[ctx.bumps.vault]];
	let signer = [&seeds[..]];
	let cpi_ctx = CpiContext::new_with_signer(
	ctx.accounts.system_program.to_account_info(), 
	system_program::Transfer{
		from: ctx.accounts.vault.to_account_info(),
		to: ctx.accounts.from.to_account_info(),
	},
	&signer);

	system_program::transfer(cpi_ctx, estimate_res.actual_received_amount)?;

	// Transfer project tax to project treasury
	let cpi_ctx = CpiContext::new_with_signer(
	ctx.accounts.system_program.to_account_info(),
	system_program::Transfer{
		from: ctx.accounts.vault.to_account_info(),
		to: ctx.accounts.project_treasury.to_account_info(),
	}, &signer);

	system_program::transfer(cpi_ctx, estimate_res.project_fee)?;

	// Increment burve tax counter
	ctx.accounts.project_metadata.burve_tax_counter += estimate_res.burve_fee;

	Ok(())
}

#[derive(Accounts)]
#[instruction(args: ClaimBurveSPLTaxArgs)]
pub struct ClaimBurveSPLTax<'info> {
	#[account(
		has_one = admin @ Errors::SignerIsNotAdmin,
		seeds = [b"burve"], 
		bump 
	)]
	pub burve_base: Account<'info, BurveBase>,
	#[account(
		constraint = project_metadata.raising_token == Some(raising_token.key()),
		constraint = project_metadata.symbol == args.symbol,
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	#[account(
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
	pub mint: InterfaceAccount<'info, Mint>,
	#[account()]
	pub raising_token: InterfaceAccount<'info, Mint>,
	#[account(mut)]
	pub admin: Signer<'info>,
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump
	)]
	pub vault: InterfaceAccount<'info, TokenAccount>,
	#[account(
		init_if_needed,
		payer = admin,
		associated_token::token_program = token_program,
		associated_token::mint = raising_token,
		associated_token::authority = admin,
	)]
	pub burve_treasury: InterfaceAccount<'info, TokenAccount>,
	pub token_program: Program<'info, Token2022>,
	pub associated_token_program: Program<'info, AssociatedToken>,
	pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimBurveSPLTaxArgs {
	pub symbol: String,
}

pub fn route_claim_burve_spl_tax(ctx: Context<ClaimBurveSPLTax>, args: ClaimBurveSPLTaxArgs) -> Result<()> {
	let burve_tax = ctx.accounts.project_metadata.burve_tax_counter;

	// Transfer burve tax to burve treasury
	let token_program = ctx.accounts.token_program.to_account_info();
	let accounts = TransferChecked {
		from: ctx.accounts.vault.to_account_info().clone(),
		to: ctx.accounts.burve_treasury.to_account_info().clone(),
		authority: ctx.accounts.mint.to_account_info().clone(),
		mint: ctx.accounts.raising_token.to_account_info().clone(),
	};
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];

	let cpi_ctx = CpiContext::new_with_signer(token_program, accounts, &signer);

	transfer_checked(cpi_ctx, burve_tax, ctx.accounts.raising_token.decimals)?;

	// Reset burve tax counter
	ctx.accounts.project_metadata.burve_tax_counter = 0;

	Ok(())
}

#[derive(Accounts)]
#[instruction(args: ClaimBurveSOLTaxArgs)]
pub struct ClaimBurveSOLTax<'info> {
	#[account(
		has_one = admin @ Errors::SignerIsNotAdmin,
		seeds = [b"burve"], 
		bump 
	)]
	pub burve_base: Account<'info, BurveBase>,
	#[account(
		constraint = project_metadata.raising_token == None,
		constraint = project_metadata.symbol == args.symbol,
		seeds = [PROJECT_METADATA_SEED, mint.key().as_ref() ], 
		bump 
	)]
	pub project_metadata: Account<'info, ProjectMetadata>,
	#[account(
		seeds = [MINT_ACCOUNT_SEED, args.symbol.as_bytes()],
		bump,
	)]  
	pub mint: InterfaceAccount<'info, Mint>,
	#[account()]
	pub raising_token: InterfaceAccount<'info, Mint>,
	#[account(mut)]
	pub admin: Signer<'info>,
	#[account(
		mut,
		seeds = [b"vault", mint.key().as_ref()],
		bump
	)]
	pub vault: InterfaceAccount<'info, TokenAccount>,
	#[account(
		mut,
		seeds = [b"burve-sol-treasury"],
		bump
	)]
	pub burve_treasury: SystemAccount<'info>,
	pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimBurveSOLTaxArgs {
	pub symbol: String,
}

pub fn route_claim_burve_sol_tax(ctx: Context<ClaimBurveSOLTax>, args: ClaimBurveSOLTaxArgs) -> Result<()> {
	let burve_tax = ctx.accounts.project_metadata.burve_tax_counter;

	// Transfer burve tax to burve treasury
	let seeds = &[MINT_ACCOUNT_SEED, args.symbol.as_bytes(), &[ctx.bumps.mint]];
	let signer = [&seeds[..]];
	let cpi_ctx = CpiContext::new_with_signer(
	ctx.accounts.system_program.to_account_info(),
	system_program::Transfer{
		from: ctx.accounts.vault.to_account_info(),
		to: ctx.accounts.burve_treasury.to_account_info(),
	}, &signer);

	system_program::transfer(cpi_ctx, burve_tax)?;

	// Reset burve tax counter
	ctx.accounts.project_metadata.burve_tax_counter = 0;

	Ok(())
}

