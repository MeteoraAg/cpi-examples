use crate::dynamic_amm;
use crate::dynamic_amm::accounts::Pool;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;

#[derive(Accounts)]
pub struct DynamicAmmClaimFee<'info> {
    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Pool LP mint
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Lock escrow of user
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    /// CHECK: Lock escrow owner
    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Token vault of lock escrow
    #[account(mut)]
    pub escrow_vault: UncheckedAccount<'info>,

    /// CHECK: Token account of vault A
    #[account(mut)]
    pub a_token_vault: UncheckedAccount<'info>,

    /// CHECK: Token account of vault B
    #[account(mut)]
    pub b_token_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token a. token a of the pool will be deposit / withdraw from this vault account.
    #[account(mut)]
    pub a_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token b. token b of the pool will be deposit / withdraw from this vault account.
    #[account(mut)]
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    #[account(mut)]
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    #[account(mut)]
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault a
    #[account(mut)]
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault b
    #[account(mut)]
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token A account. Used to receive fee
    pub user_a_token: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: User token B account. Used to receive fee
    pub user_b_token: UncheckedAccount<'info>,

    /// CHECK: Token program
    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Dynamic AMM
    #[account(
        address = dynamic_amm::ID
    )]
    pub dynamic_amm: UncheckedAccount<'info>,

    /// CHECK: Dynamic vault
    pub dynamic_vault: UncheckedAccount<'info>,
}

/// Claims the fee for a user from the locked liquidity in the pool.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.

pub fn handle_claim_fee(ctx: Context<DynamicAmmClaimFee>) -> Result<()> {
    let accounts = dynamic_amm::cpi::accounts::ClaimFee {
        pool: ctx.accounts.pool.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        // Unused anymore, but still remained for compatibility. Passing escrow_vault can save 1 account
        source_tokens: ctx.accounts.escrow_vault.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        user_a_token: ctx.accounts.user_a_token.to_account_info(),
        user_b_token: ctx.accounts.user_b_token.to_account_info(),
        vault_program: ctx.accounts.dynamic_vault.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
        b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dynamic_amm.to_account_info(), accounts);

    // Claim max fee
    dynamic_amm::cpi::claim_fee(cpi_context, u64::MAX)
}

#[derive(Accounts)]
pub struct DynamicAmmClaimFeePdaCreator<'info> {
    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Pool LP mint
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Pool creator authority. PDA.
    #[account(
        mut,
        seeds = [b"creator"],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Lock escrow of creator PDA
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    /// CHECK: Only admin can claim fee for creator PDA.
    #[account(
        constraint = crate::assert_eq_admin(cpi_example_admin.key())
    )]
    pub cpi_example_admin: Signer<'info>,

    /// CHECK: Token vault of lock escrow
    #[account(mut)]
    pub escrow_vault: UncheckedAccount<'info>,

    /// CHECK: Token account of vault A
    #[account(mut)]
    pub a_token_vault: UncheckedAccount<'info>,

    /// CHECK: Token account of vault B
    #[account(mut)]
    pub b_token_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token a. token a of the pool will be deposit / withdraw from this vault account.
    #[account(mut)]
    pub a_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token b. token b of the pool will be deposit / withdraw from this vault account.
    #[account(mut)]
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    #[account(mut)]
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    #[account(mut)]
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault a
    #[account(mut)]
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault b
    #[account(mut)]
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Creator token A account. Used to receive fee
    pub creator_a_token: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Creator token B account. Used to receive fee
    pub creator_b_token: UncheckedAccount<'info>,

    /// CHECK: Token program
    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Dynamic AMM
    #[account(
        address = dynamic_amm::ID
    )]
    pub dynamic_amm: UncheckedAccount<'info>,

    /// CHECK: Dynamic vault
    pub dynamic_vault: UncheckedAccount<'info>,
}

/// Claims fee for creator PDA.
///
/// This instruction is used to claim fee for creator PDA. The claimed fee will be hold by creator PDA.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_claim_fee_pda_creator(ctx: Context<DynamicAmmClaimFeePdaCreator>) -> Result<()> {
    let creator_a_token_key = get_associated_token_address_with_program_id(
        &ctx.accounts.creator_authority.key(),
        &ctx.accounts.pool.token_a_mint,
        &ctx.accounts.token_program.key(),
    );

    let creator_b_token_key = get_associated_token_address_with_program_id(
        &ctx.accounts.creator_authority.key(),
        &ctx.accounts.pool.token_b_mint,
        &ctx.accounts.token_program.key(),
    );

    assert_eq!(
        creator_a_token_key,
        ctx.accounts.creator_a_token.key(),
        "Invalid creator_a_token"
    );
    assert_eq!(
        creator_b_token_key,
        ctx.accounts.creator_b_token.key(),
        "Invalid creator_b_token"
    );

    let accounts = dynamic_amm::cpi::accounts::ClaimFee {
        pool: ctx.accounts.pool.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        // Unused anymore, but still remained for compatibility. Passing escrow_vault can save 1 account
        source_tokens: ctx.accounts.escrow_vault.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        user_a_token: ctx.accounts.creator_a_token.to_account_info(),
        user_b_token: ctx.accounts.creator_b_token.to_account_info(),
        vault_program: ctx.accounts.dynamic_vault.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
        b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
    };

    let seeds = [b"creator".as_ref(), &[ctx.bumps.creator_authority]];

    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.dynamic_amm.to_account_info(),
        accounts,
        signer_seeds,
    );

    // Claim max fee
    dynamic_amm::cpi::claim_fee(cpi_context, u64::MAX)
}
