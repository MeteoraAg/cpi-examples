mod helpers;
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use dynamic_amm::instructions::CustomizableParams;
use dynamic_amm::state::Pool;
use dynamic_vault::state::Vault;
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

async fn generate_swap_fees(banks_client: &mut BanksClient, pool: Pubkey, user: &Keypair) {
    let pool_account = banks_client.get_account(pool).await.unwrap().unwrap();
    let pool_state = Pool::try_deserialize(&mut pool_account.data.as_ref()).unwrap();

    let a_vault_account = banks_client
        .get_account(pool_state.a_vault)
        .await
        .unwrap()
        .unwrap();
    let a_vault_state = Vault::try_deserialize(&mut a_vault_account.data.as_ref()).unwrap();

    let b_vault_account = banks_client
        .get_account(pool_state.b_vault)
        .await
        .unwrap()
        .unwrap();
    let b_vault_state = Vault::try_deserialize(&mut b_vault_account.data.as_ref()).unwrap();

    for (in_token, out_token) in [
        (pool_state.token_a_mint, pool_state.token_b_mint),
        (pool_state.token_b_mint, pool_state.token_a_mint),
    ] {
        let in_token_ata = get_associated_token_address(&user.pubkey(), &in_token);
        let out_token_ata = get_associated_token_address(&user.pubkey(), &out_token);

        let protocol_token_fee = if in_token.eq(&pool_state.token_a_mint) {
            pool_state.protocol_token_a_fee
        } else {
            pool_state.protocol_token_b_fee
        };

        let accounts = dynamic_amm::accounts::Swap {
            pool,
            user_source_token: in_token_ata,
            user_destination_token: out_token_ata,
            a_vault: pool_state.a_vault,
            b_vault: pool_state.b_vault,
            a_token_vault: a_vault_state.token_vault,
            b_token_vault: b_vault_state.token_vault,
            a_vault_lp: pool_state.a_vault_lp,
            b_vault_lp: pool_state.b_vault_lp,
            a_vault_lp_mint: a_vault_state.lp_mint,
            b_vault_lp_mint: b_vault_state.lp_mint,
            token_program: anchor_spl::token::ID,
            protocol_token_fee,
            user: user.pubkey(),
            vault_program: dynamic_vault::ID,
        }
        .to_account_metas(None);

        let ix_data = dynamic_amm::instruction::Swap {
            in_amount: 1_000_000,
            minimum_out_amount: 0,
        }
        .data();

        let instruction = Instruction {
            program_id: dynamic_amm::ID,
            accounts,
            data: ix_data,
        };

        process_and_assert_ok(&[instruction], user, &[&user], banks_client).await;
    }
}

#[tokio::test]
async fn test_claim_fee_pda_creator() {
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
    let usdc_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let (creator_authority, _bump) = Pubkey::find_program_address(&[b"creator"], &cpi_example::ID);

    let creator_token_a = get_associated_token_address(&creator_authority, &JUP);
    let creator_token_b = get_associated_token_address(&creator_authority, &USDC);

    let pool_key = derive_customizable_permissionless_constant_product_pool_key(JUP, USDC);
    let lp_mint = derive_lp_mint(pool_key);

    let creator_pool_lp = get_associated_token_address(&creator_authority, &lp_mint);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(usdc_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let metadata_pda = derive_metadata_key(lp_mint);

    // 1. Initialize pool
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

    // 2. Lock liquidity 50/50 to pda creator + user
    let user = Keypair::new();
    let allocations = [5000_u16; 2];

    let lock_escrow_creator = derive_lock_escrow(pool_key, creator_authority);
    let lock_escrow_0 = derive_lock_escrow(pool_key, user.pubkey());

    let escrow_vault_creator = get_associated_token_address(&lock_escrow_creator, &lp_mint);
    let escrow_vault_0 = get_associated_token_address(&lock_escrow_0, &lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmLockLiquidityPdaCreator {
        pool: pool_key,
        creator_authority,
        lock_escrow_creator,
        lp_mint,
        lock_escrow_0,
        source_lp_tokens: creator_pool_lp,
        escrow_vault_0,
        escrow_vault_creator,
        payer: mock_user.pubkey(),
        user_0: user.pubkey(),
        a_vault: jup_vault.key,
        b_vault: usdc_vault.key,
        a_vault_lp,
        b_vault_lp,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
        dynamic_amm_program: dynamic_amm::ID,
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

    process_and_assert_ok(&[instruction], &mock_user, &[&mock_user], &mut banks_client).await;

    // 3. Generate some swap fee
    generate_swap_fees(&mut banks_client, pool_key, &mock_user).await;

    // 4. Claim fee
    let accounts = cpi_example::accounts::DynamicAmmClaimFeePdaCreator {
        pool: pool_key,
        lp_mint,
        creator_authority,
        lock_escrow: lock_escrow_creator,
        escrow_vault: escrow_vault_creator,
        a_token_vault: jup_vault.vault_state.token_vault,
        b_token_vault: usdc_vault.vault_state.token_vault,
        cpi_example_admin: mock_user.pubkey(),
        a_vault: jup_vault.key,
        b_vault: usdc_vault.key,
        a_vault_lp,
        b_vault_lp,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
        creator_a_token: creator_token_a,
        creator_b_token: creator_token_b,
        token_program: anchor_spl::token::ID,
        dynamic_amm: dynamic_amm::ID,
        dynamic_vault: dynamic_vault::ID,
    }
    .to_account_metas(None);

    let ix_data = cpi_example::instruction::DynamicAmmClaimFeePdaCreator {}.data();

    let instruction = Instruction {
        program_id: cpi_example::ID,
        accounts,
        data: ix_data,
    };

    process_and_assert_ok(&[instruction], &mock_user, &[&mock_user], &mut banks_client).await;
}

#[tokio::test]
async fn test_claim_fee() {
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
    let usdc_vault = setup_vault_from_cluster(&mut test, USDC, mock_user.pubkey()).await;

    let (mut banks_client, _, _) = test.start().await;

    let pool_key = derive_customizable_permissionless_constant_product_pool_key(JUP, USDC);
    let lp_mint = derive_lp_mint(pool_key);

    let payer_pool_lp = get_associated_token_address(&mock_user.pubkey(), &lp_mint);

    let a_vault_lp = derive_vault_lp_key(jup_vault.key, pool_key);
    let b_vault_lp = derive_vault_lp_key(usdc_vault.key, pool_key);

    let protocol_token_a_fee = derive_protocol_fee_key(JUP, pool_key);
    let protocol_token_b_fee = derive_protocol_fee_key(USDC, pool_key);

    let payer_token_a = get_associated_token_address(&mock_user.pubkey(), &JUP);
    let payer_token_b = get_associated_token_address(&mock_user.pubkey(), &USDC);

    let metadata_pda = derive_metadata_key(lp_mint);

    // 1. Initialize pool
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

    // 2. Lock liquidity 50/50 to user 0 + user 1
    let user_0 = mock_user.pubkey();
    let user_1_kp = Keypair::new();
    let user_1 = user_1_kp.pubkey();

    let allocations = [5000_u16; 2];

    let lock_escrow_0 = derive_lock_escrow(pool_key, user_0);
    let lock_escrow_1 = derive_lock_escrow(pool_key, user_1);

    let escrow_vault_0 = get_associated_token_address(&lock_escrow_0, &lp_mint);
    let escrow_vault_1 = get_associated_token_address(&lock_escrow_1, &lp_mint);

    let accounts = cpi_example::accounts::DynamicAmmLockLiquidity {
        pool: pool_key,
        lock_escrow_1,
        lp_mint,
        lock_escrow_0,
        source_lp_tokens: payer_pool_lp,
        escrow_vault_0,
        escrow_vault_1,
        payer: mock_user.pubkey(),
        user_0,
        user_1,
        a_vault: jup_vault.key,
        b_vault: usdc_vault.key,
        a_vault_lp,
        b_vault_lp,
        a_vault_lp_mint: jup_vault.vault_state.lp_mint,
        b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
        dynamic_amm_program: dynamic_amm::ID,
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

    process_and_assert_ok(&[instruction], &mock_user, &[&mock_user], &mut banks_client).await;

    // 3. Generate some swap fees
    generate_swap_fees(&mut banks_client, pool_key, &mock_user).await;

    // 4. Claim fee for user 0 + 1
    for user in [&mock_user, &user_1_kp] {
        let lock_escrow = derive_lock_escrow(pool_key, user.pubkey());
        let escrow_vault = get_associated_token_address(&lock_escrow, &lp_mint);

        let user_token_a =
            get_associated_token_address(&user.pubkey(), &jup_vault.vault_state.token_mint);

        let user_token_b =
            get_associated_token_address(&user.pubkey(), &usdc_vault.vault_state.token_mint);

        let init_user_token_a_ix =
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &mock_user.pubkey(),
                &user.pubkey(),
                &jup_vault.vault_state.token_mint,
                &anchor_spl::token::ID,
            );

        let init_user_token_b_ix =
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &mock_user.pubkey(),
                &user.pubkey(),
                &usdc_vault.vault_state.token_mint,
                &anchor_spl::token::ID,
            );

        let accounts = cpi_example::accounts::DynamicAmmClaimFee {
            pool: pool_key,
            a_vault: jup_vault.key,
            b_vault: usdc_vault.key,
            a_vault_lp,
            b_vault_lp,
            a_token_vault: jup_vault.vault_state.token_vault,
            b_token_vault: usdc_vault.vault_state.token_vault,
            a_vault_lp_mint: jup_vault.vault_state.lp_mint,
            b_vault_lp_mint: usdc_vault.vault_state.lp_mint,
            lock_escrow,
            escrow_vault,
            lp_mint,
            owner: user.pubkey(),
            user_a_token: user_token_a,
            user_b_token: user_token_b,
            dynamic_amm: dynamic_amm::ID,
            dynamic_vault: dynamic_vault::ID,
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None);

        let ix_data = cpi_example::instruction::DynamicAmmClaimFee {}.data();

        let instruction = Instruction {
            program_id: cpi_example::ID,
            accounts,
            data: ix_data,
        };

        process_and_assert_ok(
            &[init_user_token_a_ix, init_user_token_b_ix, instruction],
            &mock_user,
            &[&mock_user, &user],
            &mut banks_client,
        )
        .await;
    }
}
