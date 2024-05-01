use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
pub const USER_STUFF: i32 = 1 << 0;
#[allow(dead_code)]
pub const USER_DEVELOPER: i32 = 1 << 1;
#[allow(dead_code)]
pub const USER_TEACHER: i32 = 1 << 2;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
  pub name: String,
  pub display_name: String,
  pub id: Uuid,

  pub bio: Option<String>,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,

  pub flags: i32,
}

#[allow(dead_code)]
impl User {
  pub fn is_flag_set(&self, index: u32) -> bool {
    1 == (self.flags & 1 << index)
  }

  pub fn set_flag(&mut self, index: u32, turn_on: bool) {
    let flag = 1 << index;
    if turn_on {
      // switch on a flag
      self.flags |= flag;
    } else {
      // switch off a flag
      self.flags &= !flag;
    }
  }

  pub fn is_staff(&self) -> bool {
    self.is_flag_set(0)
  }

  pub fn set_staff(&mut self, turn_on: bool) {
    self.set_flag(0, turn_on)
  }

  pub fn is_developer(&self) -> bool {
    self.is_flag_set(1)
  }

  pub fn set_developer(&mut self, turn_on: bool) {
    self.set_flag(1, turn_on)
  }

  pub fn is_teacher(&self) -> bool {
    self.is_flag_set(2)
  }

  pub fn set_teacher(&mut self, turn_on: bool) {
    self.set_flag(2, turn_on)
  }
}