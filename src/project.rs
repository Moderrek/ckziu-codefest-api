use std::collections::HashMap;

pub mod models;
pub mod responses;
pub mod api;
pub mod db;

pub fn validate_name(name: String) -> Result<String, String> {
  let name = name.to_lowercase();
  if name.len() < 3 {
    return Err("Nazwa musi mieć conajmniej 3 znaki.".into());
  }
  if name.len() > 40 {
    return Err("Nazwa może mieć maksymalnie 40 znaków.".into());
  }
  let legalize_chars = HashMap::from([
    (' ', '-'),
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
  for val in name.chars() {
    let val = match legalize_chars.contains_key(&val) {
      true => *legalize_chars.get(&val).unwrap(),
      _ => val,
    };
    builder.push(val);
  }


  let result: String = builder.iter().collect();

  Ok(result)
}

pub fn validate_display_name(display_name: String) -> Result<String, String> {
  if display_name.len() < 3 {
    return Err("Wyświetlana nazwa musi posiadać conajmniej 3 znaki.".into());
  }
  if display_name.len() > 40 {
    return Err("Wyświetlana nazwa nie może przekraczać długości 40 znaków.".into());
  }
  let result: String = display_name.trim_start().trim_end().into();
  Ok(result)
}

pub fn validate_description(description: Option<String>) -> Result<Option<String>, String> {
  if description.is_none() {
    return Ok(None);
  }
  let description = description.unwrap();
  if description.len() > 100 {
    return Err("Opis nie może przekraczać 100 znaków.".into());
  }
  let result: String = description.trim_start().trim_end().into();
  Ok(Some(result))
}
