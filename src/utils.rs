//! Utilitiles
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;

use crate::core::error;
use crate::prelude::*;

pub fn hash_password(password: &str) -> Result<String> {
    hash(password, DEFAULT_COST).map_err(error::BadRequest)
}

pub fn verify_password(raw_password: &str, password: &str) -> Result<bool> {
    verify(raw_password, password).map_err(error::BadRequest)
}

pub fn random_avatar() -> String {
    let mut rng = rand::thread_rng();
    let avatar_num: i32 = rng.gen_range(1, 21);
    format!("/api/images/avatars/{}.png", avatar_num)
}
