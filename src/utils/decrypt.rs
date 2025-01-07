use pgp::{
    composed::{message::Message, signed_key::SignedSecretKey},
    types::KeyId,
    Deserializable,
};
use std::str;

pub fn decrypt_string(
    input: &str,
    key_password: &str,
    private_keys: &[&str],
) -> Result<(Message, Vec<KeyId>), pgp::errors::Error> {
    let (message, _) = Message::from_string(input)?;
    let private_keys: Result<Vec<SignedSecretKey>, _> = private_keys
        .iter()
        .map(|key| SignedSecretKey::from_string(key).map(|(key, _)| key))
        .collect();
    let decrypted_data = message.decrypt(
        || key_password.to_string(),
        &private_keys?.iter().collect::<Vec<_>>(),
    )?;

    Ok(decrypted_data)
}
