# Meteora-CPI-examples

The repository containing examples for CPI (Cross-Program Invocation) to [DLMM](https://github.com/meteoraAg/dlmm-sdk) and [Dynamic AMM](https://github.com/MeteoraAG/dynamic-amm-sdk) programs.

Disclaimer: This repository only serves as examples and is not intended for production use.

## Dependencies

- anchor 0.31.0
- solana 2.1.0
- rust 1.85.0

## Contents

- [CPI to DLMM swap example](programs/cpi-example/src/instructions/dlmm_cpi/swap.rs)
- [CPI to Dynamic AMM swap example](programs/cpi-example/src/instructions/dynamic_amm_cpi/swap.rs)

- [CPI to Dynamic AMM initialize pool example](programs/cpi-example/src/instructions/dynamic_amm_cpi/initialize_customizable_permissionless_pool.rs)
  - [PDA-as-creator variant](programs/cpi-example/src/instructions/dynamic_amm_cpi/initialize_customizable_permissionless_pool.rs) (a program PDA acts as pool creator and holds the LP)
- [CPI to Dynamic AMM initialize pool with config example](programs/cpi-example/src/instructions/dynamic_amm_cpi/initialize_permissionless_pool_with_config.rs)
  - [PDA-as-creator variant](programs/cpi-example/src/instructions/dynamic_amm_cpi/initialize_permissionless_pool_with_config.rs)

- [CPI to Dynamic AMM lock liquidity example](programs/cpi-example/src/instructions/dynamic_amm_cpi/lock_liquidity.rs)
  - [PDA-as-creator variant](programs/cpi-example/src/instructions/dynamic_amm_cpi/lock_liquidity.rs) (PDA locks LP to itself and another user)
- [CPI to Dynamic AMM claim fee example](programs/cpi-example/src/instructions/dynamic_amm_cpi/claim_fee.rs)
  - [PDA-as-creator variant](programs/cpi-example/src/instructions/dynamic_amm_cpi/claim_fee.rs) (PDA claims the locked-LP fees)

- [CPI to M3m3 initialize vault example](programs/cpi-example/src/instructions/m3m3_cpi/initialize_vault.rs)

- [Tests](programs/cpi-example/tests/)

For more details, please check the respective [DLMM](https://github.com/meteoraAg/dlmm-sdk) and [Dynamic AMM](https://github.com/MeteoraAG/dynamic-amm-sdk) repo.
