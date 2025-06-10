use anchor_lang::{prelude::*, system_program::Transfer};
use anchor_spl::token::TokenAccount;

use crate::dlmm::constants::MAX_BIN_PER_ARRAY;

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

// Reference: https://github.com/MeteoraAg/dlmm-sdk/blob/9c4aeacf7ffabbbc68df0db7f2ed981847b12cfb/commons/src/extensions/bin_array.rs#L62
pub fn bin_id_to_bin_array_index(bin_id: i32) -> Option<i32> {
    let idx = bin_id.checked_div(MAX_BIN_PER_ARRAY as i32)?;
    let rem = bin_id.checked_rem(MAX_BIN_PER_ARRAY as i32)?;

    if bin_id.is_negative() && rem != 0 {
        idx.checked_sub(1)
    } else {
        Some(idx)
    }
}
