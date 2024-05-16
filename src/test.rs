use super::*;

#[test]
fn name() {
  let name = "  nOrmal-NamE333  \t";
  let validated = utils::validate_name(name.into()).unwrap();
  assert_eq!("normal-name333", validated);
}

#[test]
fn diacritic() {
  let name = "polishńaęłąś";
  let validated = utils::validate_name(name.into()).unwrap();
  assert_eq!("polishnaelas", validated);
}

#[test]
#[should_panic]
fn name_illegal_char() {
  let name = "moł🪸da";
  utils::validate_name(name.into()).unwrap();
}

#[test]
#[should_panic]
fn name_illegal_start() {
  let name = "-name";
  utils::validate_name(name.into()).unwrap();
}

#[test]
#[should_panic]
fn name_illegal_end() {
  let name = "-name-";
  utils::validate_name(name.into()).unwrap();
}
