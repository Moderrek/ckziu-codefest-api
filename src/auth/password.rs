use bcrypt::{BcryptResult, DEFAULT_COST, hash, verify};

pub fn password_hash(to_hash: &String) -> BcryptResult<String> {
  hash(to_hash, DEFAULT_COST)
}

pub fn password_verify(password: &String, hash: &str) -> BcryptResult<bool> {
  verify(password, hash)
}