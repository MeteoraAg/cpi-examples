use anchor_lang::prelude::*;
use anchor_lang::system_program::Transfer as NativeSolTransfer;
use anchor_spl::token::{Mint, Transfer as TokenTransfer};
use anchor_spl::token::{Token, TokenAccount};

const POOL_SIZE: usize = 8 + 944;

pub struct FundCreatorAuthorityAccounts<'b, 'info> {
    pub creator_token_a: &'b Account<'info, TokenAccount>,
    pub creator_token_b: &'b Account<'info, TokenAccount>,
    pub payer_token_a: &'b AccountInfo<'info>,
    pub payer_token_b: &'b AccountInfo<'info>,
    pub token_program: &'b Program<'info, Token>,
    pub payer: &'b Signer<'info>,
    pub system_program: &'b Program<'info, System>,
    pub creator_authority: &'b AccountInfo<'info>,
}

pub fn fund_creator_authority<'b, 'info>(
    token_a_amount: u64,
    token_b_amount: u64,
    accounts: FundCreatorAuthorityAccounts<'b, 'info>,
) -> Result<()> {
    let FundCreatorAuthorityAccounts {
        creator_token_a,
        creator_token_b,
        payer_token_a,
        payer_token_b,
        token_program,
        payer,
        system_program,
        creator_authority,
    } = accounts;

    // Fund creator PDA with token A and token B
    if token_a_amount > creator_token_a.amount {
        let amount = token_a_amount - creator_token_a.amount;
        anchor_spl::token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                TokenTransfer {
                    from: payer_token_a.to_account_info(),
                    to: creator_token_a.to_account_info(),
                    authority: payer.to_account_info(),
                },
            ),
            amount,
        )?;
    }

    if token_b_amount > creator_token_b.amount {
        let amount = token_b_amount - creator_token_b.amount;
        anchor_spl::token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                TokenTransfer {
                    from: payer_token_b.to_account_info(),
                    to: creator_token_b.to_account_info(),
                    authority: payer.to_account_info(),
                },
            ),
            amount,
        )?;
    }

    // Fund creator PDA with SOL to pay for account rental
    let mut lamports: u64 = 0;

    // Pool
    lamports += Rent::get()?.minimum_balance(POOL_SIZE);
    // LP mint
    lamports += Rent::get()?.minimum_balance(Mint::LEN);
    //  a_vault_lp + b_vault_lp + creator LP ATA + protocol fee A + protocol fee B
    let token_account_lamports = Rent::get()?.minimum_balance(TokenAccount::LEN);
    lamports += token_account_lamports * 5;
    // LP mint Metadata
    lamports += Rent::get()?.minimum_balance(679);
    // Metaplex fee ...
    lamports += 10_000_000;

    msg!("Required lamports: {}", lamports);

    anchor_lang::system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            NativeSolTransfer {
                from: payer.to_account_info(),
                to: creator_authority.to_account_info(),
            },
        ),
        // Weird bug in bpf, more lamport causes failure. The calculated lamports above should be the correct one
        // lamports,
        34290400,
    )?;

    Ok(())
}
