use super::dlmm_pda::*;
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, Discriminator};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use cpi_example::dlmm::accounts::LbPair;
use cpi_example::dlmm::types::{ProtocolFee, RewardInfo, StaticParameters, VariableParameters};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;

use super::utils::add_packable_account;
use super::RPC;

struct BorshLbPairWrapper(LbPair);

impl AccountDeserialize for BorshLbPairWrapper {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        if buf[..8] != *LbPair::DISCRIMINATOR {
            return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Self::try_deserialize_unchecked(&mut &buf[8..])
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let parameters = StaticParameters {
            base_factor: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            filter_period: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            decay_period: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            reduction_factor: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            variable_fee_control: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            max_volatility_accumulator: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(
                buf,
            )?,
            min_bin_id: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            max_bin_id: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            protocol_share: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            padding: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
        };

        let v_parameters = VariableParameters {
            volatility_accumulator: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(
                buf,
            )?,
            volatility_reference: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            index_reference: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            padding: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            last_update_timestamp: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            padding1: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
        };

        let bump_seed = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let bin_step_seed = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let pair_type = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let active_id = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let bin_step = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let status = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let require_base_factor_seed =
            anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let base_factor_seed = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let activation_type = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let padding0 = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let token_x_mint = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let token_y_mint = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let reserve_x = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let reserve_y = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let protocol_fee = ProtocolFee {
            amount_x: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            amount_y: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
        };
        let padding1 = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let reward_infos: [RewardInfo; 2] = [
            RewardInfo {
                mint: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                vault: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                funder: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                reward_duration: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                reward_duration_end: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(
                    buf,
                )?,
                reward_rate: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                last_update_time: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                cumulative_seconds_with_empty_liquidity_reward:
                    anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            },
            RewardInfo {
                mint: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                vault: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                funder: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                reward_duration: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                reward_duration_end: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(
                    buf,
                )?,
                reward_rate: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                last_update_time: anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
                cumulative_seconds_with_empty_liquidity_reward:
                    anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?,
            },
        ];
        let oracle = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let bin_array_bitmap = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let last_updated_at = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let padding2 = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let pre_activation_swap_address =
            anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let base_key = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let activation_point = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let pre_activation_duration =
            anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let padding3 = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let padding4 = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let creator = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;
        let reserved = anchor_lang::prelude::borsh::BorshDeserialize::deserialize(buf)?;

        Ok(Self(LbPair {
            parameters,
            v_parameters,
            bump_seed,
            bin_step_seed,
            pair_type,
            active_id,
            bin_step,
            status,
            require_base_factor_seed,
            base_factor_seed,
            activation_type,
            padding0,
            token_x_mint,
            token_y_mint,
            reserve_x,
            reserve_y,
            protocol_fee,
            padding1,
            reward_infos,
            oracle,
            bin_array_bitmap,
            last_updated_at,
            padding2,
            pre_activation_swap_address,
            base_key,
            activation_point,
            pre_activation_duration,
            padding3,
            padding4,
            creator,
            reserved,
        }))
    }
}

/// Get bin array index from bin id
pub fn bin_id_to_bin_array_index(bin_id: i32) -> Option<i32> {
    use cpi_example::dlmm::constants::MAX_BIN_PER_ARRAY;
    let idx = bin_id.checked_div(MAX_BIN_PER_ARRAY as i32)?;
    let rem = bin_id.checked_rem(MAX_BIN_PER_ARRAY as i32)?;

    if bin_id.is_negative() && rem != 0 {
        idx.checked_sub(1)
    } else {
        Some(idx)
    }
}

pub struct PoolSetupContext {
    pub pool_state: LbPair,
    pub user_token_x: Pubkey,
    pub user_token_y: Pubkey,
}

pub async fn setup_pool_from_cluster(
    test: &mut ProgramTest,
    pool: Pubkey,
    mock_user: Pubkey,
) -> PoolSetupContext {
    let rpc_client = RpcClient::new(RPC.to_owned());

    let pool_account = rpc_client.get_account(&pool).await.unwrap();
    let pool_state = BorshLbPairWrapper::try_deserialize(&mut pool_account.data.as_ref())
        .unwrap()
        .0;

    test.add_account(pool, pool_account);

    let (oracle_key, _bump) = derive_oracle_pda(pool);
    let oracle_account = rpc_client.get_account(&oracle_key).await.unwrap();
    test.add_account(oracle_key, oracle_account);

    let active_bin_array_idx = bin_id_to_bin_array_index(pool_state.active_id).unwrap();
    let (active_bin_array_key, _bump) = derive_bin_array_pda(pool, active_bin_array_idx.into());

    let bin_array_account = rpc_client.get_account(&active_bin_array_key).await.unwrap();
    test.add_account(active_bin_array_key, bin_array_account);

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];
    let mints = rpc_client.get_multiple_accounts(&mint_keys).await.unwrap();

    for (key, account) in mint_keys.iter().zip(mints) {
        test.add_account(*key, account.unwrap());
    }

    let reserve_keys = vec![pool_state.reserve_x, pool_state.reserve_y];

    let tokens = rpc_client
        .get_multiple_accounts(&reserve_keys)
        .await
        .unwrap();

    for (key, account) in reserve_keys.into_iter().zip(tokens) {
        test.add_account(key, account.unwrap());
    }

    test.add_account(
        mock_user,
        Account {
            lamports: u32::MAX.into(),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            ..Default::default()
        },
    );

    let token_ata_key = mint_keys
        .iter()
        .map(|key| get_associated_token_address(&mock_user, key))
        .collect::<Vec<_>>();

    for (ata_key, mint_key) in token_ata_key.iter().zip(mint_keys) {
        let state = anchor_spl::token::spl_token::state::Account {
            mint: mint_key,
            owner: mock_user,
            amount: u64::MAX / 2,
            state: AccountState::Initialized,
            ..Default::default()
        };

        add_packable_account(test, state, anchor_spl::token::ID, *ata_key);
    }

    PoolSetupContext {
        pool_state,
        user_token_x: token_ata_key[0],
        user_token_y: token_ata_key[1],
    }
}
