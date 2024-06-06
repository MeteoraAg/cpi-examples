use anchor_lang::AccountDeserialize;
use anchor_spl::{
    associated_token::get_associated_token_address, token::spl_token::state::AccountState,
};
use dynamic_amm::state::Pool;
use dynamic_vault::state::Vault;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::ProgramTest;
use solana_sdk::{account::Account, pubkey::Pubkey};

use super::{utils::add_packable_account, RPC};

pub struct SetupContext {
    pub pool_state: Pool,
    pub a_vault_state: Vault,
    pub b_vault_state: Vault,
    pub user_token_a: Pubkey,
    pub user_token_b: Pubkey,
}

pub async fn setup_pool_from_cluster(
    test: &mut ProgramTest,
    pool: Pubkey,
    mock_user: Pubkey,
) -> SetupContext {
    let rpc_client = RpcClient::new(RPC.to_owned());

    let pool_account = rpc_client.get_account(&pool).await.unwrap();
    let pool_state = Pool::try_deserialize(&mut pool_account.data.as_ref()).unwrap();

    test.add_account(pool, pool_account);

    let a_vault_account = rpc_client.get_account(&pool_state.a_vault).await.unwrap();
    let a_vault_state = Vault::try_deserialize(&mut a_vault_account.data.as_ref()).unwrap();

    let b_vault_account = rpc_client.get_account(&pool_state.b_vault).await.unwrap();
    let b_vault_state = Vault::try_deserialize(&mut b_vault_account.data.as_ref()).unwrap();

    test.add_account(pool_state.a_vault, a_vault_account);
    test.add_account(pool_state.b_vault, b_vault_account);

    let mint_keys = vec![
        pool_state.token_a_mint,
        pool_state.token_b_mint,
        pool_state.lp_mint,
        a_vault_state.lp_mint,
        b_vault_state.lp_mint,
    ];

    let accounts = rpc_client.get_multiple_accounts(&mint_keys).await.unwrap();

    for (key, account) in mint_keys.iter().zip(accounts) {
        test.add_account(*key, account.unwrap());
    }

    let token_keys = vec![
        pool_state.admin_token_a_fee,
        pool_state.admin_token_b_fee,
        pool_state.a_vault_lp,
        pool_state.b_vault_lp,
        a_vault_state.token_vault,
        b_vault_state.token_vault,
    ];

    let accounts = rpc_client.get_multiple_accounts(&token_keys).await.unwrap();

    for (key, account) in token_keys.iter().zip(accounts) {
        test.add_account(*key, account.unwrap());
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

    let token_ata_key = [pool_state.token_a_mint, pool_state.token_b_mint]
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
        a_vault_state,
        b_vault_state,
        user_token_a: token_ata_key[0],
        user_token_b: token_ata_key[1],
    }
}
