use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
mod helpers;
use helpers::dynamic_amm_utils::{setup_pool_from_cluster, SetupContext};
use helpers::process_and_assert_ok;
use solana_program_test::*;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer,
};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("32D4zRxNc1EssbJieVHfPhZM3rH6CzfUPrWUuWxD9prG");

#[tokio::test]
async fn dlmm_swap() {
    let mock_user = Keypair::new();

    let mut test = ProgramTest::new(
        "cpi_example",
        cpi_example::id(),
        processor!(cpi_example::entry),
    );

    test.add_program("dynamic_amm", dynamic_amm::ID, None);
    test.add_program("dynamic_vault", dynamic_vault::ID, None);

    let SetupContext {
        pool_state,
        a_vault_state,
        b_vault_state,
        user_token_a,
        user_token_b,
    } = setup_pool_from_cluster(&mut test, USDC_USDT_POOL, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let ix_data = cpi_example::instruction::DynamicAmmSwap {
        amount_in: 1_000_000,
        min_amount_out: 0,
    }
    .data();

    let accounts = cpi_example::accounts::DynamicAmmSwap {
        pool: USDC_USDT_POOL,
        a_vault: pool_state.a_vault,
        b_vault: pool_state.b_vault,
        a_token_vault: a_vault_state.token_vault,
        b_token_vault: b_vault_state.token_vault,
        a_vault_lp: pool_state.a_vault_lp,
        b_vault_lp: pool_state.b_vault_lp,
        a_vault_lp_mint: a_vault_state.lp_mint,
        b_vault_lp_mint: b_vault_state.lp_mint,
        admin_token_fee: pool_state.admin_token_a_fee,
        user_source_token: user_token_a,
        user_destination_token: user_token_b,
        user: mock_user.pubkey(),
        dynamic_amm_program: dynamic_amm::ID,
        vault_program: dynamic_vault::ID,
        token_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

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
