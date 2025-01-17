mod helpers;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use dynamic_amm::instructions::CustomizableParams;
use helpers::dynamic_amm_utils::setup_vault_from_cluster;
use helpers::dynamic_amm_utils::*;
use helpers::process_and_assert_ok;
use helpers::*;
use solana_program_test::*;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{system_program, sysvar};

#[tokio::test]
async fn test_initialize_customizable_permissionless_pool_with_pda_creator() {
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

    let jup_vault = setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    let sol_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let (creator_authority, _bump) = Pubkey::find_program_address(&[b"creator"], &cpi_example::ID);

    let creator_token_a = get_associated_token_address(&creator_authority, &JUP);
    let creator_token_b = get_associated_token_address(&creator_authority, &USDC);

    let pool_key = derive_customizable_permissionless_constant_product_pool_key(JUP, USDC);
    let lp_mint = derive_lp_mint(pool_key);

    let creator_pool_lp = get_associated_token_address(&creator_authority, &lp_mint);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(sol_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let metadata_pda = derive_metadata_key(lp_mint);

    let accounts =
        cpi_example::accounts::DynamicAmmInitializeCustomizablePermissionlessPoolPdaCreator {
            pool: pool_key,
            creator_authority,
            creator_token_a,
            creator_token_b,
            lp_mint,
            token_a_mint: JUP,
            token_b_mint: USDC,
            a_vault: jup_vault.key,
            b_vault: sol_vault.key,
            a_token_vault: jup_vault.vault_state.token_vault,
            b_token_vault: sol_vault.vault_state.token_vault,
            a_vault_lp_mint: jup_vault.vault_state.lp_mint,
            b_vault_lp_mint: sol_vault.vault_state.lp_mint,
            payer: mock_user.pubkey(),
            token_program: anchor_spl::token::ID,
            a_vault_lp,
            b_vault_lp,
            protocol_token_a_fee,
            protocol_token_b_fee,
            creator_pool_lp,
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

    let ix_data =
        cpi_example::instruction::InitializeDynamicAmmCustomizablePermissionlessPoolPdaCreator {
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
}

#[tokio::test]
async fn test_initialize_customizable_permissionless_pool() {
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

    let jup_vault = setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    let sol_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let pool_key = derive_customizable_permissionless_constant_product_pool_key(JUP, USDC);
    let lp_mint = derive_lp_mint(pool_key);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(sol_vault.key, pool_key);

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
        b_vault: sol_vault.key,
        a_token_vault: jup_vault.vault_state.token_vault,
        b_token_vault: sol_vault.vault_state.token_vault,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: sol_vault.vault_state.lp_mint,
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
}

#[tokio::test]
async fn test_initialize_permissionless_pool_with_config() {
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

    let jup_vault = setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    let sol_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    setup_pool_config_from_cluster(&mut test, CONFIG).await;

    let (mut banks_client, _, _) = test.start().await;

    let pool_key = derive_permissionless_constant_product_pool_with_config_key(JUP, USDC, CONFIG);
    let lp_mint = derive_lp_mint(pool_key);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(sol_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let payer_pool_lp = get_associated_token_address(&mock_user.pubkey(), &lp_mint);

    let metadata_pda = derive_metadata_key(lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmInitializePermissionlessPoolWithConfig {
        pool: pool_key,
        lp_mint,
        token_a_mint: JUP,
        token_b_mint: USDC,
        a_vault: jup_vault.key,
        b_vault: sol_vault.key,
        a_token_vault: jup_vault.vault_state.token_vault,
        b_token_vault: sol_vault.vault_state.token_vault,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: sol_vault.vault_state.lp_mint,
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
        config: CONFIG,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::InitializeDynamicAmmPermissionPoolWithConfig {
        token_a_amount: 100_000_000,
        token_b_amount: 100_000_000,
        activation_point: None,
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
}

#[tokio::test]
async fn test_initialize_permissionless_pool_with_config_pda_pool_creator() {
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

    let jup_vault = setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    let sol_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;
    setup_pool_config_from_cluster(&mut test, CONFIG).await;

    let (mut banks_client, _, _) = test.start().await;

    let (creator_authority, _bump) = Pubkey::find_program_address(&[b"creator"], &cpi_example::ID);

    let creator_token_a = get_associated_token_address(&creator_authority, &JUP);
    let creator_token_b = get_associated_token_address(&creator_authority, &USDC);

    let pool_key = derive_permissionless_constant_product_pool_with_config_key(JUP, USDC, CONFIG);
    let lp_mint = derive_lp_mint(pool_key);

    let creator_pool_lp = get_associated_token_address(&creator_authority, &lp_mint);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(sol_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let metadata_pda = derive_metadata_key(lp_mint);

    let accounts =
        cpi_example::accounts::DynamicAmmInitializePermissionlessPoolWithConfigPoolPdaCreator {
            pool: pool_key,
            creator_authority,
            creator_token_a,
            creator_token_b,
            lp_mint,
            token_a_mint: JUP,
            token_b_mint: USDC,
            a_vault: jup_vault.key,
            b_vault: sol_vault.key,
            a_token_vault: jup_vault.vault_state.token_vault,
            b_token_vault: sol_vault.vault_state.token_vault,
            a_vault_lp_mint: jup_vault.vault_state.lp_mint,
            b_vault_lp_mint: sol_vault.vault_state.lp_mint,
            payer: mock_user.pubkey(),
            token_program: anchor_spl::token::ID,
            a_vault_lp,
            b_vault_lp,
            protocol_token_a_fee,
            protocol_token_b_fee,
            creator_pool_lp,
            config: CONFIG,
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

    let ix_data =
        cpi_example::instruction::InitializeDynamicAmmPermissionPoolWithConfigPdaCreator {
            token_a_amount: 100_000_000,
            token_b_amount: 100_000_000,
            activation_point: None,
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
}
