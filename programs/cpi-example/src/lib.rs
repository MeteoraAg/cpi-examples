use anchor_lang::prelude::*;

use dynamic_amm::instructions::CustomizableParams as DynamicAmmCustomizableParams;
use m3m3::InitializeVaultParams;

pub mod instructions;
pub use instructions::*;

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod cpi_example {
    use super::*;

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dlmm_cpi::swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }

    pub fn initialize_dynamic_amm_customizable_permissionless_pool(
        ctx: Context<DynamicAmmInitializeCustomizablePermissionlessPool>,
        token_a_amount: u64,
        token_b_amount: u64,
        params: DynamicAmmCustomizableParams,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_customizable_permissionless_pool::handle_initialize_customizable_permissionless_pool(
            ctx,
            token_a_amount,
            token_b_amount,
            params,
        )
    }

    pub fn initialize_dynamic_amm_customizable_permissionless_pool_pda_creator(
        ctx: Context<DynamicAmmInitializeCustomizablePermissionlessPoolPdaCreator>,
        token_a_amount: u64,
        token_b_amount: u64,
        params: DynamicAmmCustomizableParams,
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
        instructions::dynamic_amm_cpi::initialize_permissionless_pool_with_config::handle_initialize_customizable_permissionless_pool_with_config(
            ctx,
            token_a_amount,
            token_b_amount,
            activation_point,
        )
    }

    pub fn initialize_dynamic_amm_permission_pool_with_config_pda_creator(
        ctx: Context<DynamicAmmInitializePermissionlessPoolWithConfigPoolPdaCreator>,
        token_a_amount: u64,
        token_b_amount: u64,
        activation_point: Option<u64>,
    ) -> Result<()> {
        instructions::dynamic_amm_cpi::initialize_permissionless_pool_with_config::handle_initialize_customizable_permissionless_pool_with_pda_creator(
            ctx,
            token_a_amount,
            token_b_amount,
            activation_point,
        )
    }

    pub fn initialize_m3m3_vault(
        ctx: Context<InitializeM3m3Vault>,
        max_amount: u64,
        vault_params: InitializeVaultParams,
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
        instructions::dynamic_amm_cpi::swap::handle_dynamic_amm_swap(ctx, amount_in, min_amount_out)
    }
}
