use solana_sdk::pubkey::Pubkey;

pub mod dlmm_utils;
pub mod dynamic_amm_utils;
pub mod m3m3_utils;
mod utils;

pub use utils::process_and_assert_ok;
const RPC: &str = "https://api.mainnet-beta.solana.com";

pub const JUP: Pubkey = solana_sdk::pubkey!("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN");
pub const USDC: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const CONFIG: Pubkey = solana_sdk::pubkey!("FiENCCbPi3rFh5pW2AJ59HC53yM32eLaCjMKxRqanKFJ");
