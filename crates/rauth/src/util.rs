use regex::Regex;

use crate::{Error, Result};

lazy_static! {
    static ref ARGON_CONFIG: argon2::Config<'static> = argon2::Config::default();
}

static ALPHABET: [char; 32] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j',
    'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z',
];

/// Strip special characters and aliases from emails
pub fn normalise_email(original: String) -> String {
    lazy_static! {
        static ref SPLIT: Regex = Regex::new("([^@]+)(@.+)").unwrap();
        static ref SYMBOL_RE: Regex = Regex::new("\\+.+|\\.").unwrap();
    }

    let split = SPLIT.captures(&original).unwrap();
    let mut clean = SYMBOL_RE
        .replace_all(split.get(1).unwrap().as_str(), "")
        .to_string();

    clean.push_str(split.get(2).unwrap().as_str());

    clean
}

/// Hash a password using argon2
pub fn hash_password(plaintext_password: String) -> Result<String> {
    argon2::hash_encoded(
        plaintext_password.as_bytes(),
        nanoid::nanoid!(24).as_bytes(),
        &ARGON_CONFIG,
    )
    .map_err(|_| Error::InternalError)
}
