use crate::helpers;
use crate::helpers::deserialize_zc_unalignment;
use anchor_lang::prelude::AccountMeta;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use anchor_spl::token::spl_token::instruction::transfer;
use cpi_example::dlmm;
use cpi_example::dlmm::accounts::LbPair;
use cpi_example::dlmm::accounts::PositionV2;
use cpi_example::dlmm::types::InitializeLbPair2Params;
use cpi_example::dlmm::types::LiquidityParameterByStrategy;
use cpi_example::dlmm::types::RemainingAccountsInfo;
use cpi_example::dlmm::types::StrategyParameters;
use cpi_example::dlmm::types::StrategyType;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_cpi_example_program};
use solana_program_test::*;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::system_program;
use solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

const PRESET_PARAMETER_2_BIN_STEP_1: Pubkey =
    solana_sdk::pubkey!("BB2atM1VveWJJUERbufE73fzZAss74J6DcEx1jGdTvwg");

const USDC: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

async fn create_new_lb_pair(banks_client: &mut BanksClient, mock_user: &Keypair) -> Pubkey {
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

    process_and_assert_ok(&[init_lb_pair_ix], mock_user, &[mock_user], banks_client).await;

    lb_pair_key
}

async fn create_position_required_bin_arrays(
    banks_client: &mut BanksClient,
    position_state: &PositionV2,
    lb_pair_address: Pubkey,
    mock_user: &Keypair,
) -> Vec<Pubkey> {
    let bin_array_lower_index = bin_id_to_bin_array_index(position_state.lower_bin_id).unwrap();
    let bin_array_upper_index = bin_array_lower_index + 1;

    let mut bin_array_addresses = vec![];

    for ba_idx in bin_array_lower_index..=bin_array_upper_index {
        let (bin_array_address, _bump) = derive_bin_array_pda(lb_pair_address, ba_idx.into());

        let accounts = dlmm::client::accounts::InitializeBinArray {
            lb_pair: lb_pair_address,
            bin_array: bin_array_address,
            funder: mock_user.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        let ix_data = dlmm::client::args::InitializeBinArray {
            index: ba_idx.into(),
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

#[tokio::test]
async fn test_dlmm_deposit() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Initialize position
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

    // 2. Deposit
    let position_account = banks_client
        .get_account(position_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let position_state: PositionV2 = deserialize_zc_unalignment(&position_account).unwrap();

    let bin_array_addresses = create_position_required_bin_arrays(
        &mut banks_client,
        &position_state,
        lb_pair_address,
        &mock_user,
    )
    .await;

    let user_token_x = get_associated_token_address(&mock_user.pubkey(), &USDC);
    let user_token_y = get_associated_token_address(&mock_user.pubkey(), &USDT);

    let mut accounts = cpi_example::accounts::DlmmDeposit {
        lb_pair: lb_pair_address,
        position: position_keypair.pubkey(),
        bin_array_bitmap_extension: Some(dlmm::ID),
        user_token_x,
        user_token_y,
        user: mock_user.pubkey(),
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_program: dlmm::ID,
        dlmm_event_authority,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses {
        accounts.push(AccountMeta {
            pubkey: bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    let ix_data = cpi_example::instruction::DlmmDeposit {
        liquidity_parameter: LiquidityParameterByStrategy {
            strategy_parameters: StrategyParameters {
                min_bin_id: position_state.lower_bin_id,
                max_bin_id: position_state.upper_bin_id,
                strategy_type: StrategyType::SpotImBalanced,
                parameteres: [0u8; 64],
            },
            max_active_bin_slippage: 0,
            active_id: lb_pair_state.active_id,
            amount_y: 1_000_000_000,
            amount_x: 1_000_000_000,
        },
        remaining_account_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let deposit_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            deposit_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_deposit_position_pda() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Initialize position
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
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
        owner: mock_user.pubkey(),
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

    // 2. Deposit
    let position_account = banks_client.get_account(position).await.unwrap().unwrap();
    let position_state: PositionV2 = deserialize_zc_unalignment(&position_account).unwrap();

    let bin_array_addresses = create_position_required_bin_arrays(
        &mut banks_client,
        &position_state,
        lb_pair_address,
        &mock_user,
    )
    .await;

    let user_token_x = get_associated_token_address(&mock_user.pubkey(), &USDC);
    let user_token_y = get_associated_token_address(&mock_user.pubkey(), &USDT);

    let mut accounts = cpi_example::accounts::DlmmDepositToPositionPda {
        lb_pair: lb_pair_address,
        position,
        bin_array_bitmap_extension: Some(dlmm::ID),
        user_token_x,
        user_token_y,
        user: mock_user.pubkey(),
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_program: dlmm::ID,
        dlmm_event_authority,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses {
        accounts.push(AccountMeta {
            pubkey: bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    let ix_data = cpi_example::instruction::DlmmDepositToPositionPda {
        position_index: index,
        liquidity_parameter: LiquidityParameterByStrategy {
            strategy_parameters: StrategyParameters {
                min_bin_id: position_state.lower_bin_id,
                max_bin_id: position_state.upper_bin_id,
                strategy_type: StrategyType::SpotImBalanced,
                parameteres: [0u8; 64],
            },
            max_active_bin_slippage: 0,
            active_id: lb_pair_state.active_id,
            amount_y: 1_000_000_000,
            amount_x: 1_000_000_000,
        },
        remaining_account_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let deposit_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            deposit_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_deposit_to_position_with_pda_authority() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Initialize position
    let lb_pair_address = create_new_lb_pair(&mut banks_client, &mock_user).await;
    let (dlmm_event_authority, _bump) = derive_event_authority_pda();

    let position = Keypair::new();

    let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &cpi_example::ID);

    let accounts = cpi_example::accounts::InitializeDlmmPositionWithPdaOwner {
        position: position.pubkey(),
        lb_pair: lb_pair_address,
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
        authority,
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
        &[&mock_user, &position],
        &mut banks_client,
    )
    .await;

    // 2. Deposit
    let position_account = banks_client
        .get_account(position.pubkey())
        .await
        .unwrap()
        .unwrap();
    let position_state: PositionV2 = deserialize_zc_unalignment(&position_account).unwrap();

    let bin_array_addresses = create_position_required_bin_arrays(
        &mut banks_client,
        &position_state,
        lb_pair_address,
        &mock_user,
    )
    .await;

    let deposit_amount_x = 1_000_000_000;
    let deposit_amount_y = 1_000_000_000;

    let user_token_x = get_associated_token_address(&mock_user.pubkey(), &USDC);
    let user_token_y = get_associated_token_address(&mock_user.pubkey(), &USDT);

    let authority_token_x = get_associated_token_address(&authority, &USDC);
    let authority_token_y = get_associated_token_address(&authority, &USDT);

    // Assume the PDA authority already have the funds before hand
    let fund_authority_token_x = transfer(
        &anchor_spl::token::ID,
        &user_token_x,
        &authority_token_x,
        &mock_user.pubkey(),
        &[&mock_user.pubkey()],
        deposit_amount_x,
    )
    .unwrap();

    let fund_authority_token_y = transfer(
        &anchor_spl::token::ID,
        &user_token_y,
        &authority_token_y,
        &mock_user.pubkey(),
        &[&mock_user.pubkey()],
        deposit_amount_y,
    )
    .unwrap();

    let create_authority_token_x = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &USDC,
        &anchor_spl::token::ID,
    );

    let create_authority_token_y = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &USDT,
        &anchor_spl::token::ID,
    );

    process_and_assert_ok(
        &[
            create_authority_token_x,
            create_authority_token_y,
            fund_authority_token_x,
            fund_authority_token_y,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;

    let mut accounts = cpi_example::accounts::DlmmDepositToPositionWithPdaAuthority {
        lb_pair: lb_pair_address,
        position: position.pubkey(),
        bin_array_bitmap_extension: Some(dlmm::ID),
        authority_token_x,
        authority_token_y,
        authority,
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_program: dlmm::ID,
        dlmm_event_authority,
        admin: mock_user.pubkey(),
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses {
        accounts.push(AccountMeta {
            pubkey: bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    let ix_data = cpi_example::instruction::DlmmDepositToPositionWithPdaAuthority {
        liquidity_parameter: LiquidityParameterByStrategy {
            strategy_parameters: StrategyParameters {
                min_bin_id: position_state.lower_bin_id,
                max_bin_id: position_state.upper_bin_id,
                strategy_type: StrategyType::SpotImBalanced,
                parameteres: [0u8; 64],
            },
            max_active_bin_slippage: 0,
            active_id: lb_pair_state.active_id,
            amount_y: deposit_amount_x,
            amount_x: deposit_amount_y,
        },
        remaining_account_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let deposit_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            deposit_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_deposit_to_pda_position_with_pda_authority() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    setup_accounts_from_cluster(&mut test, &[PRESET_PARAMETER_2_BIN_STEP_1]).await;

    let mint_keys = vec![USDC, USDT];
    setup_mint_from_cluster(&mut test, &mint_keys, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Initialize position
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
        system_program: system_program::ID,
        payer: mock_user.pubkey(),
        dlmm_event_authority,
        rent: solana_sdk::sysvar::rent::ID,
        dlmm_program: dlmm::ID,
        authority,
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

    // 2. Deposit
    let position_account = banks_client.get_account(position).await.unwrap().unwrap();
    let position_state: PositionV2 = deserialize_zc_unalignment(&position_account).unwrap();

    let bin_array_addresses = create_position_required_bin_arrays(
        &mut banks_client,
        &position_state,
        lb_pair_address,
        &mock_user,
    )
    .await;

    let deposit_amount_x = 1_000_000_000;
    let deposit_amount_y = 1_000_000_000;

    let user_token_x = get_associated_token_address(&mock_user.pubkey(), &USDC);
    let user_token_y = get_associated_token_address(&mock_user.pubkey(), &USDT);

    let authority_token_x = get_associated_token_address(&authority, &USDC);
    let authority_token_y = get_associated_token_address(&authority, &USDT);

    let mut accounts = cpi_example::accounts::DlmmDepositToPdaPositionWithPdaAuthority {
        lb_pair: lb_pair_address,
        position,
        bin_array_bitmap_extension: Some(dlmm::ID),
        authority_token_x,
        authority_token_y,
        authority,
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_program: dlmm::ID,
        dlmm_event_authority,
        user: mock_user.pubkey(),
        user_token_x,
        user_token_y,
        system_program: system_program::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses {
        accounts.push(AccountMeta {
            pubkey: bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    let ix_data = cpi_example::instruction::DlmmDepositToPdaPositionWithPdaAuthority {
        position_index: index,
        liquidity_parameter: LiquidityParameterByStrategy {
            strategy_parameters: StrategyParameters {
                min_bin_id: position_state.lower_bin_id,
                max_bin_id: position_state.upper_bin_id,
                strategy_type: StrategyType::SpotImBalanced,
                parameteres: [0u8; 64],
            },
            max_active_bin_slippage: 0,
            active_id: lb_pair_state.active_id,
            amount_y: deposit_amount_x,
            amount_x: deposit_amount_y,
        },
        remaining_account_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let deposit_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            deposit_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}
