use clap::Parser;
use std::path::PathBuf;
use std::path::Path;
use walkdir::WalkDir;
use std::process::Command;

use std::process::Stdio;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Get the data
#[derive(Parser)]
struct Cli {
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    #[arg(short = 'p', long = "path")]
    path: Option<PathBuf>,
}

/// Build a container image with podman from the path
fn build_image(file: &Path) -> Result<(), String> {

  let spinner = ProgressBar::new_spinner();
  spinner.set_style(
    ProgressStyle::with_template("{spinner} {msg}")
      .unwrap()
  );
  // spinner.set_message(format!("Building {}", file.display()));
  spinner.enable_steady_tick(Duration::from_millis(100));

  let build_status = Command::new("podman")
    .args(["build", "-t", "temp-image"])
    .arg("-f")
    .arg(file)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map_err(|e| format!("failed to run podman: {e}"))?;

  spinner.finish_and_clear();

  if !build_status.success() {
    return Err(format!("podman build failed with status: {build_status}"));
  }

  println!("Build succeeded for '{}'", file.display());

  println!("Now deleting the temporary image! Image tagged as temp-image");

  let remove_status = Command::new("podman")
    .args(["rmi", "temp-image"])
    .status()
    .map_err(|e| format!("failed to run podman rmi: {e}"))?;

  if !remove_status.success() {
    return Err(format!("podman rmi failed with status: {remove_status}"));
  }

  Ok(())
}


/// Find all the Docker/Container files
fn find_files(cli: &Cli) -> Vec<PathBuf> {

  let path = cli.path.as_ref().expect("path is required");

  if !path.exists() {
      eprintln!("Path '{}' does not exist", path.display());
  }

  println!("Searching in '{}'", path.display());

  let mut files = Vec::new();

  for entry in WalkDir::new(&path) 
    .into_iter()
    .filter_map(Result::ok)
  {
    if entry.file_type().is_file() {
      let name = entry.file_name().to_string_lossy();

      if name == "Dockerfile" || name == "Containerfile" {
        files.push(entry.path().to_path_buf());
      }
    }
  }

  files
}

/// Get a vector of files to build
fn get_files(cli: &Cli) -> Result<Vec<PathBuf>, String> {
  match (&cli.file, &cli.path) {
    // A single file was provided
    (Some(file), None) => {
      if !file.exists() {
        return Err(format!("File does not exist: {}", file.display()));
      }

      Ok(vec![file.clone()])
    }

    // A directory was provided, search recursively
    (None, Some(path)) => {
      let files = find_files(cli);

      if files.is_empty() {
        return Err(format!(
          "No Dockerfile or Containerfile found in '{}'",
          path.display()
        ));
      }

      Ok(files)
    }

    // Nothing provided
    (None, None) => {
      Err("You must provide either --file or --path".to_string())
    }

    // Both provided
    (Some(_), Some(_)) => {
      Err("You cannot use --file and --path together".to_string())
    }
  }
}

/// Main function that will:
/// 1. read the cli
/// 2. search docker/container-files
/// 3. build images
/// 4. audit packages
/// 5. output an html file
/// 6. cleanup the images 
fn main() {
  let cli = Cli::parse();

  let files = match get_files(&cli) {
    Ok(files) => files,
    Err(err) => {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
  };

  // for file in files {
  //   println!("Building image from '{}'", file.display());
  //   build_image(&file);
  // }
  
  let mut failures = Vec::new();

  for file in files {
    println!("Building image from '{}'", file.display());

    if let Err(err) = build_image(&file) {
      failures.push(format!("{}: {}", file.display(), err));
    }
  }

  if !failures.is_empty() {
    println!("\nFailed builds:");

    for failure in failures {
      println!("  {}", failure);
    }
  }
}
