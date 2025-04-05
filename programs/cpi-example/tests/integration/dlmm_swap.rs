use crate::helpers;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
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
