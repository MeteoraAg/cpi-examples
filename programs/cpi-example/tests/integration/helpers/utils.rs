use anchor_lang::prelude::Result;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::{solana_program::instruction::Instruction, AccountDeserialize};
use anchor_lang::{Owner, ZeroCopy};
use assert_matches::assert_matches;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub async fn process_and_assert_ok(
    instructions: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
    banks_client: &mut BanksClient,
) {
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    let mut all_signers = vec![payer];
    all_signers.extend_from_slice(signers);

    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &all_signers,
        recent_blockhash,
    );

    assert_matches!(banks_client.process_transaction(tx).await, Ok(()));
}

pub fn add_packable_account<T: Pack>(
    test: &mut ProgramTest,
    account: T,
    owner: Pubkey,
    pk: Pubkey,
) {
    let mut data = vec![0u8; T::LEN];
    account.pack_into_slice(&mut data);

    test.add_account(
        pk,
        Account {
            lamports: u32::MAX.into(),
            data,
            owner,
            ..Default::default()
        },
    );
}

pub fn deserialize_zc_unalignment<T: AccountDeserialize + ZeroCopy + Owner>(
    account: &Account,
) -> Result<T> {
    if account.owner != T::owner() {
        return Err(anchor_lang::prelude::Error::from(
            anchor_lang::prelude::ErrorCode::AccountOwnedByWrongProgram,
        )
        .with_pubkeys((account.owner, T::owner())));
    }

    let data = &account.data[..];
    let disc = T::DISCRIMINATOR;
    if data.len() < disc.len() {
        return Err(anchor_lang::prelude::ErrorCode::AccountDiscriminatorNotFound.into());
    }

    let given_disc = &data[..disc.len()];
    if given_disc != disc {
        return Err(anchor_lang::prelude::ErrorCode::AccountDiscriminatorMismatch.into());
    }

    Ok(bytemuck::pod_read_unaligned(&data[disc.len()..]))
}
