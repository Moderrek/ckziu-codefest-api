use chrono::{DateTime, Utc};
use rand::prelude::SliceRandom;

const OTP_DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Debug, Clone)]
pub struct OTP {
    pub code: String,
    pub expires_on: DateTime<Utc>,
}

impl OTP {
  pub fn new_expirable_code(length: usize, duration: chrono::Duration) -> OTP {
    let code = generate_otp(length);
    let expires_on = Utc::now()
      .checked_add_signed(duration)
      .expect("Date out of range");
    OTP { code, expires_on }
  }

  pub fn is_expired(&self) -> bool {
    Utc::now().timestamp() > self.expires_on.timestamp()
  }

  pub fn check(&self, other_code: &String) -> bool {
    self.code == *other_code
  }
}

// Generates random code with given length.
pub fn generate_otp(length: usize) -> String {
  // Reserving memory space
  let mut buffer = String::with_capacity(length);
  // Create randomiser
  let mut randomiser = rand::thread_rng();

  // Randomise digits and push them to buffer
  for _ in 0..length {
      let digit = *OTP_DIGITS.choose(&mut randomiser).unwrap();
    buffer.push(digit);
  }

  buffer
}
