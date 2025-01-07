use passd::utils::encrypt::encrypt_string;
use pgp;
use std::fs;

#[test]
fn test_encrypt_invalid_key() -> Result<(), pgp::errors::Error> {
    let invalid_key = fs::read_to_string("tests/assets/keys/invalid_pub.asc")?;
    let result = encrypt_string("Hello, world!", &[&invalid_key]);

    assert!(
        result.is_err(),
        "Encryption should fail with an invalid key"
    );

    Ok(())
}

#[test]
fn test_encrypt_sign_key() -> Result<(), pgp::errors::Error> {
    let signing_key = fs::read_to_string("tests/assets/keys/signing_pub.asc")?;
    let result = encrypt_string("Hello, world!", &[&signing_key]);

    assert!(
        result.is_err(),
        "Encryption with a signing key may not be valid"
    );

    Ok(())
}

#[test]
fn test_encrypt_valid() -> Result<(), pgp::errors::Error> {
    let valid_key = fs::read_to_string("tests/assets/keys/valid_pub.asc")?;
    let result = encrypt_string("Hello, world!", &[&valid_key]);

    assert!(
        result.is_ok(),
        "Encryption should succeed with a valid key, but failed with error: {:?}",
        result.err()
    );

    Ok(())
}
