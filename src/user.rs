use std::fmt;
use std::fmt::{Display, Formatter};

use uuid::Uuid;

pub mod models;
pub mod db;
pub mod api;
pub mod responses;

#[allow(dead_code)]
pub enum UserSelector {
  Uuid(Uuid),
  Username(String),
  Mail(String),
}

impl UserSelector {
  fn as_string(&self) -> String {
    if let UserSelector::Uuid(uuid) = self {
      return uuid.to_string();
    }
    if let UserSelector::Username(username) = self {
      return username.clone();
    }
    if let UserSelector::Mail(mail) = self {
      return mail.clone();
    }
    unreachable!("The statements should always return String type");
  }
}

impl Display for UserSelector {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_string())
  }
}
