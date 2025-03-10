mod helpers;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use dynamic_amm::instructions::CustomizableParams;
use dynamic_amm_common::dynamic_amm::ix_account_builder::IxAccountBuilder;
use dynamic_amm_common::dynamic_amm::pda::derive_lock_escrow_key;
use dynamic_amm_common::dynamic_amm::pda::METAPLEX_PROGRAM_ID;
use helpers::dynamic_amm_utils::setup_vault_from_cluster;
use helpers::process_and_assert_ok;
use helpers::*;
use m3m3::InitializeVaultParams;
use m3m3_common::pda::*;

use solana_program_test::*;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{system_program, sysvar};

#[tokio::test]
async fn test_initialize_m3m3_vault() {
    let mock_user = Keypair::new();

    let mut test = ProgramTest::new(
        "cpi_example",
        cpi_example::id(),
        processor!(cpi_example::entry),
    );
    test.prefer_bpf(true);

    test.add_program("dynamic_amm", dynamic_amm::ID, None);
    test.add_program("dynamic_vault", dynamic_vault::ID, None);
    test.add_program("metaplex", METAPLEX_PROGRAM_ID, None);
    test.add_program("m3m3", m3m3::ID, None);

    setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Create pool

    let init_pool_accounts =
        IxAccountBuilder::initialize_customizable_permissionless_constant_product_pool(
            JUP,
            USDC,
            mock_user.pubkey(),
        );

    let accounts = cpi_example::accounts::DynamicAmmInitializeCustomizablePermissionlessPool {
        pool: init_pool_accounts.pool,
        lp_mint: init_pool_accounts.lp_mint,
        token_a_mint: init_pool_accounts.token_a_mint,
        token_b_mint: init_pool_accounts.token_b_mint,
        a_vault: init_pool_accounts.a_vault,
        b_vault: init_pool_accounts.b_vault,
        a_token_vault: init_pool_accounts.a_token_vault,
        b_token_vault: init_pool_accounts.b_token_vault,
        a_vault_lp_mint: init_pool_accounts.a_vault_lp_mint,
        b_vault_lp_mint: init_pool_accounts.b_vault_lp_mint,
        payer: mock_user.pubkey(),
        token_program: anchor_spl::token::ID,
        a_vault_lp: init_pool_accounts.a_vault_lp,
        b_vault_lp: init_pool_accounts.b_vault_lp,
        protocol_token_a_fee: init_pool_accounts.protocol_token_a_fee,
        protocol_token_b_fee: init_pool_accounts.protocol_token_b_fee,
        payer_pool_lp: init_pool_accounts.payer_pool_lp,
        payer_token_a: init_pool_accounts.payer_token_a,
        payer_token_b: init_pool_accounts.payer_token_b,
        rent: sysvar::rent::ID,
        metadata_program: METAPLEX_PROGRAM_ID,
        mint_metadata: init_pool_accounts.mint_metadata,
        vault_program: dynamic_vault::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        system_program: system_program::ID,
        dynamic_amm_program: dynamic_amm::ID,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::InitializeDynamicAmmCustomizablePermissionlessPool {
        token_a_amount: 100_000_000,
        token_b_amount: 100_000_000,
        params: CustomizableParams {
            trade_fee_numerator: 10_000,
            activation_point: None,
            has_alpha_vault: false,
            activation_type: 1,
            padding: [0u8; 90],
        },
    }
    .data();

    let instruction = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
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

    // 2. Create lock escrow + lock + initialize m3m3 vault
    let m3m3_vault = derive_m3m3_vault_key(init_pool_accounts.pool);
    let lock_escrow = derive_lock_escrow_key(init_pool_accounts.pool, m3m3_vault);
    let escrow_vault = get_associated_token_address(&lock_escrow, &init_pool_accounts.lp_mint);
    let m3m3_event_authority = derive_m3m3_event_authority_key();
    let stake_token_vault =
        get_associated_token_address(&m3m3_vault, &init_pool_accounts.token_a_mint);
    let quote_token_vault =
        get_associated_token_address(&m3m3_vault, &init_pool_accounts.token_b_mint);
    let top_staker_list = derive_top_staker_list_key(m3m3_vault);
    let full_balance_list = derive_full_balance_list_key(m3m3_vault);

    let accounts = cpi_example::accounts::InitializeM3m3Vault {
        pool: init_pool_accounts.pool,
        lock_escrow,
        a_vault: init_pool_accounts.a_vault,
        b_vault: init_pool_accounts.b_vault,
        a_vault_lp_mint: init_pool_accounts.a_vault_lp_mint,
        b_vault_lp_mint: init_pool_accounts.b_vault_lp_mint,
        a_vault_lp: init_pool_accounts.a_vault_lp,
        b_vault_lp: init_pool_accounts.b_vault_lp,
        lp_mint: init_pool_accounts.lp_mint,
        source_lp_tokens: init_pool_accounts.payer_pool_lp,
        payer: mock_user.pubkey(),
        token_program: anchor_spl::token::ID,
        escrow_vault,
        m3m3_event_authority,
        stake_mint: init_pool_accounts.token_a_mint,
        stake_token_vault,
        top_staker_list,
        full_balance_list,
        m3m3_vault,
        quote_mint: init_pool_accounts.token_b_mint,
        quote_token_vault,
        system_program: system_program::ID,
        dynamic_amm_program: dynamic_amm::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        m3m3_program: m3m3::ID,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::InitializeM3m3Vault {
        max_amount: u64::MAX,
        vault_params: InitializeVaultParams {
            top_list_length: 999,
            seconds_to_full_unlock: 86400 * 7,
            unstake_lock_duration: 86400,
            start_fee_distribute_timestamp: None,
            padding: [0u8; 64],
        },
    };

    let instruction = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data.data(),
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
