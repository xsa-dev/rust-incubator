use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use argon2::Argon2;
use argon2::password_hash::{Error as PasswordHashError, PasswordHasher, SaltString};
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use sha3::{Digest, Sha3_256};

/// Convenient alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, RandCryptoError>;

/// Errors that may be produced by helper routines in this crate.
#[derive(Debug)]
pub enum RandCryptoError {
    /// Attempted to generate random value from an empty alphabet.
    EmptyAlphabet,
    /// Errors bubbling up from I/O operations.
    Io(io::Error),
    /// Errors produced during Argon2 password hashing.
    PasswordHash(PasswordHashError),
}

impl fmt::Display for RandCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandCryptoError::EmptyAlphabet => write!(f, "alphabet used for generation is empty"),
            RandCryptoError::Io(err) => write!(f, "I/O error: {err}"),
            RandCryptoError::PasswordHash(err) => write!(f, "password hashing error: {err}"),
        }
    }
}

impl std::error::Error for RandCryptoError {}

impl From<io::Error> for RandCryptoError {
    fn from(value: io::Error) -> Self {
        RandCryptoError::Io(value)
    }
}

impl From<PasswordHashError> for RandCryptoError {
    fn from(value: PasswordHashError) -> Self {
        RandCryptoError::PasswordHash(value)
    }
}

/// Generates a random password of the requested length using a provided alphabet.
pub fn generate_password(length: usize, alphabet: &[char]) -> Result<String> {
    if alphabet.is_empty() {
        return Err(RandCryptoError::EmptyAlphabet);
    }

    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    for _ in 0..length {
        // Safety: we just checked that alphabet isn't empty.
        let next = alphabet
            .choose(&mut rng)
            .ok_or(RandCryptoError::EmptyAlphabet)?;
        password.push(*next);
    }

    Ok(password)
}

/// Selects a random value from a slice.
pub fn select_rand_val<T>(values: &[T]) -> Option<&T> {
    let mut rng = rand::thread_rng();
    values.choose(&mut rng)
}

/// Generates a cryptographically secure access token consisting of a-zA-Z0-9 symbols.
pub fn new_access_token() -> String {
    const TOKEN_LEN: usize = 64;
    let mut rng = OsRng;
    (0..TOKEN_LEN)
        .map(|_| rng.sample(Alphanumeric) as char)
        .map(|c| match c {
            '0'..='9' | 'A'..='Z' | 'a'..='z' => c,
            _ => {
                // Alphanumeric already guarantees ASCII digits and letters, but match keeps
                // us honest when the enum extends in the future.
                let allowed: &[u8] =
                    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
                let idx = rng.gen_range(0..allowed.len());
                allowed[idx] as char
            }
        })
        .collect()
}

/// Calculates SHA3-256 hash of the file located at the provided path.
pub fn get_file_hash(path: impl AsRef<Path>) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha3_256::new();
    let mut buf = [0u8; 8 * 1024];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }

    let digest = hasher.finalize();
    Ok(hex::encode(digest))
}

/// Generates an Argon2 password hash using a randomly generated salt.
pub fn hash_password(password: impl AsRef<[u8]>) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_ref(), &salt)?;
    Ok(hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_generation_respects_length() {
        let alphabet: Vec<char> = "abc".chars().collect();
        let result = generate_password(32, &alphabet).expect("alphabet isn't empty");
        assert_eq!(result.len(), 32);
        assert!(result.chars().all(|c| alphabet.contains(&c)));
    }

    #[test]
    fn select_rand_returns_none_for_empty_slice() {
        let empty: [u8; 0] = [];
        assert!(select_rand_val(&empty).is_none());
    }

    #[test]
    fn token_has_expected_length() {
        let token = new_access_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn hashing_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file_path = dir.path().join("sample.txt");
        std::fs::write(&file_path, b"hello world").expect("write file");
        let hash = get_file_hash(&file_path).expect("hash");
        assert_eq!(
            hash,
            "644bcc7e564373040999aac89e7622f3ca71fba1d972fd94a31c3bfbf24e3938"
        );
    }

    #[test]
    fn argon2_hash_is_well_formed() {
        let hash = hash_password("s3cret").expect("hash");
        assert!(hash.starts_with("$argon2id$"));
    }
}
