use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use m3m3::{InitializeVaultIxArgs, InitializeVaultParams};

#[derive(Accounts)]
pub struct InitializeM3m3Vault<'info> {
    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Lock escrow for m3m3 vault
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    /// CHECK: Pool LP mint
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Payer lp token account
    #[account(mut)]
    pub source_lp_tokens: UncheckedAccount<'info>,

    /// CHECK: Escrow vault
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow,
        payer = payer
    )]
    pub escrow_vault: Box<Account<'info, TokenAccount>>,

    /// Wallet that hold LP tokens and wish to lock to m3m3 vault
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Vault account for token a. token a of the pool will be deposit / withdraw from this vault account.
    pub a_vault: UncheckedAccount<'info>,

    /// CHECK: Vault account for token b. token b of the pool will be deposit / withdraw from this vault account.
    pub b_vault: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub a_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub b_vault_lp: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault a
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: LP token mint of vault b
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    /// CHECK: M3m3 vault
    #[account(mut)]
    pub m3m3_vault: UncheckedAccount<'info>,

    /// CHECK: Stake token vault
    #[account(mut)]
    pub stake_token_vault: UncheckedAccount<'info>,

    /// CHECK: Quote token vault
    #[account(mut)]
    pub quote_token_vault: UncheckedAccount<'info>,

    /// CHECK: Quote token mint
    #[account(mut)]
    pub top_staker_list: UncheckedAccount<'info>,

    /// CHECK: Full balance list
    #[account(mut)]
    pub full_balance_list: UncheckedAccount<'info>,

    /// CHECK: Stake mint
    pub stake_mint: UncheckedAccount<'info>,

    /// CHECK: Quote mint
    pub quote_mint: UncheckedAccount<'info>,

    /// CHECK: M3m3 event authority
    pub m3m3_event_authority: UncheckedAccount<'info>,

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,

    /// CHECK: M3m3 program
    #[account(address = m3m3::ID)]
    pub m3m3_program: UncheckedAccount<'info>,

    /// CHECK: System program
    pub system_program: UncheckedAccount<'info>,

    /// CHECK: Token program.
    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Associated token program
    pub associated_token_program: UncheckedAccount<'info>,
}

/// Initializes a new M3M3 vault.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `max_amount` - The maximum amount of LP token to be deposited.
/// * `vault_params` - The configuration parameters for the m3m3 vault.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_m3m3_vault(
    ctx: Context<InitializeM3m3Vault>,
    max_amount: u64,
    vault_params: InitializeVaultParams,
) -> Result<()> {
    // 1. Initialize lock escrow for m3m3 vault
    let accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.m3m3_vault.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dynamic_amm_program.to_account_info(), accounts);
    dynamic_amm::cpi::create_lock_escrow(cpi_context)?;

    // 2. Lock user LP to m3m3 lock escrow
    let accounts = dynamic_amm::cpi::accounts::Lock {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        owner: ctx.accounts.payer.to_account_info(),
        source_tokens: ctx.accounts.source_lp_tokens.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dynamic_amm_program.to_account_info(), accounts);
    dynamic_amm::cpi::lock(cpi_context, max_amount)?;

    // 3. Initialize m3m3 vault
    let accounts = m3m3::InitializeVaultAccounts {
        vault: &ctx.accounts.m3m3_vault,
        stake_token_vault: &ctx.accounts.stake_token_vault,
        quote_token_vault: &ctx.accounts.quote_token_vault,
        top_staker_list: &ctx.accounts.top_staker_list,
        full_balance_list: &ctx.accounts.full_balance_list,
        stake_mint: &ctx.accounts.stake_mint,
        quote_mint: &ctx.accounts.quote_mint,
        system_program: &ctx.accounts.system_program,
        pool: &ctx.accounts.pool,
        event_authority: &ctx.accounts.m3m3_event_authority,
        lock_escrow: &ctx.accounts.lock_escrow,
        token_program: &ctx.accounts.token_program,
        associated_token_program: &ctx.accounts.associated_token_program,
        payer: &ctx.accounts.payer,
        program: &ctx.accounts.m3m3_program,
    };

    m3m3::initialize_vault_invoke(
        accounts,
        InitializeVaultIxArgs {
            params: vault_params,
        },
    )?;

    Ok(())
}
