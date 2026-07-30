//! ============================================================
//! File: descrip.rs
//! Author: taylanot
//! ============================================================

use regex::Regex;
use std::path::PathBuf;

pub fn get_description(file: &PathBuf) -> Result<Option<String>, String> {
  let text = std::fs::read_to_string(file).map_err(|e| e.to_string())?;

  let re =
    Regex::new(r#"LABEL\s+(\w+)="([^"]*)""#).map_err(|e| e.to_string())?;

  for cap in re.captures_iter(&text) {
    let label = &cap[1];
    let value = &cap[2];

    if label == "description" {
      return Ok(Some(value.to_string()));
    }
  }

  Ok(None)
}

#[cfg(test)]
mod tests {

  use super::*;
  use std::fs;

  #[test]
  fn description_found() {
    let path = PathBuf::from("with");

    fs::write(
      &path,
      r#"
          LABEL name="my-app"
          LABEL description="A web server image"
          LABEL version="1.0"
          "#,
    )
    .unwrap();

    let result = get_description(&path).unwrap();

    assert_eq!(result, Some("A web server image".to_string()));

    fs::remove_file(path).unwrap();
  }

  #[test]
  fn description_not_found() {
    let path = PathBuf::from("without");

    fs::write(
      &path,
      r#"
          LABEL name="my-app"
          LABEL version="1.0"
          "#,
    )
    .unwrap();

    let result = get_description(&path).unwrap();

    assert_eq!(result, None);

    fs::remove_file(path).unwrap();
  }
}
