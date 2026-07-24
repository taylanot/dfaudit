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

#[doc(hidden)]
pub fn find_files(path: &PathBuf) -> Vec<PathBuf> {
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
