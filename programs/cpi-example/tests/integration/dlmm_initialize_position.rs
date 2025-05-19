use crate::helpers;
use crate::helpers::deserialize_zc_unalignment;
use crate::PRESET_PARAMETER_2_BIN_STEP_1;
use crate::USDC;
use crate::USDT;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use cpi_example::dlmm;
use cpi_example::dlmm::accounts::LbPair;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_cpi_example_program};
use solana_program_test::*;
use solana_sdk::system_program;
use solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};

async fn create_new_lb_pair(banks_client: &mut BanksClient, mock_user: &Keypair) -> Pubkey {
    crate::helpers::dlmm_utils::create_new_lb_pair(
        banks_client,
        mock_user,
        PRESET_PARAMETER_2_BIN_STEP_1,
        USDC,
        USDT,
    )
    .await
}

#[tokio::test]
async fn test_dlmm_initialize_position() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let lb_pair_address = create_new_lb_pair(&mut banks_client, &mock_user).await;
    let position_keypair = Keypair::new();

    let (dlmm_event_authority, _bump) = derive_event_authority_pda();

    let accounts = cpi_example::accounts::InitializeDlmmPosition {
        position: position_keypair.pubkey(),
        lb_pair: lb_pair_address,
        owner: mock_user.pubkey(),
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    let lb_pair_account = banks_client
        .get_account(lb_pair_address)
        .await
        .unwrap()
        .unwrap();

    let lb_pair_state: LbPair = deserialize_zc_unalignment(&lb_pair_account).unwrap();

    // 60 bins
    let width = 60;
    let lower_bin_id = lb_pair_state.active_id - width / 2;

    let ix_data = cpi_example::instruction::InitializeDlmmPosition {
        lower_bin_id,
        width,
    }
    .data();

    let init_position_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[init_position_ix],
        &mock_user,
        &[&mock_user, &position_keypair],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_initialize_position_with_pda_creator() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let lb_pair_address = create_new_lb_pair(&mut banks_client, &mock_user).await;
    let position_keypair = Keypair::new();

    let (dlmm_event_authority, _bump) = derive_event_authority_pda();

    let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &cpi_example::ID);

    let accounts = cpi_example::accounts::InitializeDlmmPositionWithPdaOwner {
        position: position_keypair.pubkey(),
        lb_pair: lb_pair_address,
        authority,
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    let lb_pair_account = banks_client
        .get_account(lb_pair_address)
        .await
        .unwrap()
        .unwrap();

    let lb_pair_state: LbPair = deserialize_zc_unalignment(&lb_pair_account).unwrap();

    // 60 bins
    let width = 60;
    let lower_bin_id = lb_pair_state.active_id - width / 2;

    let ix_data = cpi_example::instruction::InitializeDlmmPositionWithPdaOwner {
        lower_bin_id,
        width,
    }
    .data();

    let init_position_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[init_position_ix],
        &mock_user,
        &[&mock_user, &position_keypair],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_initialize_pda_position() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let lb_pair_address = create_new_lb_pair(&mut banks_client, &mock_user).await;

    let (dlmm_event_authority, _bump) = derive_event_authority_pda();

    let index: i64 = 0;
    let (position, _bump) = Pubkey::find_program_address(
        &[
            b"position",
            lb_pair_address.as_ref(),
            &index.to_le_bytes(),
            mock_user.pubkey().as_ref(),
        ],
        &cpi_example::ID,
    );

    let accounts = cpi_example::accounts::InitializeDlmmPdaPosition {
        position,
        lb_pair: lb_pair_address,
        owner: mock_user.pubkey(),
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    let lb_pair_account = banks_client
        .get_account(lb_pair_address)
        .await
        .unwrap()
        .unwrap();

    let lb_pair_state: LbPair = deserialize_zc_unalignment(&lb_pair_account).unwrap();

    // 60 bins
    let width = 60;
    let lower_bin_id = lb_pair_state.active_id - width / 2;

    let ix_data = cpi_example::instruction::InitializeDlmmPdaPosition {
        position_index: index,
        lower_bin_id,
        width,
    }
    .data();

    let init_position_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[init_position_ix],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_initialize_pda_position_with_pda_creator() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let lb_pair_address = create_new_lb_pair(&mut banks_client, &mock_user).await;

    let (dlmm_event_authority, _bump) = derive_event_authority_pda();

    let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &cpi_example::ID);

    let index: i64 = 0;
    let (position, _bump) = Pubkey::find_program_address(
        &[
            b"position",
            lb_pair_address.as_ref(),
            &index.to_le_bytes(),
            mock_user.pubkey().as_ref(),
        ],
        &cpi_example::ID,
    );

    let accounts = cpi_example::accounts::InitializeDlmmPdaPositionWithPdaOwner {
        position,
        lb_pair: lb_pair_address,
        authority,
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    let lb_pair_account = banks_client
        .get_account(lb_pair_address)
        .await
        .unwrap()
        .unwrap();

    let lb_pair_state: LbPair = deserialize_zc_unalignment(&lb_pair_account).unwrap();

    // 60 bins
    let width = 60;
    let lower_bin_id = lb_pair_state.active_id - width / 2;

    let ix_data = cpi_example::instruction::InitializeDlmmPdaPositionWithPdaOwner {
        index,
        lower_bin_id,
        width,
    }
    .data();

    let init_position_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[init_position_ix],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}
