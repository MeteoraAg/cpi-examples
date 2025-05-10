// https://github.com/coral-xyz/anchor/issues/3401#issuecomment-2513466441
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

mod utils;

declare_program!(dlmm);
declare_program!(dynamic_amm);
declare_program!(dynamic_vault);
declare_program!(m3m3);

use crate::dlmm_swap::*;
use crate::dynamic_amm_swap::*;

fn assert_eq_admin(_key: Pubkey) -> bool {
    true
}

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod cpi_example {
    use super::*;

    pub fn initialize_lb_pair(
        ctx: Context<InitializeLbPair>,
        params: dlmm::types::InitializeLbPair2Params,
    ) -> Result<()> {
        instructions::dlmm_cpi::handle_initialize_lb_pair(ctx, params)
    }

    // NOTE: Pool creator will be PDA
    pub fn initialize_lb_pair_with_pda_creator(
        ctx: Context<InitializeLbPairWithPdaCreator>,
        params: dlmm::types::InitializeLbPair2Params,
    ) -> Result<()> {
        instructions::dlmm_cpi::handle_initialize_lb_pair_with_pda_creator(ctx, params)
    }

    pub fn initialize_position(
        ctx: Context<InitializePosition>,
        lower_bin_id: i32,
        width: i32,
    ) -> Result<()> {
        instructions::dlmm_cpi::handle_initialize_position(ctx, lower_bin_id, width)
    }

    // NOTE: Position owner will be PDA
    pub fn initialize_position_with_pda_owner(
        ctx: Context<InitializePositionWithPdaOwner>,
        lower_bin_id: i32,
        width: i32,
    ) -> Result<()> {
        instructions::dlmm_cpi::handle_initialize_position_with_pda_owner(ctx, lower_bin_id, width)
    }

    // NOTE: Position and position's owner will be PDA
    pub fn initialize_pda_position_with_pda_owner(
        ctx: Context<InitializePdaPositionWithPdaOwner>,
        index: i64,
        lower_bin_id: i32,
        width: i32,
    ) -> Result<()> {
        instructions::dlmm_cpi::handle_initialize_pda_position_with_pda_owner(
            ctx,
            index,
            lower_bin_id,
            width,
        )
    }

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dlmm_cpi::dlmm_swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }

    pub fn dlmm_swap_with_pda_authority<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwapWithPdaAuthority<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dlmm_cpi::dlmm_swap::handle_dlmm_swap_with_pda_authority(
            ctx,
            amount_in,
            min_amount_out,
        )
    }

    pub fn initialize_dynamic_amm_customizable_permissionless_pool(
        ctx: Context<DynamicAmmInitializeCustomizablePermissionlessPool>,
        token_a_amount: u64,
        token_b_amount: u64,
        params: dynamic_amm::types::CustomizableParams,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_customizable_permissionless_pool::handle_initialize_customizable_permissionless_pool(
            ctx,
            token_a_amount,
            token_b_amount,
            params,
        )
    }

    // NOTE: Creator authority PDA will be holding the LP
    pub fn initialize_dynamic_amm_customizable_permissionless_pool_pda_creator(
        ctx: Context<DynamicAmmInitializeCustomizablePermissionlessPoolPdaCreator>,
        token_a_amount: u64,
        token_b_amount: u64,
        params: dynamic_amm::types::CustomizableParams,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_customizable_permissionless_pool::handle_initialize_customizable_permissionless_pool_with_pda_creator(
            ctx, token_a_amount, token_b_amount, params
        )
    }

    pub fn initialize_dynamic_amm_permission_pool_with_config(
        ctx: Context<DynamicAmmInitializePermissionlessPoolWithConfig>,
        token_a_amount: u64,
        token_b_amount: u64,
        activation_point: Option<u64>,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_permissionless_pool_with_config::handle_initialize_permissionless_pool_with_config(
            ctx,
            token_a_amount,
            token_b_amount,
            activation_point,
        )
    }

    // NOTE: Creator authority PDA will be holding the LP
    pub fn initialize_dynamic_amm_permission_pool_with_config_pda_creator(
        ctx: Context<DynamicAmmInitializePermissionlessPoolWithConfigPdaCreator>,
        token_a_amount: u64,
        token_b_amount: u64,
        activation_point: Option<u64>,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_permissionless_pool_with_config::handle_initialize_permissionless_pool_with_pda_creator(
            ctx,
            token_a_amount,
            token_b_amount,
            activation_point,
        )
    }

    pub fn initialize_m3m3_vault(
        ctx: Context<InitializeM3m3Vault>,
        max_amount: u64,
        vault_params: m3m3::types::InitializeVaultParams,
    ) -> Result<()> {
        instructions::m3m3_cpi::initialize_vault::handle_initialize_m3m3_vault(
            ctx,
            max_amount,
            vault_params,
        )
    }

    pub fn dynamic_amm_swap(
        ctx: Context<DynamicAmmSwap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::dynamic_amm_swap::handle_dynamic_amm_swap(
            ctx,
            amount_in,
            min_amount_out,
        )
    }

    pub fn dynamic_amm_lock_liquidity(
        ctx: Context<DynamicAmmLockLiquidity>,
        allocations: [u16; 2],
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::lock_liquidity::handle_lock_liquidity(ctx, allocations)
    }

    // NOTE: Creator authority PDA lock LP token hold to self + other user
    pub fn dynamic_amm_lock_liquidity_pda_creator(
        ctx: Context<DynamicAmmLockLiquidityPdaCreator>,
        allocations: [u16; 2],
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::lock_liquidity::handle_lock_liquidity_pda_creator(
            ctx,
            allocations,
        )
    }

    pub fn dynamic_amm_claim_fee(ctx: Context<DynamicAmmClaimFee>) -> Result<()> {
        instructions::dynamic_amm_cpi::claim_fee::handle_claim_fee(ctx)
    }

    // NOTE: Creator authority PDA claim fee. LP token must lock to creator authority PDA.
    pub fn dynamic_amm_claim_fee_pda_creator(
        ctx: Context<DynamicAmmClaimFeePdaCreator>,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::claim_fee::handle_claim_fee_pda_creator(ctx)
    }
}
