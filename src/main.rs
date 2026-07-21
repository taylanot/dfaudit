use clap::Parser;
use std::path::PathBuf;
use std::path::Path;
use walkdir::WalkDir;
use std::process::Command;

use std::process::Stdio;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use serde::{Serialize,Deserialize};
use std::fs::File;
use std::io::Write;


static IMAGE_NAME: &str = "temp-image";

#[derive(Debug, Serialize, Deserialize)]
struct AuditReport {
  python_packages: Option<Vec<Package>>,
  r_packages: Option<Vec<Package>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
  name: String,
  version: String,
}

fn write_report(report: &AuditReport, filename: &str) -> Result<(), String> {
  let json = serde_json::to_string_pretty(report)
    .map_err(|e| format!("failed to serialize report: {e}"))?;

  let mut file = File::create(filename)
    .map_err(|e| format!("failed to create report file: {e}"))?;

  file.write_all(json.as_bytes())
    .map_err(|e| format!("failed to write report: {e}"))?;

  Ok(())
}

/// Get the data
#[derive(Parser)]
struct Cli {
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    #[arg(short = 'p', long = "path")]
    path: Option<PathBuf>,

    #[arg(short, long, default_value = "audit")]
    output: PathBuf,
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
    .args(["build", "-t", &IMAGE_NAME])
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

  Ok(())
}

/// Clean the temporary image
fn clean_image( ) -> Result<(), String> {

  println!("Now deleting the temporary image! Image tagged as temp-image");

  let remove_status = Command::new("podman")
    .args(["rmi", &IMAGE_NAME])
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

fn audit_python(image: &str) -> Result<Option<Vec<Package>>, String> {
  let output = Command::new("podman")
    .args([
      "run",
      "--rm",
      image,
      "python3",
      "-m",
      "pip",
      "list",
      "--format=json",
    ])
    .output()
    .map_err(|e| format!("failed to run pip audit: {e}"))?;

  if !output.status.success() {
    return Ok(None);
  }

  let packages: Vec<Package> = serde_json::from_slice(&output.stdout)
    .map_err(|e| format!("failed parsing pip output: {e}"))?;

  Ok(Some(packages))
}

fn audit_r(image: &str) -> Result<Option<Vec<Package>>, String> {
  let output = Command::new("podman")
    .args([
      "run",
      "--rm",
      image,
      "Rscript",
      "-e",
      "installed.packages()[,c('Package','Version')]",
    ])
    .output()
    .map_err(|e| format!("failed to run R audit: {e}"))?;

  // R is not installed in the image
  if !output.status.success() {
    return Ok(None);
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  let packages = stdout
    .lines()
    .skip(1) // skip header
    .filter_map(|line| {
      let parts: Vec<&str> = line.split_whitespace().collect();

      if parts.len() >= 2 {
        Some(Package {
          name: parts[0].to_string(),
          version: parts[1].to_string(),
        })
      } else {
        None
      }
    })
    .collect();

  Ok(Some(packages))
}

fn report_directory(output: &Path, file: &Path) -> Result<PathBuf, String> {
  let parent = file
    .parent()
    .ok_or("Could not determine Dockerfile directory")?;

  let dir_name = parent
    .file_name()
    .ok_or("Could not get directory name")?;

  let output_dir = output.join(dir_name);

  std::fs::create_dir_all(&output_dir)
    .map_err(|e| format!("failed creating report directory: {e}"))?;

  Ok(output_dir)
}
/// Main function that will:
/// 1. read the cli
/// 2. search docker/container-files
/// 3. build images
/// 4. audit packages
/// 5. output an html file
/// 6. cleanup the images

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();

  let files = get_files(&cli)?;

  let mut failures = Vec::new();

  for file in files {
    println!("Building image from '{}'", file.display());

    if let Err(err) = build_image(&file) {
      let failure = format!("{}: {}", file.display(), err);

      eprintln!("⚠ Build failed: {}", failure);

      failures.push(failure);
      continue;
    }

    let report = AuditReport {
      python_packages: audit_python(IMAGE_NAME)?,
      r_packages: audit_r(IMAGE_NAME)?,
    };

    let report_dir = report_directory(&cli.output, &file)?;

    let report_file = report_dir.join("audit-report.json");

    write_report(
      &report,
      report_file.to_str().unwrap(),
    )?;

    println!("Report written to '{}'", report_file.to_str().unwrap());

    clean_image( )?;
  }

  if !failures.is_empty() {
    println!("\nFailed builds:");

    for failure in failures {
      println!("  {}", failure);
    }
  }

  Ok(())
}
