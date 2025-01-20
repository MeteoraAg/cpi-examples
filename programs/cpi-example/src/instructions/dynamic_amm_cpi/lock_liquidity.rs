use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use dynamic_amm::state::Pool;

#[derive(Accounts)]
pub struct DynamicAmmLockLiquidity<'info> {
    /// CHECK: Pool account (PDA)
    #[account(
        mut,
        has_one = lp_mint,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Pool LP mint
    pub lp_mint: Box<Account<'info, Mint>>,

    /// CHECK: Lock escrow for user 0
    #[account(mut)]
    pub lock_escrow_0: UncheckedAccount<'info>,

    /// CHECK: Lock escrow for user 1
    #[account(mut)]
    pub lock_escrow_1: UncheckedAccount<'info>,

    /// CHECK: Payer lp token account
    #[account(
        mut,
        token::mint = lp_mint,
    )]
    pub source_lp_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: Escrow vault 0
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow_0,
        payer = payer
    )]
    pub escrow_vault_0: Box<Account<'info, TokenAccount>>,

    /// CHECK: Escrow vault 1
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow_1,
        payer = payer
    )]
    pub escrow_vault_1: Box<Account<'info, TokenAccount>>,

    /// Wallet that hold LP tokens and wish to lock user 0 and 1
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: User 0 account
    pub user_0: UncheckedAccount<'info>,

    /// CHECK: User 1 account
    pub user_1: UncheckedAccount<'info>,

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

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,

    /// CHECK: System program
    pub system_program: UncheckedAccount<'info>,

    /// CHECK: Token program.
    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Associated token program
    pub associated_token_program: UncheckedAccount<'info>,
}

struct LockUserAccountsAndInfo<'b, 'info> {
    pub lp_amount: u64,
    pub lock_escrow: &'b UncheckedAccount<'info>,
    pub escrow_vault: &'b Account<'info, TokenAccount>,
    pub owner: &'b UncheckedAccount<'info>,
}

/// Lock liquidity of a user to multiple users. Each user can claim fee on locked liquidity based on their allocation.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `allocations` - The percentage of liquidity to be locked for each user. The values must add up to 10_000.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_lock_liquidity(ctx: Context<DynamicAmmLockLiquidity>, allocations: [u16; 2]) -> Result<()> {
    let total_bps: u32 = allocations
        .iter()
        .map(|alloc| Into::<u32>::into(*alloc))
        .sum();

    assert!(total_bps == 10_000, "Invalid total bps");

    let user_0_lp_amount: u64 = u128::from(ctx.accounts.source_lp_tokens.amount)
        .checked_mul(allocations[0].into())
        .unwrap()
        .checked_div(10_000)
        .unwrap()
        .try_into()
        .unwrap();

    let user_1_lp_amount = ctx
        .accounts
        .source_lp_tokens
        .amount
        .checked_sub(user_0_lp_amount)
        .unwrap();

    let user_accounts_and_info = [
        LockUserAccountsAndInfo {
            lp_amount: user_0_lp_amount,
            lock_escrow: &ctx.accounts.lock_escrow_0,
            escrow_vault: ctx.accounts.escrow_vault_0.as_ref(),
            owner: &ctx.accounts.user_0,
        },
        LockUserAccountsAndInfo {
            lp_amount: user_1_lp_amount,
            lock_escrow: &ctx.accounts.lock_escrow_1,
            escrow_vault: ctx.accounts.escrow_vault_1.as_ref(),
            owner: &ctx.accounts.user_1,
        },
    ];

    for LockUserAccountsAndInfo {
        lp_amount,
        lock_escrow,
        escrow_vault,
        owner,
    } in user_accounts_and_info
    {
        // 1. Initialize stake escrow
        let accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
            pool: ctx.accounts.pool.to_account_info(),
            lock_escrow: lock_escrow.to_account_info(),
            owner: owner.to_account_info(),
            lp_mint: ctx.accounts.lp_mint.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(ctx.accounts.dynamic_amm_program.to_account_info(), accounts);
        dynamic_amm::cpi::create_lock_escrow(cpi_context)?;

        // 2. Lock user LP to user
        let accounts = dynamic_amm::cpi::accounts::Lock {
            pool: ctx.accounts.pool.to_account_info(),
            lock_escrow: lock_escrow.to_account_info(),
            lp_mint: ctx.accounts.lp_mint.to_account_info(),
            owner: ctx.accounts.payer.to_account_info(),
            source_tokens: ctx.accounts.source_lp_tokens.to_account_info(),
            escrow_vault: escrow_vault.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            a_vault: ctx.accounts.a_vault.to_account_info(),
            b_vault: ctx.accounts.b_vault.to_account_info(),
            a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
            b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
            a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
            b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(ctx.accounts.dynamic_amm_program.to_account_info(), accounts);
        dynamic_amm::cpi::lock(cpi_context, lp_amount)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct DynamicAmmLockLiquidityPdaCreator<'info> {
    /// CHECK: Pool account (PDA)
    #[account(
        mut,
        has_one = lp_mint,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Pool LP mint
    pub lp_mint: Box<Account<'info, Mint>>,

    /// CHECK: Lock escrow for pool creator. PDA.
    #[account(mut)]
    pub lock_escrow_creator: UncheckedAccount<'info>,

    /// CHECK: Lock escrow for user 0
    #[account(mut)]
    pub lock_escrow_0: UncheckedAccount<'info>,

    /// CHECK: Pool creator authority. PDA.
    #[account(
        mut,
        seeds = [b"creator"],
        bump
    )]
    pub creator_authority: UncheckedAccount<'info>,

    /// CHECK: Creator lp token account
    #[account(
        mut,
        token::mint = lp_mint,
        token::authority = creator_authority,
    )]
    pub source_lp_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: Escrow vault pool creator
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow_creator,
        payer = payer
    )]
    pub escrow_vault_creator: Box<Account<'info, TokenAccount>>,

    /// CHECK: Escrow vault 0
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = lock_escrow_0,
        payer = payer
    )]
    pub escrow_vault_0: Box<Account<'info, TokenAccount>>,

    /// CHECK: CPI example program admin. Only admin can call this instruction. Also funder for account rental.
    #[account(
        mut, 
        constraint = crate::assert_eq_admin(payer.key()
    ))]
    pub payer: Signer<'info>,

    /// CHECK: User 0 account
    pub user_0: UncheckedAccount<'info>,

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

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,

    /// CHECK: System program
    pub system_program: UncheckedAccount<'info>,

    /// CHECK: Token program.
    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Associated token program
    pub associated_token_program: UncheckedAccount<'info>,
}

/// Lock liquidity of pool creator PDA to self and an user. Each can claim fee on locked liquidity based on their allocation.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `allocations` - The percentage of liquidity to be locked for each user. The values must add up to 10_000.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_lock_liquidity_pda_creator(
    ctx: Context<DynamicAmmLockLiquidityPdaCreator>,
    allocations: [u16; 2],
) -> Result<()> {
    let total_bps: u32 = allocations
        .iter()
        .map(|alloc| Into::<u32>::into(*alloc))
        .sum();

    assert!(total_bps == 10_000, "Invalid total bps");

    let pda_creator_lp_amount: u64 = u128::from(ctx.accounts.source_lp_tokens.amount)
        .checked_mul(allocations[0].into())
        .unwrap()
        .checked_div(10_000)
        .unwrap()
        .try_into()
        .unwrap();

    let user_1_lp_amount = ctx
        .accounts
        .source_lp_tokens
        .amount
        .checked_sub(pda_creator_lp_amount)
        .unwrap();

    // 1. Initialize lock escrow for pool creator PDA
    let accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow_creator.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        accounts,
    );

    dynamic_amm::cpi::create_lock_escrow(cpi_context)?;

    // 2. Lock pool creator PDA LP to pool creator PDA lock escrow
    let accounts = dynamic_amm::cpi::accounts::Lock {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow_creator.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        source_tokens: ctx.accounts.source_lp_tokens.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault_creator.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
    };

    let seeds = [
        b"creator".as_ref(),
        &[*ctx.bumps.get("creator_authority").unwrap()],
    ];

    let signer_seeds = &[&seeds[..]];

    msg!("Lock for pda");

    let cpi_context =
        CpiContext::new_with_signer(ctx.accounts.dynamic_amm_program.to_account_info(), accounts, signer_seeds);
    dynamic_amm::cpi::lock(cpi_context, pda_creator_lp_amount)?;

    msg!("Lock for pda done");


    // 3. Initialize lock escrow for user 1
    let accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow_0.to_account_info(),
        owner: ctx.accounts.user_0.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        accounts,
    );

    dynamic_amm::cpi::create_lock_escrow(cpi_context)?;

    // 4. Lock pool creator PDA LP to user 1 lock escrow
    let accounts = dynamic_amm::cpi::accounts::Lock {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow_0.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        owner: ctx.accounts.creator_authority.to_account_info(),
        source_tokens: ctx.accounts.source_lp_tokens.to_account_info(),
        escrow_vault: ctx.accounts.escrow_vault_0.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        a_vault: ctx.accounts.a_vault.to_account_info(),
        b_vault: ctx.accounts.b_vault.to_account_info(),
        a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
        b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
        a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
        b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
    };

    let cpi_context =
        CpiContext::new_with_signer(ctx.accounts.dynamic_amm_program.to_account_info(), accounts, signer_seeds);
    dynamic_amm::cpi::lock(cpi_context, user_1_lp_amount)?; 

    Ok(())
}
