use passd::commands::decrypt_string;
use pgp;
use std::fs;

#[test]
fn test_decrypt_invalid_key() -> Result<(), pgp::errors::Error> {
    let encrypted_message = fs::read_to_string("tests/assets/keys/encrypted_message.asc")?;
    let invalid_key = fs::read_to_string("tests/assets/keys/invalid_priv.asc")?;
    let result = decrypt_string(&encrypted_message, "password", &[&invalid_key]);

    assert!(
        result.is_err(),
        "Decryption should fail with an invalid private key"
    );

    Ok(())
}

#[test]
fn test_decrypt_invalid_password() -> Result<(), pgp::errors::Error> {
    let encrypted_message = fs::read_to_string("tests/assets/keys/encrypted_message.asc")?;
    let private_key = fs::read_to_string("tests/assets/keys/valid_priv.asc")?;
    let result = decrypt_string(&encrypted_message, "wrong_password", &[&private_key]);

    assert!(
        result.is_err(),
        "Decryption should fail with an incorrect password"
    );

    Ok(())
}

#[test]
fn test_decrypt_valid_key() -> Result<(), pgp::errors::Error> {
    let encrypted_message = fs::read_to_string("tests/assets/keys/encrypted_message.asc")?;
    let private_key = fs::read_to_string("tests/assets/keys/valid_priv.asc")?;
    let result = decrypt_string(&encrypted_message, "password", &[&private_key]);

    assert!(
        result.is_ok(),
        "Decryption should succeed with the correct key and password, but failed with error: {:?}",
        result.err()
    );

    let (message, _) = result?;
    if let Some(bytes) = message.get_content()? {
        if let Ok(decrypted_content) = String::from_utf8(bytes) {
            assert_eq!(
                decrypted_content, "Hello, world!",
                "Decrypted content does not match the expected content"
            );
        }
    }

    Ok(())
}
