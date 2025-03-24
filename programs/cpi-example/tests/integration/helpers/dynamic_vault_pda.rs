use super::dynamic_vault_aux_lp_mint::*;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

const VAULT_BASE_ADDRESS: Pubkey = pubkey!("HWzXGcGHy4tcpYfaRDCyLNzXqBTv3E6BttpCH2vJxArv");

pub fn derive_vault_key(mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"vault", mint.as_ref(), VAULT_BASE_ADDRESS.as_ref()],
        &cpi_example::dynamic_vault::ID,
    )
    .0
}

pub fn derive_token_vault_key(vault: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"token_vault", vault.as_ref()],
        &cpi_example::dynamic_vault::ID,
    )
    .0
}

pub fn derive_lp_mint_key(vault: Pubkey) -> Pubkey {
    let non_derived_based_lp_mint = VAULT_WITH_NON_PDA_BASED_LP_MINT.get(&vault).cloned();

    if let Some(lp_mint) = non_derived_based_lp_mint {
        lp_mint
    } else {
        Pubkey::find_program_address(
            &[b"lp_mint", vault.as_ref()],
            &cpi_example::dynamic_vault::ID,
        )
        .0
    }
}
