use crate::helpers;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use anchor_spl::token::spl_token::instruction::transfer;
use cpi_example::dlmm;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_cpi_example_program};
use solana_program_test::*;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");

#[tokio::test]
async fn test_dlmm_swap() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    let PoolSetupContext {
        pool_state,
        user_token_x,
        user_token_y,
    } = setup_pool_from_cluster(&mut test, USDC_USDT_POOL, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let ix_data = cpi_example::instruction::DlmmSwap {
        amount_in: 1_000_000,
        min_amount_out: 0,
    }
    .data();

    let mut accounts = cpi_example::accounts::DlmmSwap {
        lb_pair: USDC_USDT_POOL,
        bin_array_bitmap_extension: None,
        reserve_x: pool_state.reserve_x,
        reserve_y: pool_state.reserve_y,
        user_token_in: user_token_x,
        user_token_out: user_token_y,
        token_x_mint: pool_state.token_x_mint,
        token_y_mint: pool_state.token_y_mint,
        oracle: pool_state.oracle,
        host_fee_in: None,
        user: mock_user.pubkey(),
        dlmm_program: dlmm::ID,
        event_authority: derive_event_authority_pda().0,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let (active_bin_array_key, _bump) = derive_bin_array_pda(
        USDC_USDT_POOL,
        bin_id_to_bin_array_index(pool_state.active_id)
            .unwrap()
            .into(),
    );

    accounts.push(AccountMeta::new(active_bin_array_key, false));

    let instruction = Instruction {
        program_id: cpi_example::id(),
        data: ix_data,
        accounts,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
            instruction,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}

#[tokio::test]
async fn test_dlmm_swap_with_pda_authority() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    let PoolSetupContext {
        pool_state,
        user_token_x,
        ..
    } = setup_pool_from_cluster(&mut test, USDC_USDT_POOL, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let amount_in = 1_000_000;

    let ix_data = cpi_example::instruction::DlmmSwapWithPdaAuthority {
        amount_in,
        min_amount_out: 0,
    }
    .data();

    let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &cpi_example::ID);

    let authority_token_in = get_associated_token_address_with_program_id(
        &authority,
        &pool_state.token_x_mint,
        &anchor_spl::token::ID,
    );

    let authority_token_out = get_associated_token_address_with_program_id(
        &authority,
        &pool_state.token_y_mint,
        &anchor_spl::token::ID,
    );

    let mut accounts = cpi_example::accounts::DlmmSwapWithPdaAuthority {
        lb_pair: USDC_USDT_POOL,
        bin_array_bitmap_extension: None,
        reserve_x: pool_state.reserve_x,
        reserve_y: pool_state.reserve_y,
        authority,
        authority_token_in,
        authority_token_out,
        token_x_mint: pool_state.token_x_mint,
        token_y_mint: pool_state.token_y_mint,
        oracle: pool_state.oracle,
        host_fee_in: None,
        dlmm_program: dlmm::ID,
        event_authority: derive_event_authority_pda().0,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let (active_bin_array_key, _bump) = derive_bin_array_pda(
        USDC_USDT_POOL,
        bin_id_to_bin_array_index(pool_state.active_id)
            .unwrap()
            .into(),
    );

    accounts.push(AccountMeta::new(active_bin_array_key, false));

    let swap_ix = Instruction {
        program_id: cpi_example::id(),
        data: ix_data,
        accounts,
    };

    let mut instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)];

    let create_authority_user_token_in_ix = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &pool_state.token_x_mint,
        &anchor_spl::token::ID,
    );

    let create_authority_user_token_out_ix = create_associated_token_account_idempotent(
        &mock_user.pubkey(),
        &authority,
        &pool_state.token_y_mint,
        &anchor_spl::token::ID,
    );

    instructions.push(create_authority_user_token_in_ix);
    instructions.push(create_authority_user_token_out_ix);

    // As example, we transfer some token for the authority token accounts.
    let authority_fund_ix = transfer(
        &anchor_spl::token::ID,
        &user_token_x,
        &authority_token_in,
        &mock_user.pubkey(),
        &[&mock_user.pubkey()],
        amount_in,
    )
    .unwrap();

    instructions.push(authority_fund_ix);
    instructions.push(swap_ix);

    process_and_assert_ok(&instructions, &mock_user, &[&mock_user], &mut banks_client).await;
}
