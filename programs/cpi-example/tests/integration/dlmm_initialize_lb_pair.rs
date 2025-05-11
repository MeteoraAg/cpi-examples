use crate::helpers;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use cpi_example::dlmm;
use cpi_example::dlmm::types::InitializeLbPair2Params;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_cpi_example_program};
use solana_program_test::*;
use solana_sdk::system_program;
use solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};

const PRESET_PARAMETER_2_BIN_STEP_1: Pubkey =
    solana_sdk::pubkey!("BB2atM1VveWJJUERbufE73fzZAss74J6DcEx1jGdTvwg");

const USDC: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[tokio::test]
async fn test_dlmm_initialize_lb_pair() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let (lb_pair_key, _bump) =
        derive_lb_pair_pda_with_config(PRESET_PARAMETER_2_BIN_STEP_1, USDC, USDT);
    let (reserve_x_key, _bump) = derive_reserve_pda(USDC, lb_pair_key);
    let (reserve_y_key, _bump) = derive_reserve_pda(USDT, lb_pair_key);
    let (oracle_key, _bump) = derive_oracle_pda(lb_pair_key);
    let (damm_event_authority, _bump) = derive_event_authority_pda();
    let (token_badge_x, _bump) = derive_token_badge_pda(USDC);
    let (token_badge_y, _bump) = derive_token_badge_pda(USDT);

    let usdc_mint_account = banks_client.get_account(USDC).await.unwrap().unwrap();
    let usdt_mint_account = banks_client.get_account(USDT).await.unwrap().unwrap();
    let token_badge_x_account = banks_client.get_account(token_badge_x).await.unwrap();
    let token_badge_y_account = banks_client.get_account(token_badge_y).await.unwrap();

    let accounts = cpi_example::accounts::InitializeLbPair {
        lb_pair: lb_pair_key,
        bin_array_bitmap_extension: Some(cpi_example::dlmm::ID),
        token_mint_x: USDC,
        token_mint_y: USDT,
        reserve_x: reserve_x_key,
        reserve_y: reserve_y_key,
        oracle: oracle_key,
        preset_parameter: PRESET_PARAMETER_2_BIN_STEP_1,
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

    process_and_assert_ok(
        &[init_lb_pair_ix],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_initialize_lb_pair_with_pda_creator() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let (lb_pair_key, _bump) =
        derive_lb_pair_pda_with_config(PRESET_PARAMETER_2_BIN_STEP_1, USDC, USDT);
    let (reserve_x_key, _bump) = derive_reserve_pda(USDC, lb_pair_key);
    let (reserve_y_key, _bump) = derive_reserve_pda(USDT, lb_pair_key);
    let (oracle_key, _bump) = derive_oracle_pda(lb_pair_key);
    let (damm_event_authority, _bump) = derive_event_authority_pda();
    let (token_badge_x, _bump) = derive_token_badge_pda(USDC);
    let (token_badge_y, _bump) = derive_token_badge_pda(USDT);

    let usdc_mint_account = banks_client.get_account(USDC).await.unwrap().unwrap();
    let usdt_mint_account = banks_client.get_account(USDT).await.unwrap().unwrap();
    let token_badge_x_account = banks_client.get_account(token_badge_x).await.unwrap();
    let token_badge_y_account = banks_client.get_account(token_badge_y).await.unwrap();

    let (creator_authority, _bump) = Pubkey::find_program_address(&[b"creator"], &cpi_example::ID);

    let accounts = cpi_example::accounts::InitializeLbPairWithPdaCreator {
        lb_pair: lb_pair_key,
        bin_array_bitmap_extension: Some(cpi_example::dlmm::ID),
        token_mint_x: USDC,
        token_mint_y: USDT,
        reserve_x: reserve_x_key,
        reserve_y: reserve_y_key,
        oracle: oracle_key,
        preset_parameter: PRESET_PARAMETER_2_BIN_STEP_1,
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
        creator_authority,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::InitializeLbPairWithPdaCreator {
        params: InitializeLbPair2Params {
            // See: https://github.com/MeteoraAg/dlmm-sdk/blob/e00b46ed85c71927e1cb9554d1cfe636dd440e04/ts-client/src/dlmm/index.ts#L1172 on how to convert price to bin id
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

    process_and_assert_ok(
        &[init_lb_pair_ix],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}
