use solana_sdk::pubkey::Pubkey;

const PRESET_PARAMETER_2_BIN_STEP_1: Pubkey =
    solana_sdk::pubkey!("BB2atM1VveWJJUERbufE73fzZAss74J6DcEx1jGdTvwg");
const USDC: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");

mod helpers;

mod dlmm_deposit;
mod dlmm_initialize_lb_pair;
mod dlmm_initialize_position;
mod dlmm_swap;
mod dlmm_withdraw;
mod dynamic_amm_claim_fee;
mod dynamic_amm_init_pool;
mod dynamic_amm_lock_liquidity;
mod dynamic_amm_swap;
mod m3m3_initialize_vault;
