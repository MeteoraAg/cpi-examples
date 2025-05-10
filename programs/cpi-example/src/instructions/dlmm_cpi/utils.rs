use anchor_lang::{prelude::*, system_program::Transfer};
use anchor_spl::token::TokenAccount;

const LB_PAIR_SIZE: usize = 904;
const ORACLE_SIZE: usize = 3232;

pub fn fund_creator_authority<'info>(
    system_program: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    creator_authority: AccountInfo<'info>,
) -> Result<()> {
    // Fund creator PDA with SOL to pay for account rental
    let mut lamports: u64 = 0;

    // Pool
    let lb_pair_account_lamports = Rent::get()?.minimum_balance(LB_PAIR_SIZE);
    lamports += lb_pair_account_lamports;

    // Reserve X + Y
    let token_account_lamports = Rent::get()?.minimum_balance(TokenAccount::LEN);
    lamports += token_account_lamports * 2;

    let oracle_account_lamports = Rent::get()?.minimum_balance(ORACLE_SIZE);
    lamports += oracle_account_lamports;

    msg!("Required lamports: {}", lamports);

    anchor_lang::system_program::transfer(
        CpiContext::new(
            system_program,
            Transfer {
                from: payer,
                to: creator_authority,
            },
        ),
        lamports,
    )?;

    Ok(())
}
