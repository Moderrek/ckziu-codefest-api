use bcrypt::{BcryptResult, DEFAULT_COST, hash, verify};

// Hashes password
pub fn password_hash(to_hash: &String) -> BcryptResult<String> {
  hash(to_hash, DEFAULT_COST)
}

// Verify raw password string to hashed password
pub fn password_verify(password: &String, hash: &str) -> BcryptResult<bool> {
  verify(password, hash)
}
