use crate::helpers;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use cpi_example::dynamic_amm::types::CustomizableParams;
use helpers::dynamic_amm_ix_account_builder::IxAccountBuilder;
use helpers::dynamic_amm_pda::{derive_lock_escrow_key, METAPLEX_PROGRAM_ID};
use helpers::dynamic_amm_utils::setup_vault_from_cluster;
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
async fn test_lock_liquidity_pda_creator() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();
    test.prefer_bpf(true);

    test.add_program("dynamic_amm", cpi_example::dynamic_amm::ID, None);
    test.add_program("dynamic_vault", cpi_example::dynamic_vault::ID, None);
    test.add_program("metaplex", METAPLEX_PROGRAM_ID, None);

    setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let (creator_authority, _bump) = Pubkey::find_program_address(&[b"creator"], &cpi_example::ID);

    let init_pool_accounts =
        IxAccountBuilder::initialize_customizable_permissionless_constant_product_pool(
            JUP,
            USDC,
            creator_authority,
        );

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    // 1. Initialize pool
    let accounts =
        cpi_example::accounts::DynamicAmmInitializeCustomizablePermissionlessPoolPdaCreator {
            pool: init_pool_accounts.pool,
            creator_authority,
            creator_token_a: init_pool_accounts.payer_token_a,
            creator_token_b: init_pool_accounts.payer_token_b,
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
            creator_pool_lp: init_pool_accounts.payer_pool_lp,
            payer_token_a,
            payer_token_b,
            rent: sysvar::rent::ID,
            metadata_program: METAPLEX_PROGRAM_ID,
            mint_metadata: init_pool_accounts.mint_metadata,
            vault_program: cpi_example::dynamic_vault::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
            dynamic_amm_program: cpi_example::dynamic_amm::ID,
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

    // 2. Lock liquidity 50/50 to pda creator + user
    let user = Keypair::new();
    let allocations = [5000_u16; 2];

    let lock_escrow_creator = derive_lock_escrow_key(init_pool_accounts.pool, creator_authority);
    let lock_escrow_0 = derive_lock_escrow_key(init_pool_accounts.pool, user.pubkey());

    let escrow_vault_creator =
        get_associated_token_address(&lock_escrow_creator, &init_pool_accounts.lp_mint);
    let escrow_vault_0 = get_associated_token_address(&lock_escrow_0, &init_pool_accounts.lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmLockLiquidityPdaCreator {
        pool: init_pool_accounts.pool,
        creator_authority,
        lock_escrow_creator,
        lp_mint: init_pool_accounts.lp_mint,
        lock_escrow_0,
        source_lp_tokens: init_pool_accounts.payer_pool_lp,
        escrow_vault_0,
        escrow_vault_creator,
        payer: mock_user.pubkey(),
        user_0: user.pubkey(),
        a_vault: init_pool_accounts.a_vault,
        b_vault: init_pool_accounts.b_vault,
        a_vault_lp: init_pool_accounts.a_vault_lp,
        b_vault_lp: init_pool_accounts.b_vault_lp,
        a_vault_lp_mint: init_pool_accounts.a_vault_lp_mint,
        b_vault_lp_mint: init_pool_accounts.b_vault_lp_mint,
        dynamic_amm_program: cpi_example::dynamic_amm::ID,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
    }
    .to_account_metas(None);

    let ix_data =
        cpi_example::instruction::DynamicAmmLockLiquidityPdaCreator { allocations }.data();

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
async fn test_lock_liquidity() {
    let mock_user = Keypair::new();

    let mut test = setup_cpi_example_program();
    test.prefer_bpf(true);

    test.add_program("dynamic_amm", cpi_example::dynamic_amm::ID, None);
    test.add_program("dynamic_vault", cpi_example::dynamic_vault::ID, None);
    test.add_program("metaplex", METAPLEX_PROGRAM_ID, None);

    setup_vault_from_cluster(&mut test, JUP, mock_user.pubkey()).await;
    setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let init_pool_accounts =
        IxAccountBuilder::initialize_customizable_permissionless_constant_product_pool(
            JUP,
            USDC,
            mock_user.pubkey(),
        );

    // 1. Initialize pool
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
        vault_program: cpi_example::dynamic_vault::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        system_program: system_program::ID,
        dynamic_amm_program: cpi_example::dynamic_amm::ID,
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

    // 2. Lock liquidity 50/50 to user 0 + user 1
    let user_0 = mock_user.pubkey();
    let user_1_kp = Keypair::new();
    let user_1 = user_1_kp.pubkey();

    let allocations = [5000_u16; 2];

    let lock_escrow_0 = derive_lock_escrow_key(init_pool_accounts.pool, user_0);
    let lock_escrow_1 = derive_lock_escrow_key(init_pool_accounts.pool, user_1);

    let escrow_vault_0 = get_associated_token_address(&lock_escrow_0, &init_pool_accounts.lp_mint);
    let escrow_vault_1 = get_associated_token_address(&lock_escrow_1, &init_pool_accounts.lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmLockLiquidity {
        pool: init_pool_accounts.pool,
        lock_escrow_1,
        lp_mint: init_pool_accounts.lp_mint,
        lock_escrow_0,
        source_lp_tokens: init_pool_accounts.payer_pool_lp,
        escrow_vault_0,
        escrow_vault_1,
        payer: mock_user.pubkey(),
        user_0,
        user_1,
        a_vault: init_pool_accounts.a_vault,
        b_vault: init_pool_accounts.b_vault,
        a_vault_lp: init_pool_accounts.a_vault_lp,
        b_vault_lp: init_pool_accounts.b_vault_lp,
        a_vault_lp_mint: init_pool_accounts.a_vault_lp_mint,
        b_vault_lp_mint: init_pool_accounts.b_vault_lp_mint,
        dynamic_amm_program: cpi_example::dynamic_amm::ID,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::DynamicAmmLockLiquidity { allocations }.data();

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
