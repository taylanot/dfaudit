//! ============================================================
//! File: dfiles.rs
//! Author: taylanot
//! ============================================================

use std::path::PathBuf;
use walkdir::WalkDir;

use crate::cli::Cli;

pub fn get_files(cli: &Cli) -> Result<Vec<PathBuf>, String> {
  match (&cli.file, &cli.path) {
    (Some(file), None) => {
      if !file.exists() {
        return Err(format!("File does not exist: {}", file.display()));
      }

      Ok(vec![file.clone()])
    }

    (None, Some(path)) => {
      let files = find_files(path);

      if files.is_empty() {
        return Err(format!(
          "No Dockerfile or Containerfile found in '{}'",
          path.display()
        ));
      }

      Ok(files)
    }

    (None, None) => Err("Provide either --file or --path".into()),

    (Some(_), Some(_)) => Err("Cannot use --file and --path together".into()),
  }
}

fn find_files(path: &PathBuf) -> Vec<PathBuf> {
  WalkDir::new(path)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|entry| {
      if !entry.file_type().is_file() {
        return false;
      }

      let name = entry.file_name().to_string_lossy();

      name == "Dockerfile" || name == "Containerfile"
    })
    .map(|entry| entry.path().to_path_buf())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ignores_wrong_names() {
    let fixture_path = PathBuf::from("tests/fixtures/wrong_names");
    let files = find_files(&fixture_path);
    assert!(files.is_empty(), "similarly-named files should not match");
  }

  #[test]
  fn no_matches() {
    let fixture_path = PathBuf::from("tests/fixtures/c");
    let files = find_files(&fixture_path);
    assert!(files.is_empty());
  }

  #[test]
  fn both_types() {
    let fixture_path = PathBuf::from("tests/fixtures");
    let files = find_files(&fixture_path);
    let mut names: Vec<String> = files
      .iter()
      .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
      .collect();
    names.sort();
    assert_eq!(names, vec!["Containerfile", "Dockerfile"]);
  }

  #[test]
  fn dummyfile() {
    let fixture_path = PathBuf::from("tests/fixtures");
    let files = find_files(&fixture_path);
    let names: Vec<String> = files
      .iter()
      .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
      .collect();
    assert!(
      !names.contains(&"Dummyfile".to_string()),
      "Dummyfile should never be matched"
    );
    assert_eq!(files.len(), 2, "expected only Dockerfile and Containerfile");
  }
}
