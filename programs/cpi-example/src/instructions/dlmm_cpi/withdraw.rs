use crate::assert_eq_admin;
use crate::dlmm::{
    self,
    types::{BinLiquidityReduction, RemainingAccountsInfo},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    /// CHECK: Position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Lb pair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: User token X account
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: User token Y account
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: Reserve x account
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve y account
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Token x mint
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: Token y mint
    pub token_y_mint: UncheckedAccount<'info>,

    pub sender: Signer<'info>,

    /// CHECK: Token x program
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: Token y program
    pub token_y_program: UncheckedAccount<'info>,

    /// CHECK: Memo program
    pub memo_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,
    // Transfer hook and bin arrays in remaining accounts
}

pub fn handle_withdraw(
    ctx: Context<RemoveLiquidity>,
    bin_liquidity_removal: Vec<BinLiquidityReduction>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::RemoveLiquidity2 {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.position.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        user_token_x: ctx.accounts.user_token_x.to_account_info(),
        user_token_y: ctx.accounts.user_token_y.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
        sender: ctx.accounts.sender.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);
    dlmm::cpi::remove_liquidity2(cpi_context, bin_liquidity_removal, remaining_accounts_info)
}

#[derive(Accounts)]
pub struct RemoveLiquidityWithPdaAuthority<'info> {
    /// CHECK: Position account
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Lb pair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: Authority token X account
    #[account(mut)]
    pub authority_token_x: UncheckedAccount<'info>,

    /// CHECK: Authority token Y account
    #[account(mut)]
    pub authority_token_y: UncheckedAccount<'info>,

    /// CHECK: Reserve x account
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve y account
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Token x mint
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: Token y mint
    pub token_y_mint: UncheckedAccount<'info>,

    /// CHECK: Token x program
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: Token y program
    pub token_y_program: UncheckedAccount<'info>,

    /// CHECK: Memo program
    pub memo_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    /// CHECK: Authority
    #[account(
        seeds = [b"authority".as_ref()],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    #[account(
        constraint = assert_eq_admin(admin.key())
    )]
    pub admin: Signer<'info>,
    // Transfer hook and bin arrays in remaining accounts
}

pub fn handle_withdraw_position_with_pda_authority(
    ctx: Context<RemoveLiquidityWithPdaAuthority>,
    bin_liquidity_removal: Vec<BinLiquidityReduction>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::RemoveLiquidity2 {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.position.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        user_token_x: ctx.accounts.authority_token_x.to_account_info(),
        user_token_y: ctx.accounts.authority_token_y.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
        sender: ctx.accounts.authority.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    dlmm::
}
