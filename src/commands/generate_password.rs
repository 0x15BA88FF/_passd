use std::io;
use rand::Rng;

#[derive(PartialEq)]
pub enum Filter {
    All,
    Alphabets,
    Capital,
    Numbers,
    Symbols,
    Custom
}

pub fn generate_password(
    length: Option<usize>,
    filter: Option<Vec<Filter>>,
    custom: Option<Vec<String>>,
    separators: Option<Vec<&str>>,
) -> Result<String, io::Error> {
    let length = length.unwrap_or(25);
    let filters = filter.unwrap_or_else(|| vec![Filter::All]);
    let custom = custom.unwrap_or_default();
    let separators = separators.unwrap_or_default();

    let numbers: Vec<String> = "0123456789".chars().map(|c| c.to_string()).collect();
    let lowercase: Vec<String> = "abcdefghijklmnopqrstuvwxyz".chars().map(|c| c.to_string()).collect();
    let uppercase: Vec<String> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().map(|c| c.to_string()).collect();
    let symbols: Vec<String> = "!@#$%^&*()-_=+[]{}|;:',.<>?/`~".chars().map(|c| c.to_string()).collect();

    let mut pool: Vec<String> = Vec::new();

    if filters.contains(&Filter::All) {
        pool.extend(lowercase);
        pool.extend(uppercase);
        pool.extend(numbers);
        pool.extend(symbols);
        pool.extend(custom);
    } else {
        if filters.contains(&Filter::Alphabets) { pool.extend(lowercase); }
        if filters.contains(&Filter::Capital) { pool.extend(uppercase); }
        if filters.contains(&Filter::Numbers) { pool.extend(numbers); }
        if filters.contains(&Filter::Symbols) { pool.extend(symbols); }
        if filters.contains(&Filter::Custom) { pool.extend(custom); }
    }

    if pool.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No characters available in the pool to generate a password."
        ));
    }

    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let random_index = rng.gen_range(0..pool.len());
            pool[random_index].clone()
        })
        .collect();

    if !separators.is_empty() {
        let sep: Vec<String> = separators
            .iter()
            .flat_map(|s| s.chars().map(|c| c.to_string()))
            .collect();

        let mut password_with_separators = String::new();
        for (i, c) in password.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                password_with_separators.push_str(&sep[rng.gen_range(0..sep.len())]);
            }
            password_with_separators.push(c);
        }
        return Ok(password_with_separators);
    }

    Ok(password)
}
