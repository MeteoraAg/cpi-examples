use solana_sdk::pubkey::Pubkey;

pub fn derive_m3m3_vault_key(pool_key: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", pool_key.as_ref()], &m3m3::ID).0
}

pub fn derive_top_staker_list_key(vault_key: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"list", vault_key.as_ref()], &m3m3::ID).0
}

pub fn derive_full_balance_list_key(vault_key: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"balance", vault_key.as_ref()], &m3m3::ID).0
}

pub fn derive_m3m3_event_authority_key() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &m3m3::ID).0
}
