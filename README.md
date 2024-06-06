# Meteora-CPI-examples

The repository containing examples for CPI (Cross-Program Invocation) to swap at [DLMM](https://github.com/meteoraAg/dlmm-sdk) and [Dynamic AMM](https://github.com/mercurial-finance/mercurial-dynamic-amm-sdk) programs.

## Dependencies

- anchor 0.28.0
- solana 1.16.1
- rust 1.68.0

## Contents

- [CPI to DLMM swap example](programs/cpi-example/src/instructions/dlmm_swap.rs)
- [CPI to Dynamic AMM swap example](programs/cpi-example/src/instructions/dynamic_amm.rs)
- [DLMM CPI test](programs/cpi-example/tests/dlmm_swap.rs)
- [Dynamic AMM CPI test](programs/cpi-example/tests/dynamic_dlmm_swap.rs)

For more details, please check the respective [DLMM](https://github.com/meteoraAg/dlmm-sdk) and [Dynamic AMM](https://github.com/mercurial-finance/mercurial-dynamic-amm-sdk) repo.
