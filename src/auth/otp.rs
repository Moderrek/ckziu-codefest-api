use chrono::{DateTime, Utc};
use rand::prelude::SliceRandom;

#[derive(Debug)]
pub struct OTPData {
  pub code: String,
  pub expired: DateTime<Utc>,
}

pub fn generate_otp(length: usize) -> String {
  let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
  let mut buffer = String::with_capacity(length);

  for _ in 0..length {
    buffer.push(*digits.choose(&mut rand::thread_rng()).unwrap());
  }

  buffer
}

