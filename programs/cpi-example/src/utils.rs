use anchor_lang::{prelude::*, ZeroCopy};

// NOTE: One shouldn't do this in production. AccountLoader deserialization will works on mainnet. It's failing in bpf test due to rust version u128 layout changes.
pub fn deserialize_zc_account_workaround<'info, T: AccountDeserialize + ZeroCopy + Owner>(
    acc_info: &UncheckedAccount<'info>,
) -> Result<T> {
    if acc_info.owner != &T::owner() {
        return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
            .with_pubkeys((*acc_info.owner, T::owner())));
    }

    let data = &acc_info.try_borrow_data()?;
    let disc = T::DISCRIMINATOR;
    if data.len() < disc.len() {
        return Err(ErrorCode::AccountDiscriminatorNotFound.into());
    }

    let given_disc = &data[..disc.len()];
    if given_disc != disc {
        return Err(ErrorCode::AccountDiscriminatorMismatch.into());
    }

    Ok(bytemuck::pod_read_unaligned(&data[disc.len()..]))
}
