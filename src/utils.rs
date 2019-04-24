//! Utilitiles
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use rand::Rng;

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn verify_password(
    raw_password: &str,
    password: &str,
) -> Result<bool, BcryptError> {
    Ok(verify(raw_password, password)?)
}

pub fn random_avatar() -> String {
    let mut rng = rand::thread_rng();
    let avatar_num: i32 = rng.gen_range(1, 21);
    format!("/api/images/avatars/{}.png", avatar_num)
}
