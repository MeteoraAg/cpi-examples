use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{entrypoint::ProgramResult, pubkey::Pubkey};

pub mod dlmm_pda;
pub mod dlmm_utils;
pub mod dynamic_amm_ix_account_builder;
pub mod dynamic_amm_pda;
pub mod dynamic_amm_utils;
pub mod dynamic_vault_pda;
pub mod m3m3_pda;

mod dynamic_amm_aux_lp_mint;
mod dynamic_vault_aux_lp_mint;

mod utils;

pub use utils::process_and_assert_ok;
const RPC: &str = "https://api.mainnet-beta.solana.com";

pub const JUP: Pubkey = solana_sdk::pubkey!("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN");
pub const USDC: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const CONFIG: Pubkey = solana_sdk::pubkey!("FiENCCbPi3rFh5pW2AJ59HC53yM32eLaCjMKxRqanKFJ");

pub fn setup_cpi_example_program() -> ProgramTest {
    ProgramTest::new("cpi_example", cpi_example::ID, processor!(entry))
}

/// This is a wrapper to get the processor macro to work.
fn entry(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts = Box::leak(Box::new(accounts.to_vec()));
    cpi_example::entry(program_id, accounts, instruction_data)
}
