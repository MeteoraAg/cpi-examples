use anchor_lang::prelude::*;

pub mod instructions;
use instructions::dlmm_swap::*;
use instructions::dynamic_amm_swap::*;

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod cpi_example {
    use super::*;

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dlmm_swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }

    pub fn dynamic_amm_swap(
        ctx: Context<DynamicAmmSwap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dynamic_amm_swap::handle_dynamic_amm_swap(ctx, amount_in, min_amount_out)
    }
}
