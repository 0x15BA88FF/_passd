use pgp::{
    composed::{
        message::Message,
        signed_key::SignedPublicKey
    },
    crypto::{
        aead::AeadAlgorithm,
        sym::SymmetricKeyAlgorithm
    },
    Deserializable
};
use rand::rngs::OsRng;

pub fn encrypt_string(
    input: &str,
    public_keys: &[&str]
) -> Result<String, pgp::errors::Error> {
    let message = Message::new_literal("content", input);
    let public_keys: Result<Vec<SignedPublicKey>, _> = public_keys
        .iter()
        .map(|key| SignedPublicKey::from_string(key).map(|(key, _)| key))
        .collect();

    let encrypted_message = message.encrypt_to_keys_seipdv2(
        OsRng,
        SymmetricKeyAlgorithm::AES256,
        AeadAlgorithm::Eax,
        0,
        &public_keys?.iter().collect::<Vec<_>>()
    )?;

    let encrypted_string = encrypted_message.to_armored_string(None.into())?;

    Ok(encrypted_string)
}
