use super::{dlmm_pda::*, process_and_assert_ok};
use anchor_lang::prelude::Pubkey;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use cpi_example::dlmm;
use cpi_example::dlmm::accounts::{LbPair, PositionV2};
use cpi_example::dlmm::types::InitializeLbPair2Params;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;

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

pub async fn create_new_lb_pair(
    banks_client: &mut BanksClient,
    mock_user: &Keypair,
    preset_parameter: Pubkey,
    token_x_mint: Pubkey,
    token_y_mint: Pubkey,
) -> Pubkey {
    let (lb_pair_key, _bump) =
        derive_lb_pair_pda_with_config(preset_parameter, token_x_mint, token_y_mint);
    let (reserve_x_key, _bump) = derive_reserve_pda(token_x_mint, lb_pair_key);
    let (reserve_y_key, _bump) = derive_reserve_pda(token_y_mint, lb_pair_key);
    let (oracle_key, _bump) = derive_oracle_pda(lb_pair_key);
    let (damm_event_authority, _bump) = derive_event_authority_pda();
    let (token_badge_x, _bump) = derive_token_badge_pda(token_x_mint);
    let (token_badge_y, _bump) = derive_token_badge_pda(token_y_mint);

    let usdc_mint_account = banks_client
        .get_account(token_x_mint)
        .await
        .unwrap()
        .unwrap();
    let usdt_mint_account = banks_client
        .get_account(token_y_mint)
        .await
        .unwrap()
        .unwrap();
    let token_badge_x_account = banks_client.get_account(token_badge_x).await.unwrap();
    let token_badge_y_account = banks_client.get_account(token_badge_y).await.unwrap();

    let accounts = cpi_example::accounts::InitializeLbPair {
        lb_pair: lb_pair_key,
        bin_array_bitmap_extension: Some(cpi_example::dlmm::ID),
        token_mint_x: token_x_mint,
        token_mint_y: token_y_mint,
        reserve_x: reserve_x_key,
        reserve_y: reserve_y_key,
        oracle: oracle_key,
        preset_parameter,
        funder: mock_user.pubkey(),
        token_badge_x: token_badge_x_account
            .map(|_| token_badge_x)
            .or(Some(cpi_example::dlmm::ID)),
        token_badge_y: token_badge_y_account
            .map(|_| token_badge_y)
            .or(Some(cpi_example::dlmm::ID)),
        token_program_x: usdc_mint_account.owner,
        token_program_y: usdt_mint_account.owner,
        system_program: system_program::ID,
        dlmm_program: cpi_example::dlmm::ID,
        damm_event_authority,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::InitializeLbPair {
        params: InitializeLbPair2Params {
            active_id: 0,
            padding: [0u8; 96],
        },
    }
    .data();

    let init_lb_pair_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(&[init_lb_pair_ix], mock_user, &[mock_user], banks_client).await;

    lb_pair_key
}

pub async fn create_bin_arrays_by_bin_range(
    banks_client: &mut BanksClient,
    lower_bin_id: i32,
    upper_bin_id: i32,
    lb_pair_address: Pubkey,
    mock_user: &Keypair,
) -> Vec<Pubkey> {
    let lower_bin_array_index = bin_id_to_bin_array_index(lower_bin_id).unwrap();
    let upper_bin_array_index = bin_id_to_bin_array_index(upper_bin_id)
        .unwrap()
        .max(lower_bin_array_index + 1);

    let mut bin_array_addresses = vec![];

    for bin_array_index in lower_bin_array_index..=upper_bin_array_index {
        let (bin_array_address, _bump) =
            derive_bin_array_pda(lb_pair_address, bin_array_index.into());

        let accounts = dlmm::client::accounts::InitializeBinArray {
            lb_pair: lb_pair_address,
            bin_array: bin_array_address,
            funder: mock_user.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        let ix_data = dlmm::client::args::InitializeBinArray {
            index: bin_array_index.into(),
        }
        .data();

        let init_bin_array_ix = Instruction {
            program_id: dlmm::ID,
            accounts,
            data: ix_data,
        };

        process_and_assert_ok(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(350_000),
                init_bin_array_ix,
            ],
            mock_user,
            &[mock_user],
            banks_client,
        )
        .await;

        bin_array_addresses.push(bin_array_address);
    }

    bin_array_addresses
}

pub async fn create_position_required_bin_arrays(
    banks_client: &mut BanksClient,
    position_state: &PositionV2,
    lb_pair_address: Pubkey,
    mock_user: &Keypair,
) -> Vec<Pubkey> {
    let bin_array_addresses = create_bin_arrays_by_bin_range(
        banks_client,
        position_state.lower_bin_id,
        position_state.upper_bin_id,
        lb_pair_address,
        mock_user,
    )
    .await;

    bin_array_addresses
}
