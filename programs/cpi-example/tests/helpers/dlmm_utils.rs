use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use dlmm::state::{bin::BinArray, lb_pair::LbPair};
use dlmm::utils::pda::{derive_bin_array_pda, derive_oracle_pda};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;

use super::utils::add_packable_account;
use super::RPC;

pub struct SetupContext {
    pub pool_state: LbPair,
    pub user_token_x: Pubkey,
    pub user_token_y: Pubkey,
}

pub async fn setup_pool_from_cluster(
    test: &mut ProgramTest,
    pool: Pubkey,
    mock_user: Pubkey,
) -> SetupContext {
    let rpc_client = RpcClient::new(RPC.to_owned());

    let pool_account = rpc_client.get_account(&pool).await.unwrap();
    let pool_state = LbPair::try_deserialize(&mut pool_account.data.as_ref()).unwrap();

    test.add_account(pool, pool_account);

    let (oracle_key, _bump) = derive_oracle_pda(pool);
    let oracle_account = rpc_client.get_account(&oracle_key).await.unwrap();
    test.add_account(oracle_key, oracle_account);

    let active_bin_array_idx = BinArray::bin_id_to_bin_array_index(pool_state.active_id).unwrap();
    let (active_bin_array_key, _bump) = derive_bin_array_pda(pool, active_bin_array_idx.into());

    let bin_array_account = rpc_client.get_account(&active_bin_array_key).await.unwrap();
    test.add_account(active_bin_array_key, bin_array_account);

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];
    let mints = rpc_client.get_multiple_accounts(&mint_keys).await.unwrap();

    for (key, account) in mint_keys.iter().zip(mints) {
        test.add_account(*key, account.unwrap());
    }

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

    let token_ata_key = mint_keys
        .iter()
        .map(|key| get_associated_token_address(&mock_user, key))
        .collect::<Vec<_>>();

    for (ata_key, mint_key) in token_ata_key.iter().zip(mint_keys) {
        let state = anchor_spl::token::spl_token::state::Account {
            mint: mint_key,
            owner: mock_user,
            amount: u64::MAX / 2,
            state: AccountState::Initialized,
            ..Default::default()
        };

        add_packable_account(test, state, anchor_spl::token::ID, *ata_key);
    }

    SetupContext {
        pool_state,
        user_token_x: token_ata_key[0],
        user_token_y: token_ata_key[1],
    }
}
