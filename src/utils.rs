//! Utilitiles
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;

use crate::error::{ErrorKind, Result, ResultExt};

pub fn hash_password(password: &str) -> Result<String> {
    Ok(hash(password, DEFAULT_COST).context(ErrorKind::HashPasswordFailure)?)
}

pub fn verify_password(raw_password: &str, password: &str) -> Result<bool> {
    Ok(verify(raw_password, password)
        .context(ErrorKind::HashPasswordFailure)?)
}

pub fn random_avatar() -> String {
    let mut rng = rand::thread_rng();
    let avatar_num: i32 = rng.gen_range(1, 21);
    format!("/api/images/avatars/{}.png", avatar_num)
}
