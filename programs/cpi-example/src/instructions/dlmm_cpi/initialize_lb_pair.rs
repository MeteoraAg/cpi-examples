use crate::dlmm::{self, types::InitializeLbPair2Params};
use anchor_lang::prelude::*;

use super::utils::fund_creator_authority;

#[derive(Accounts)]
pub struct InitializeLbPair<'info> {
    /// CHECK: Lb pair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: Token mint X
    pub token_mint_x: UncheckedAccount<'info>,

    /// CHECK: Token mint Y
    pub token_mint_y: UncheckedAccount<'info>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Oracle
    #[account(mut)]
    pub oracle: UncheckedAccount<'info>,

    /// CHECK: Preset parameter
    pub preset_parameter: UncheckedAccount<'info>,

    #[account(mut)]
    pub funder: Signer<'info>,

    /// CHECK: Token badge X
    pub token_badge_x: Option<UncheckedAccount<'info>>,

    /// CHECK: Token badge Y
    pub token_badge_y: Option<UncheckedAccount<'info>>,

    /// CHECK: Token program X
    pub token_program_x: UncheckedAccount<'info>,

    /// CHECK: Token program Y
    pub token_program_y: UncheckedAccount<'info>,

    /// CHECK: System program
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    pub damm_event_authority: UncheckedAccount<'info>,
}

/// Initializes a liquidity bin pair (LBP) with the given parameters.
///
/// # Parameters
///
/// * `params` - The parameters for the LBP.
pub fn handle_initialize_lb_pair(
    ctx: Context<InitializeLbPair>,
    params: InitializeLbPair2Params,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializeLbPair2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        token_mint_x: ctx.accounts.token_mint_x.to_account_info(),
        token_mint_y: ctx.accounts.token_mint_y.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        oracle: ctx.accounts.oracle.to_account_info(),
        preset_parameter: ctx.accounts.preset_parameter.to_account_info(),
        funder: ctx.accounts.funder.to_account_info(),
        token_badge_x: ctx
            .accounts
            .token_badge_x
            .as_ref()
            .map(|account| account.to_account_info()),
        token_badge_y: ctx
            .accounts
            .token_badge_y
            .as_ref()
            .map(|account| account.to_account_info()),
        token_program_x: ctx.accounts.token_program_x.to_account_info(),
        token_program_y: ctx.accounts.token_program_y.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.damm_event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);
    dlmm::cpi::initialize_lb_pair2(cpi_context, params)
}

#[derive(Accounts)]
pub struct InitializeLbPairWithPdaCreator<'info> {
    /// CHECK: Creator authority
    #[account(
        mut,
        seeds = [b"creator"],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Lb pair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: Token mint X
    pub token_mint_x: UncheckedAccount<'info>,

    /// CHECK: Token mint Y
    pub token_mint_y: UncheckedAccount<'info>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Oracle
    #[account(mut)]
    pub oracle: UncheckedAccount<'info>,

    /// CHECK: Preset parameter
    pub preset_parameter: UncheckedAccount<'info>,

    #[account(mut)]
    pub funder: Signer<'info>,

    /// CHECK: Token badge X
    pub token_badge_x: Option<UncheckedAccount<'info>>,

    /// CHECK: Token badge Y
    pub token_badge_y: Option<UncheckedAccount<'info>>,

    /// CHECK: Token program X
    pub token_program_x: UncheckedAccount<'info>,

    /// CHECK: Token program Y
    pub token_program_y: UncheckedAccount<'info>,

    /// CHECK: System program
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    pub damm_event_authority: UncheckedAccount<'info>,
}

pub fn handle_initialize_lb_pair_with_pda_creator(
    ctx: Context<InitializeLbPairWithPdaCreator>,
    params: InitializeLbPair2Params,
) -> Result<()> {
    fund_creator_authority(
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.funder.to_account_info(),
        ctx.accounts.creator_authority.to_account_info(),
    )?;

    let accounts = dlmm::cpi::accounts::InitializeLbPair2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        token_mint_x: ctx.accounts.token_mint_x.to_account_info(),
        token_mint_y: ctx.accounts.token_mint_y.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        oracle: ctx.accounts.oracle.to_account_info(),
        preset_parameter: ctx.accounts.preset_parameter.to_account_info(),
        funder: ctx.accounts.creator_authority.to_account_info(),
        token_badge_x: ctx
            .accounts
            .token_badge_x
            .as_ref()
            .map(|account| account.to_account_info()),
        token_badge_y: ctx
            .accounts
            .token_badge_y
            .as_ref()
            .map(|account| account.to_account_info()),
        token_program_x: ctx.accounts.token_program_x.to_account_info(),
        token_program_y: ctx.accounts.token_program_y.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        event_authority: ctx.accounts.damm_event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let seeds = [b"creator".as_ref(), &[ctx.bumps.creator_authority]];

    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    );

    dlmm::cpi::initialize_lb_pair2(cpi_context, params)
}
