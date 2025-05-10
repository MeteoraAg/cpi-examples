use super::dlmm_pda::*;
use anchor_lang::prelude::Pubkey;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use cpi_example::dlmm::accounts::LbPair;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;

use super::utils::{add_packable_account, deserialize_zc_unalignment};
use super::RPC;

/// Get bin array index from bin id
pub fn bin_id_to_bin_array_index(bin_id: i32) -> Option<i32> {
    use cpi_example::dlmm::constants::MAX_BIN_PER_ARRAY;
    let idx = bin_id.checked_div(MAX_BIN_PER_ARRAY as i32)?;
    let rem = bin_id.checked_rem(MAX_BIN_PER_ARRAY as i32)?;

    if bin_id.is_negative() && rem != 0 {
        idx.checked_sub(1)
    } else {
        Some(idx)
    }
}

pub async fn setup_accounts_from_cluster(test: &mut ProgramTest, keys: &[Pubkey]) {
    let rpc_client = RpcClient::new(RPC.to_owned());

    let accounts = rpc_client.get_multiple_accounts(keys).await.unwrap();
    for (key, account) in keys.iter().zip(accounts) {
        test.add_account(*key, account.unwrap());
    }
}

pub struct MintSetupContext {
    #[allow(dead_code)]
    pub mint: Pubkey,
    pub user_token: Pubkey,
}

pub async fn setup_mint_from_cluster(
    test: &mut ProgramTest,
    mint_keys: &[Pubkey],
    mock_user: Pubkey,
) -> Vec<MintSetupContext> {
    let mut setup_results = vec![];

    let rpc_client = RpcClient::new(RPC.to_owned());
    let mints = rpc_client.get_multiple_accounts(mint_keys).await.unwrap();

    for (key, account) in mint_keys.iter().zip(mints) {
        test.add_account(*key, account.unwrap());
    }

    let token_ata_key = mint_keys
        .iter()
        .map(|key| get_associated_token_address(&mock_user, key))
        .collect::<Vec<_>>();

    for (ata_key, mint_key) in token_ata_key.iter().zip(mint_keys) {
        let state = anchor_spl::token::spl_token::state::Account {
            mint: *mint_key,
            owner: mock_user,
            amount: u64::MAX / 2,
            state: AccountState::Initialized,
            ..Default::default()
        };

        add_packable_account(test, state, anchor_spl::token::ID, *ata_key);

        setup_results.push(MintSetupContext {
            mint: state.mint,
            user_token: *ata_key,
        });
    }

    test.add_account(
        mock_user,
        Account {
            lamports: u32::MAX.into(),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            ..Default::default()
        },
    );

    setup_results
}

pub struct PoolSetupContext {
    pub pool_state: LbPair,
    pub user_token_x: Pubkey,
    pub user_token_y: Pubkey,
}

pub async fn setup_pool_from_cluster(
    test: &mut ProgramTest,
    pool: Pubkey,
    mock_user: Pubkey,
) -> PoolSetupContext {
    let rpc_client = RpcClient::new(RPC.to_owned());

    let pool_account = rpc_client.get_account(&pool).await.unwrap();
    let pool_state: LbPair = deserialize_zc_unalignment(&pool_account).unwrap();

    test.add_account(pool, pool_account);

    let (oracle_key, _bump) = derive_oracle_pda(pool);
    let oracle_account = rpc_client.get_account(&oracle_key).await.unwrap();
    test.add_account(oracle_key, oracle_account);

    let active_bin_array_idx = bin_id_to_bin_array_index(pool_state.active_id).unwrap();
    let (active_bin_array_key, _bump) = derive_bin_array_pda(pool, active_bin_array_idx.into());

    let bin_array_account = rpc_client.get_account(&active_bin_array_key).await.unwrap();
    test.add_account(active_bin_array_key, bin_array_account);

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];
    let mint_setup = setup_mint_from_cluster(test, &mint_keys, mock_user).await;

    let reserve_keys = vec![pool_state.reserve_x, pool_state.reserve_y];

    let tokens = rpc_client
        .get_multiple_accounts(&reserve_keys)
        .await
        .unwrap();

    for (key, account) in reserve_keys.into_iter().zip(tokens) {
        test.add_account(key, account.unwrap());
    }

    test.add_account(
        mock_user,
        Account {
            lamports: u32::MAX.into(),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            ..Default::default()
        },
    );

    PoolSetupContext {
        pool_state,
        user_token_x: mint_setup[0].user_token,
        user_token_y: mint_setup[1].user_token,
    }
}
