use std::{collections::HashMap, net::SocketAddr};

use chrono::Utc;

pub const CKZIU_MAIL_DOMAIN: &str = "ckziu.elodz.edu.pl";

pub fn current_millis() -> i64 {
  Utc::now().timestamp_millis()
}

pub fn addr_to_string(addr: &Option<SocketAddr>) -> String {
  match addr {
    Some(addr) => addr.to_string(),
    None => "Unknown".into(),
  }
}

pub fn validate_password(password: String) -> Result<String, String> {
  if password.chars().count() < 8 {
    // The Password must be longer than 8 characters.
    return Err("Hasło musi mieć conajmniej 8 znaków".into());
  }
  for c in password.chars() {
    if c.is_whitespace() {
      // Password cannot contain whitespaces.
      return Err("Hasło nie może mieć spacji i znaków białych".into());
    }
  }

  Ok(password)
}

pub fn validate_mail(mail: String) -> Result<String, String> {
  let mail = mail
    .to_lowercase()
    .trim_start()
    .trim_end()
    .into();

  // Development case
  if mail == "tymonek12345@gmail.com" || mail == "filip.sobczuk@o2.pl" {
    return Ok(mail);
  }

  if mail.len() <= CKZIU_MAIL_DOMAIN.len() {
    // Mail is too short
    return Err("Mail jest za krótki".into());
  }
  if !mail.ends_with(CKZIU_MAIL_DOMAIN) {
    // Mail domain is unallowed
    return Err("Mail nie należy do szkoły CKZiU w Łodzi".into());
  }

  Ok(mail)
}

pub fn validate_name(name: String) -> Result<String, String> {
  // Name must always be lower cased
  let name = name
    .trim_start()
    .trim_end()
    .to_lowercase();

  // Check length 3..=40
  let len = name.chars().count();
  if len < 3 {
    // The name must be at 3 characters long.
    return Err("Nazwa musi mieć conajmniej 3 znaki.".into());
  }
  if len > 40 {
    // The name cannot be longer than 40 characters.
    return Err("Nazwa może mieć maksymalnie 40 znaków.".into());
  }

  // Check starts/ends with '-'
  if name.starts_with('-') {
    // The name cannot start with '-'
    return Err("Nazwa nie może zaczynać się znakiem '-'".into());
  }
  if name.ends_with('-') {
    // The name cannot end with '-'
    return Err("Nazwa nie może kończyć się znakiem '-'".into());
  }

  // Diacritic -> ASCII
  let diacritic = HashMap::from([
    // exception
    (' ', '-'),
    // polish diacritic
    ('ą', 'a'),
    ('ć', 'c'),
    ('ę', 'e'),
    ('ó', 'o'),
    ('ś', 's'),
    ('ł', 'l'),
    ('ź', 'z'),
    ('ń', 'n'),
  ]);

  let mut builder = Vec::new();
  // Checks all chars and translate diacritic chars to ascii
  for val in name.chars() {
    let val = match diacritic.contains_key(&val) {
      true => *diacritic.get(&val).unwrap(),
      _ => val,
    };
    if !(val.is_ascii() || val.is_ascii_digit() || val == '-') {
      // The name contains illegal character '?'
      return Err(format!("Nazwa zawiera nielegalny znak '{val}'"));
    }
    // Legal character
    builder.push(val);
  }
  let name: String = builder.iter().collect();

  Ok(name)
}

pub fn validate_display_name(display_name: String) -> Result<String, String> {
  let display_name: String = display_name
    .trim_start()
    .trim_end()
    .into();
  let len = display_name.chars().count();

  if len < 3 {
    // The display name must be at 3 characters long.
    return Err("Wyświetlana nazwa musi posiadać conajmniej 3 znaki.".into());
  }
  if len > 40 {
    // The display name cannot be longer than 40 characters.
    return Err("Wyświetlana nazwa nie może przekraczać długości 40 znaków.".into());
  }

  Ok(display_name)
}

pub fn validate_description(description: Option<String>) -> Result<Option<String>, String> {
  // If description is none we don't need to validate.
  if description.is_none() {
    return Ok(None);
  }
  let description: String = description
    .unwrap()
    .trim_start()
    .trim_end()
    .into();

  if description.chars().count() > 100 {
    // The description cannot be longer than 100 characters.
    return Err("Opis nie może przekraczać 100 znaków.".into());
  }

  Ok(Some(description))
}
