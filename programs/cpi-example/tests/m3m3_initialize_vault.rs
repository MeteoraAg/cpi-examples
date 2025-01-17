mod helpers;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use dynamic_amm::instructions::CustomizableParams;
use helpers::dynamic_amm_utils::setup_vault_from_cluster;
use helpers::dynamic_amm_utils::*;
use helpers::process_and_assert_ok;
use helpers::*;
use m3m3::InitializeVaultParams;
use m3m3_utils::derive_full_balance_list_key;
use m3m3_utils::derive_m3m3_event_authority_key;
use m3m3_utils::derive_m3m3_vault_key;
use m3m3_utils::derive_top_staker_list_key;
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

    let jup_vault = setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    let usdc_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    // 1. Create pool
    let pool_key = derive_customizable_permissionless_constant_product_pool_key(JUP, USDC);
    let lp_mint = derive_lp_mint(pool_key);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(usdc_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let payer_pool_lp = get_associated_token_address(&mock_user.pubkey(), &lp_mint);

    let metadata_pda = derive_metadata_key(lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmInitializeCustomizablePermissionlessPool {
        pool: pool_key,
        lp_mint,
        token_a_mint: JUP,
        token_b_mint: USDC,
        a_vault: jup_vault.key,
        b_vault: usdc_vault.key,
        a_token_vault: jup_vault.vault_state.token_vault,
        b_token_vault: usdc_vault.vault_state.token_vault,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
        payer: mock_user.pubkey(),
        token_program: anchor_spl::token::ID,
        a_vault_lp,
        b_vault_lp,
        protocol_token_a_fee,
        protocol_token_b_fee,
        payer_pool_lp,
        payer_token_a,
        payer_token_b,
        rent: sysvar::rent::ID,
        metadata_program: METAPLEX_PROGRAM_ID,
        mint_metadata: metadata_pda,
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
    let m3m3_vault = derive_m3m3_vault_key(pool_key);
    let lock_escrow = derive_lock_escrow(pool_key, m3m3_vault);
    let escrow_vault = get_associated_token_address(&lock_escrow, &lp_mint);
    let m3m3_event_authority = derive_m3m3_event_authority_key();
    let stake_token_vault =
        get_associated_token_address(&m3m3_vault, &jup_vault.vault_state.token_mint);
    let quote_token_vault =
        get_associated_token_address(&m3m3_vault, &usdc_vault.vault_state.token_mint);
    let top_staker_list = derive_top_staker_list_key(m3m3_vault);
    let full_balance_list = derive_full_balance_list_key(m3m3_vault);

    let accounts = cpi_example::accounts::InitializeM3m3Vault {
        pool: pool_key,
        lock_escrow,
        a_vault: jup_vault.key,
        b_vault: usdc_vault.key,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
        a_vault_lp,
        b_vault_lp,
        lp_mint,
        source_lp_tokens: payer_pool_lp,
        payer: mock_user.pubkey(),
        token_program: anchor_spl::token::ID,
        escrow_vault,
        m3m3_event_authority,
        stake_mint: jup_vault.vault_state.token_mint,
        stake_token_vault,
        top_staker_list,
        full_balance_list,
        m3m3_vault,
        quote_mint: usdc_vault.vault_state.token_mint,
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
