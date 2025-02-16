use passd::commands::{generate_password, generate_password::Filter};

#[test]
fn test_generate_password_empty_filters() {
    let result = generate_password(None, Some(vec![]), None, None);

    assert!(
        result.is_err(),
        "Expected error no characters in pool to generate a password."
    );
}

#[test]
fn test_generate_password_length() {
    let password_length = 50;

    let result = generate_password(
        Some(password_length),
        Some(vec![Filter::Alphabets]),
        None,
        None,
    )
    .unwrap();

    assert_eq!(
        password_length,
        result.len(),
        "Password length should be {:?}.",
        password_length
    );
}

#[test]
fn test_generate_password_separators() {
    let password = generate_password(
        None,
        Some(vec![Filter::Alphabets]),
        None,
        Some(vec!["-".to_string(), "_".to_string()]),
    )
    .unwrap();

    assert!(
        password.contains('-') || password.contains('_'),
        "Password should contain at least one of the specified separators"
    );
}

#[test]
fn test_generate_password_custom_pool() {
    let custom_pool = vec!["A", "B", "C"];

    let password = generate_password(
        None,
        Some(vec![Filter::Custom]),
        Some(custom_pool.iter().map(|c| c.to_string()).collect()),
        None,
    )
    .unwrap();

    assert!(
        password
            .chars()
            .all(|c| custom_pool.contains(&c.to_string().as_str())),
        "Password should only contain characters from the custom pool"
    );
}

#[test]
fn test_generate_password_alphabets_filter() {
    let password = generate_password(None, Some(vec![Filter::Alphabets]), None, None).unwrap();

    assert!(
        password.chars().all(|c| c.is_lowercase()),
        "Password should contain only lowercase letters"
    );
}

#[test]
fn test_generate_password_capital_filter() {
    let password = generate_password(None, Some(vec![Filter::Capital]), None, None).unwrap();

    assert!(
        password.chars().all(|c| c.is_uppercase()),
        "Password should contain only uppercase letters"
    );
}

#[test]
fn test_generate_password_numbers_filter() {
    let password = generate_password(None, Some(vec![Filter::Numbers]), None, None).unwrap();

    assert!(
        password.chars().all(|c| c.is_numeric()),
        "Password should contain only numbers"
    );
}

#[test]
fn test_generate_password_symbols_filter() {
    let symbols: Vec<char> = "!@#$%^&*()-_=+[]{}|;:',.<>?/`~".chars().collect();

    let password = generate_password(None, Some(vec![Filter::Symbols]), None, None).unwrap();

    assert!(
        password.chars().all(|c| symbols.contains(&c)),
        "Password should contain only symbols"
    );
}
