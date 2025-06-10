use crate::dlmm;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeDlmmPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Arbitrary keypair generated in client side
    #[account(mut)]
    pub position: Signer<'info>,

    /// CHECK: Lb pair account
    pub lb_pair: UncheckedAccount<'info>,

    pub owner: Signer<'info>,

    /// CHECK: System program account
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    /// CHECK: Rent
    pub rent: UncheckedAccount<'info>,
}

/// Initializes a new position account with the given parameters.
///
/// # Parameters
///
/// * `ctx` - The context containing accounts and programs.
/// * `lower_bin_id` - The lower bin ID for the position.
/// * `width` - The width of the position.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_position(
    ctx: Context<InitializeDlmmPosition>,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializePosition {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);
    dlmm::cpi::initialize_position(cpi_ctx, lower_bin_id, width)
}

#[derive(Accounts)]
#[instruction(index: i64)]
pub struct InitializeDlmmPdaPositionWithPdaOwner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Position PDA's account. It's useful for indexing purposes
    #[account(
        mut,
        seeds = [
            b"position",
            lb_pair.key().as_ref(),
            index.to_le_bytes().as_ref(),
            payer.key().as_ref(),
        ],
        bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Lb pair account
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: System program account
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    /// CHECK: Rent
    pub rent: UncheckedAccount<'info>,

    /// CHECK: PDA authority
    #[account(
        seeds = [
            b"authority".as_ref()
        ],
        bump
    )]
    pub authority: UncheckedAccount<'info>,
}

/// Initializes a new position PDA account with the given parameters where position owner is a PDA.
///
/// # Parameters
///
/// * `ctx` - The context containing accounts and programs.
/// * `index` - Unused parameter.
/// * `lower_bin_id` - The lower bin ID for the position.
/// * `width` - The width of the position.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_pda_position_with_pda_owner(
    ctx: Context<InitializeDlmmPdaPositionWithPdaOwner>,
    index: i64,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializePosition {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        owner: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let lb_pair_key = ctx.accounts.lb_pair.key();
    let payer_key = ctx.accounts.payer.key();
    let index_bytes = index.to_le_bytes();

    let authority_seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let position_seeds = &[
        b"position".as_ref(),
        lb_pair_key.as_ref(),
        index_bytes.as_ref(),
        payer_key.as_ref(),
        &[ctx.bumps.position],
    ];

    let signer_seeds = &[&authority_seeds[..], &position_seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    );
    dlmm::cpi::initialize_position(cpi_ctx, lower_bin_id, width)
}

#[derive(Accounts)]
pub struct InitializeDlmmPositionWithPdaOwner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Arbitrary keypair generated in client side
    #[account(mut)]
    pub position: Signer<'info>,

    /// CHECK: Lb pair account
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: System program account
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    /// CHECK: Rent
    pub rent: UncheckedAccount<'info>,

    /// CHECK: PDA authority
    #[account(
        seeds = [
            b"authority".as_ref()
        ],
        bump
    )]
    pub authority: UncheckedAccount<'info>,
}

/// Initializes a new position account with the given parameters where position owner is a PDA.
///
/// # Parameters
///
/// * `ctx` - The context containing accounts and programs.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_position_with_pda_owner(
    ctx: Context<InitializeDlmmPositionWithPdaOwner>,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializePosition {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        owner: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let seeds = &[b"authority".as_ref(), &[ctx.bumps.authority]];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    );
    dlmm::cpi::initialize_position(cpi_ctx, lower_bin_id, width)
}

#[derive(Accounts)]
#[instruction(index: i64)]
pub struct InitializeDlmmPdaPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Position PDA's account. It's useful for indexing purposes
    #[account(
        mut,
        seeds = [
            b"position",
            lb_pair.key().as_ref(),
            index.to_le_bytes().as_ref(),
            owner.key().as_ref(),
        ],
        bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Lb pair account
    pub lb_pair: UncheckedAccount<'info>,

    pub owner: Signer<'info>,

    /// CHECK: System program account
    pub system_program: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM event authority
    pub dlmm_event_authority: UncheckedAccount<'info>,

    /// CHECK: Rent
    pub rent: UncheckedAccount<'info>,
}

/// Initializes a new position PDA account with the given parameters.
///
/// # Parameters
///
/// * `ctx` - The context containing accounts and programs.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_pda_position(
    ctx: Context<InitializeDlmmPdaPosition>,
    position_index: i64,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializePosition {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
        event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let lb_pair_key = ctx.accounts.lb_pair.key();
    let index_bytes = position_index.to_le_bytes();
    let owner_key = ctx.accounts.owner.key();

    let seeds = &[
        b"position",
        lb_pair_key.as_ref(),
        index_bytes.as_ref(),
        owner_key.as_ref(),
        &[ctx.bumps.position],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dlmm_program.to_account_info(),
        accounts,
        signer_seeds,
    );
    dlmm::cpi::initialize_position(cpi_ctx, lower_bin_id, width)
}
