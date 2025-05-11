use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Transfer},
    token_interface::{TokenAccount, TokenInterface},
};

use crate::{
    assert_eq_admin,
    dlmm::{
        self,
        accounts::PositionV2,
        types::{LiquidityParameterByStrategy, RemainingAccountsInfo},
    },
    utils::deserialize_zc_account_workaround,
};

fn validate_position_owner<'a, 'info>(
    position: &'a UncheckedAccount<'info>,
    owner: Pubkey,
) -> Result<()> {
    let position_state: PositionV2 = deserialize_zc_account_workaround(position)?;
    assert_eq!(position_state.owner, owner);
    Ok(())
}

#[derive(Accounts)]
pub struct DlmmDeposit<'info> {
    /// CHECK: Position account. Owned by user
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: LBPair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: User token X
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: User token Y
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
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

    pub user: Signer<'info>,

    pub dlmm_event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>, // Token transfer hook accounts + bin arrays accounts in remaining accounts
}

/// Deposit liquidity into a DLMM pool, given a liquidity parameter and strategy.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `liquidity_parameter` - The liquidity parameter, which is a struct that contains the parameters for the strategy.
/// * `remaining_account_infos` - A struct that contains a vector of `AccountInfo`s, which are the accounts that are needed for the strategy.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_deposit<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDeposit<'info>>,
    liquidity_parameter: LiquidityParameterByStrategy,
    remaining_account_infos: RemainingAccountsInfo,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::AddLiquidityByStrategy2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
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
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        sender: ctx.accounts.user.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::add_liquidity_by_strategy2(cpi_ctx, liquidity_parameter, remaining_account_infos)
}

#[derive(Accounts)]
// For the sake of example not to embed too many logic, we use position pda seeds to verify position actual owner
#[instruction(position_index: i64)]
pub struct DlmmDepositToPositionPda<'info> {
    /// CHECK: Position account. Owned by user
    #[account(
        mut,
         seeds = [
            b"position",
            lb_pair.key().as_ref(),
            position_index.to_le_bytes().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: LBPair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: User token X
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: User token Y
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
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

    pub user: Signer<'info>,

    pub dlmm_event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>, // Token transfer hook accounts + bin arrays accounts in remaining accounts
}

/// Deposit liquidity into a DLMM pool where position is owned by authority PDA, given a liquidity parameter and strategy.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `liquidity_parameter` - The liquidity parameter, which is a struct that contains the parameters for the strategy.
/// * `remaining_account_infos` - A struct that contains a vector of `AccountInfo`s, which are the accounts that are needed for the strategy.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_deposit_to_position_pda<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDepositToPositionPda<'info>>,
    _position_index: i64,
    liquidity_parameter: LiquidityParameterByStrategy,
    remaining_account_infos: RemainingAccountsInfo,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::AddLiquidityByStrategy2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
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
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        sender: ctx.accounts.user.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::add_liquidity_by_strategy2(cpi_ctx, liquidity_parameter, remaining_account_infos)
}

#[derive(Accounts)]
pub struct DlmmDepositToPositionWithPdaAuthority<'info> {
    /// CHECK: Position account. Owned by authority PDA
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: LBPair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: Authority token X
    #[account(mut)]
    pub authority_token_x: UncheckedAccount<'info>,

    /// CHECK: Authority token Y
    #[account(mut)]
    pub authority_token_y: UncheckedAccount<'info>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
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

    /// CHECK: PDA authority
    #[account(
        seeds = [
            b"authority".as_ref()
        ],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        constraint = assert_eq_admin(admin.key())
    )]
    pub admin: Signer<'info>,

    pub dlmm_event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>, // Token transfer hook accounts + bin arrays accounts in remaining accounts
}

/// Deposit liquidity into a DLMM pool where position owner is PDA by PDA authority
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `liquidity_parameter` - The liquidity parameter, which is a struct that contains the parameters for the strategy.
/// * `remaining_account_infos` - A struct that contains a vector of `AccountInfo`s, which are the accounts that are needed for the strategy.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_deposit_with_pda_authority<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDepositToPositionWithPdaAuthority<'info>>,
    liquidity_parameter: LiquidityParameterByStrategy,
    remaining_account_infos: RemainingAccountsInfo,
) -> Result<()> {
    validate_position_owner(&ctx.accounts.position, ctx.accounts.authority.key())?;

    let accounts = dlmm::cpi::accounts::AddLiquidityByStrategy2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
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
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        sender: ctx.accounts.authority.to_account_info(),
    };

    let seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::add_liquidity_by_strategy2(cpi_ctx, liquidity_parameter, remaining_account_infos)
}

#[derive(Accounts)]
#[instruction(position_index: i64)]
pub struct DlmmDepositToPdaPositionWithPdaAuthority<'info> {
    /// CHECK: Position account. Owned by authority PDA, indexed by user
    #[account(
        mut,
          seeds = [
            b"position",
            lb_pair.key().as_ref(),
            position_index.to_le_bytes().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: LBPair account
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array bitmap extension account
    #[account(mut)]
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    /// CHECK: User token X
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: User token Y
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: User
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Authority token X
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_x_mint,
        associated_token::authority = authority,
        associated_token::token_program = token_x_program
    )]
    pub authority_token_x: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Authority token Y
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_y_mint,
        associated_token::authority = authority,
        associated_token::token_program = token_y_program
    )]
    pub authority_token_y: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Reserve X
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Reserve Y
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Token x mint
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: Token y mint
    pub token_y_mint: UncheckedAccount<'info>,

    /// CHECK: Token x program
    pub token_x_program: Interface<'info, TokenInterface>,

    /// CHECK: Token y program
    pub token_y_program: Interface<'info, TokenInterface>,

    /// CHECK: PDA authority
    #[account(
        seeds = [
            b"authority".as_ref()
        ],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    pub dlmm_event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>, // Token transfer hook accounts + bin arrays accounts in remaining accounts

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

/// Deposit liquidity into a DLMM pool by user into a PDA position owned by PDA authority
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `liquidity_parameter` - The liquidity parameter, which is a struct that contains the parameters for the strategy.
/// * `remaining_account_infos` - A struct that contains a vector of `AccountInfo`s, which are the accounts that are needed for the strategy.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_deposit_to_pda_position_with_pda_authority<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDepositToPdaPositionWithPdaAuthority<'info>>,
    _position_index: i64,
    liquidity_parameter: LiquidityParameterByStrategy,
    remaining_account_infos: RemainingAccountsInfo,
) -> Result<()> {
    validate_position_owner(&ctx.accounts.position, ctx.accounts.authority.key())?;

    // 1. Transfer required tokens into authority token account
    transfer(
        CpiContext::new(
            ctx.accounts.token_x_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_x.to_account_info(),
                to: ctx.accounts.authority_token_x.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        liquidity_parameter.amount_x,
    )?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_y_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_y.to_account_info(),
                to: ctx.accounts.authority_token_y.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        liquidity_parameter.amount_y,
    )?;

    ctx.accounts.authority_token_x.reload()?;
    ctx.accounts.authority_token_y.reload()?;

    let before_authority_token_x_amount = ctx.accounts.authority_token_x.amount;
    let before_authority_token_y_amount = ctx.accounts.authority_token_y.amount;

    // 2. Deposit
    let accounts = dlmm::cpi::accounts::AddLiquidityByStrategy2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
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
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        sender: ctx.accounts.authority.to_account_info(),
    };

    let seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::add_liquidity_by_strategy2(cpi_ctx, liquidity_parameter, remaining_account_infos)?;

    ctx.accounts.authority_token_x.reload()?;
    ctx.accounts.authority_token_y.reload()?;

    let after_authority_token_x_amount = ctx.accounts.authority_token_x.amount;
    let after_authority_token_y_amount = ctx.accounts.authority_token_y.amount;

    let actual_consumed_amount_x = before_authority_token_x_amount
        .checked_sub(after_authority_token_x_amount)
        .unwrap();

    let actual_consumed_amount_y = before_authority_token_y_amount
        .checked_sub(after_authority_token_y_amount)
        .unwrap();

    let unused_token_x = liquidity_parameter
        .amount_x
        .checked_sub(actual_consumed_amount_x)
        .unwrap();

    let unused_token_y = liquidity_parameter
        .amount_y
        .checked_sub(actual_consumed_amount_y)
        .unwrap();

    let seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    if unused_token_x > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_x_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_x.to_account_info(),
                    to: ctx.accounts.user_token_x.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                signer_seeds,
            ),
            unused_token_x,
        )?;
    }

    if unused_token_y > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_y_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_y.to_account_info(),
                    to: ctx.accounts.user_token_y.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                signer_seeds,
            ),
            unused_token_y,
        )?;
    }

    Ok(())
}

// TODO: More advanced example
// #[derive(Accounts)]
// pub struct InitAndDepositSingleSide<'info> {}
