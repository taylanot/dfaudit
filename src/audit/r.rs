//! ============================================================
//! File: r.rs
//! Author: taylanot
//! ============================================================

use crate::container::traits::ContainerEngine;

use super::models::Package;

pub fn audit<C: ContainerEngine>(
  engine: &C,
  image: &str,
) -> Result<Option<Vec<Package>>, String> {
  let output = engine.run(
    image,
    &["Rscript", "-e", "installed.packages()[,c('Package','Version')]"],
  );

  let output = match output {
    Ok(data) => data,

    Err(_) => return Ok(None),
  };

  let stdout = String::from_utf8_lossy(&output);

  let packages = stdout
    .lines()
    .skip(1)
    .filter_map(|line| {
      let parts = line.split_whitespace().collect::<Vec<_>>();

      if parts.len() >= 2 {
        Some(Package {
          name: parts[0].to_string(),

          version: parts[2].trim_matches('"').to_string(),
        })
      } else {
        None
      }
    })
    .collect();

  Ok(Some(packages))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::audit::test_utils::MockEngine;

  const CMD: &[&str] =
    &["Rscript", "-e", "installed.packages()[,c('Package','Version')]"];

  #[test]
  fn parses_packages() {
    let engine = MockEngine::new().with_response(
      CMD,
      Ok(
        b"Package Version\n\
                  dplyr 1.1.4\n\
                  ggplot2 3.5.1\n"
          .to_vec(),
      ),
    );

    let packages = audit(&engine, "image").unwrap().unwrap();

    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].name, "dplyr");
    assert_eq!(packages[1].name, "ggplot2");
  }

  #[test]
  fn returns_none_when_r_is_missing() {
    let engine = MockEngine::new().with_response(CMD, Err("not found".into()));

    let result = audit(&engine, "image").unwrap();

    assert_eq!(result, None);
  }

  #[test]
  fn skips_invalid_lines() {
    let engine = MockEngine::new().with_response(
      CMD,
      Ok(
        b"Package Version\n\
                  dplyr 1.1.4\n\
                  invalid\n\
                  ggplot2 3.5.1\n"
          .to_vec(),
      ),
    );

    let packages = audit(&engine, "image").unwrap().unwrap();

    assert_eq!(packages.len(), 2);
  }
}
