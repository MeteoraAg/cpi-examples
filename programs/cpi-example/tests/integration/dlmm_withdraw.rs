use crate::helpers::deserialize_zc_unalignment;
use crate::{helpers, PRESET_PARAMETER_2_BIN_STEP_1, USDC, USDT};
use anchor_lang::prelude::AccountMeta;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use cpi_example::dlmm;
use cpi_example::dlmm::accounts::LbPair;
use cpi_example::dlmm::accounts::PositionV2;
use cpi_example::dlmm::constants::BASIS_POINT_MAX;
use cpi_example::dlmm::types::RemainingAccountsInfo;
use cpi_example::dlmm::types::StrategyParameters;
use cpi_example::dlmm::types::StrategyType;
use cpi_example::dlmm::types::{BinLiquidityReduction, LiquidityParameterByStrategy};
use cpi_example::dlmm_withdraw::RemoveSingleSidedArgs;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_cpi_example_program};
use solana_program_test::*;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::system_program;
use solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

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
async fn test_dlmm_withdraw() {
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
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
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

    // 3. Withdraw
    let mut accounts = cpi_example::accounts::DlmmRemoveLiquidity {
        position: position_keypair.pubkey(),
        lb_pair: lb_pair_address,
        bin_array_bitmap_extension: Some(dlmm::ID),
        user_token_x,
        user_token_y,
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        sender: mock_user.pubkey(),
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        dlmm_event_authority,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    // Remove all liquidity
    let mut bin_liquidity_removal = vec![];
    for bin_id in position_state.lower_bin_id..=position_state.upper_bin_id {
        let bin_liquidity_reduction = BinLiquidityReduction {
            bin_id,
            bps_to_remove: BASIS_POINT_MAX as u16,
        };
        bin_liquidity_removal.push(bin_liquidity_reduction);
    }

    let ix_data = cpi_example::instruction::DlmmWithdraw {
        bin_liquidity_removal,
        remaining_accounts_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let withdraw_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            withdraw_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_withdraw_with_pda_authority() {
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
    let authority = Pubkey::find_program_address(&[b"authority"], &cpi_example::ID).0;

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

    let amount_x = 1_000_000_000;
    let amount_y = 1_000_000_000;

    let user_token_x = get_associated_token_address(&mock_user.pubkey(), &USDC);
    let user_token_y = get_associated_token_address(&mock_user.pubkey(), &USDT);

    let authority_token_x = get_associated_token_address(&authority, &lb_pair_state.token_x_mint);
    let authority_token_y = get_associated_token_address(&authority, &lb_pair_state.token_y_mint);

    let init_authority_token_x_ix = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &lb_pair_state.token_x_mint,
        &anchor_spl::token::ID,
    );

    let init_authority_token_y_ix = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &lb_pair_state.token_y_mint,
        &anchor_spl::token::ID,
    );

    let fund_authority_token_x_ix = anchor_spl::token::spl_token::instruction::transfer(
        &anchor_spl::token::ID,
        &user_token_x,
        &authority_token_x,
        &mock_user.pubkey(),
        &[],
        amount_x,
    )
    .unwrap();

    let fund_authority_token_y_ix = anchor_spl::token::spl_token::instruction::transfer(
        &anchor_spl::token::ID,
        &user_token_y,
        &authority_token_y,
        &mock_user.pubkey(),
        &[],
        amount_y,
    )
    .unwrap();

    let mut accounts = cpi_example::accounts::DlmmDepositToPositionWithPdaAuthority {
        lb_pair: lb_pair_address,
        position: position_keypair.pubkey(),
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
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
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
            amount_y,
            amount_x,
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
            ComputeBudgetInstruction::set_compute_unit_limit(600_000),
            init_authority_token_x_ix,
            init_authority_token_y_ix,
            fund_authority_token_x_ix,
            fund_authority_token_y_ix,
            deposit_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;

    // 3. Withdraw
    let mut accounts = cpi_example::accounts::DlmmRemoveLiquidityWithPdaAuthority {
        position: position_keypair.pubkey(),
        lb_pair: lb_pair_address,
        bin_array_bitmap_extension: Some(dlmm::ID),
        authority,
        authority_token_x,
        authority_token_y,
        admin: mock_user.pubkey(),
        associated_token_program: spl_associated_token_account::ID,
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        dlmm_event_authority,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    // Remove all liquidity
    let mut bin_liquidity_removal = vec![];
    for bin_id in position_state.lower_bin_id..=position_state.upper_bin_id {
        let bin_liquidity_reduction = BinLiquidityReduction {
            bin_id,
            bps_to_remove: BASIS_POINT_MAX as u16,
        };
        bin_liquidity_removal.push(bin_liquidity_reduction);
    }

    let ix_data = cpi_example::instruction::DlmmWithdrawWithPdaAuthority {
        bin_liquidity_removal,
        remaining_accounts_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let withdraw_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            withdraw_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_withdraw_single_sided() {
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
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
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

    // 3. Withdraw single sided
    let mut accounts = cpi_example::accounts::DlmmRemoveLiquiditySingleSided {
        position: position_keypair.pubkey(),
        lb_pair: lb_pair_address,
        bin_array_bitmap_extension: Some(dlmm::ID),
        user_token_x,
        user_token_y,
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        sender: mock_user.pubkey(),
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        dlmm_event_authority,
        dlmm_program: dlmm::ID,
    }
    .to_account_metas(None);

    // Add bin arrays to remaining accounts
    for bin_array_address in bin_array_addresses.iter() {
        accounts.push(AccountMeta {
            pubkey: *bin_array_address,
            is_signer: false,
            is_writable: true,
        });
    }

    // Remove only ask side
    let ix_data = cpi_example::instruction::DlmmWithdrawSingleSided {
        args: RemoveSingleSidedArgs { side: 0 },
        remaining_accounts_info: RemainingAccountsInfo { slices: vec![] },
    }
    .data();

    let withdraw_ix = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(500_000),
            withdraw_ix,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}
