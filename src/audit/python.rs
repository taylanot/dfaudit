//! ============================================================
//! File: python.rs
//! Author: taylanot
//! ============================================================

use crate::container::traits::ContainerEngine;

use super::models::Package;

pub fn audit<C: ContainerEngine + ?Sized>(
  engine: &C,
  image: &str,
) -> Result<Option<Vec<Package>>, String> {
  let output =
    engine.run(image, &["python3", "-m", "pip", "list", "--format=json"]);

  let output = match output {
    Ok(data) => data,

    Err(_) => return Ok(None),
  };

  let packages = serde_json::from_slice::<Vec<Package>>(&output)
    .map_err(|e| format!("failed parsing python packages: {}", e))?;

  Ok(Some(packages))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::audit::test_utils::MockEngine;

  const CMD: &[&str] = &["python3", "-m", "pip", "list", "--format=json"];

  #[test]
  fn parse_packages() {
    let engine = MockEngine::new().with_response(
      CMD,
      Ok(br#"[{"name":"requests","version":"2.32.0"}]"#.to_vec()),
    );

    let packages = audit(&engine, "image").unwrap().unwrap();

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "requests");
    assert_eq!(packages[0].version, "2.32.0");
  }

  #[test]
  fn python_missing() {
    let engine = MockEngine::new().with_response(CMD, Err("not found".into()));

    let result = audit(&engine, "image").unwrap();

    assert_eq!(result, None);
  }

  #[test]
  fn invalid_json() {
    let engine =
      MockEngine::new().with_response(CMD, Ok(b"not json".to_vec()));

    assert!(audit(&engine, "image").is_err());
  }
}
