use crate::dlmm::{self, accounts::LbPair};
use crate::utils::deserialize_zc_account_workaround;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct DlmmSwap<'info> {
    #[account(mut)]
    /// CHECK: The pool account
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array extension account of the pool
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: Reserve account of token X
    pub reserve_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Reserve account of token Y
    pub reserve_y: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token account to sell token
    pub user_token_in: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: User token account to buy token
    pub user_token_out: UncheckedAccount<'info>,

    /// CHECK: Mint account of token X
    pub token_x_mint: UncheckedAccount<'info>,
    /// CHECK: Mint account of token Y
    pub token_y_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Oracle account of the pool
    pub oracle: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Referral fee account
    pub host_fee_in: Option<UncheckedAccount<'info>>,

    /// CHECK: User who's executing the swap
    pub user: Signer<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of mint X
    pub token_x_program: UncheckedAccount<'info>,
    /// CHECK: Token program of mint Y
    pub token_y_program: UncheckedAccount<'info>,
    // Bin arrays need to be passed using remaining accounts
}

/// Executes a DLMM swap
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `amount_in` - The amount of input tokens to be swapped.
/// * `min_amount_out` - The minimum amount of output tokens expected a.k.a slippage
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_swap<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::Swap {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        user_token_in: ctx.accounts.user_token_in.to_account_info(),
        user_token_out: ctx.accounts.user_token_out.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        oracle: ctx.accounts.oracle.to_account_info(),
        host_fee_in: ctx
            .accounts
            .host_fee_in
            .as_ref()
            .map(|account| account.to_account_info()),
        user: ctx.accounts.user.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
    dlmm::cpi::swap(cpi_context, amount_in, min_amount_out)
}

#[derive(Accounts)]
pub struct DlmmSwapWithPdaAuthority<'info> {
    #[account(mut)]
    /// CHECK: The pool account. Please use AccountLoader in production.
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array extension account of the pool
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: Reserve account of token X
    pub reserve_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Reserve account of token Y
    pub reserve_y: UncheckedAccount<'info>,

    #[account(
        seeds = [b"authority".as_ref()],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        token::authority = authority
    )]
    /// CHECK: Authority PDA's token account to sell token
    pub authority_token_in: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = authority
    )]
    /// CHECK: Authority PDA's token account to buy token
    pub authority_token_out: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::token_program = token_x_program
    )]
    /// CHECK: Mint account of token X
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mint::token_program = token_y_program
    )]
    /// CHECK: Mint account of token Y
    pub token_y_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    /// CHECK: Oracle account of the pool
    pub oracle: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Referral fee account
    pub host_fee_in: Option<UncheckedAccount<'info>>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of mint X
    pub token_x_program: UncheckedAccount<'info>,
    /// CHECK: Token program of mint Y
    pub token_y_program: UncheckedAccount<'info>,
    // Bin arrays need to be passed using remaining accounts
}

impl<'info> DlmmSwapWithPdaAuthority<'info> {
    pub fn validate_authority_associated_token_account(&self) -> Result<()> {
        let lb_pair_state: LbPair = deserialize_zc_account_workaround(&self.lb_pair)?;

        assert_eq!(lb_pair_state.token_x_mint, self.token_x_mint.key());
        assert_eq!(lb_pair_state.token_y_mint, self.token_y_mint.key());

        let authority_token_x = get_associated_token_address_with_program_id(
            &self.authority.key(),
            &self.token_x_mint.key(),
            &self.token_x_program.key(),
        );

        let authority_token_y = get_associated_token_address_with_program_id(
            &self.authority.key(),
            &self.token_y_mint.key(),
            &self.token_y_program.key(),
        );

        assert!(self.authority_token_in.key() != self.authority_token_out.key());

        let swap_for_y = self.authority_token_in.mint.eq(&self.token_x_mint.key());

        if swap_for_y {
            assert_eq!(self.authority_token_in.key(), authority_token_x);
            assert_eq!(self.authority_token_out.key(), authority_token_y);
        } else {
            assert_eq!(self.authority_token_in.key(), authority_token_y);
            assert_eq!(self.authority_token_out.key(), authority_token_x);
        }

        Ok(())
    }
}

/// Executes a DLMM swap with PDA authority
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `amount_in` - The amount of input tokens to be swapped.
/// * `min_amount_out` - The minimum amount of output tokens expected a.k.a slippage
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_swap_with_pda_authority<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmSwapWithPdaAuthority<'info>>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    ctx.accounts.validate_authority_associated_token_account()?;

    let accounts = dlmm::cpi::accounts::Swap {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        user_token_in: ctx.accounts.authority_token_in.to_account_info(),
        user_token_out: ctx.accounts.authority_token_out.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        oracle: ctx.accounts.oracle.to_account_info(),
        host_fee_in: ctx
            .accounts
            .host_fee_in
            .as_ref()
            .map(|account| account.to_account_info()),
        user: ctx.accounts.authority.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let seeds = [b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::swap(cpi_context, amount_in, min_amount_out)
}
