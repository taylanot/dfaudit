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

use chrono::{DateTime, Local};
use std::{time::SystemTime};
use std::fs;

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

/// Ensures Podman is available and, if necessary, starts the Podman machine.
///
/// On Linux:
///   - Only checks that `podman` exists.
///
/// On macOS/Windows:
///   - Checks that `podman` exists.
///   - Starts the default Podman machine if it isn't already running.
fn podman_ready() -> Result<(), String> {
    // Check whether podman exists
    let version = Command::new("podman")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match version {
        Ok(status) if status.success() => {}
        Ok(_) => {
            return Err("Podman is installed but failed to run.".to_string());
        }
        Err(_) => {
            return Err(
                "Podman is not installed or is not available in PATH.".to_string(),
            );
        }
    }

    // Only macOS and Windows require a Podman machine.
    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        let status = Command::new("podman")
            .args(["machine", "start"])
            .status()
            .map_err(|e| format!("Failed to start Podman machine: {e}"))?;

        if !status.success() {
            return Err(format!(
                "Failed to start Podman machine (exit code: {status})"
            ));
        }
    }

    Ok(())
}

/// Build a container image with podman from the path
fn build_image(file: &Path) -> Result<(), String> {

  podman_ready()?;
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

fn html_report( audit_root: &Path,) -> Result<(), Box<dyn std::error::Error>> {

  let mut html = String::new();

html.push_str(r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Audit Report</title>
<style>
:root{
  --jh-orange:#F37626;
  --jh-orange-dark:#E66A1F;
  --jh-orange-light:#F5A252;
  --jh-grey-900:#1a1a1a;
  --jh-grey-800:#2b2b2b;
  --jh-grey-100:#f7f7f7;
  --jh-grey-200:#ececeb;
  --jh-border:#e0e0e0;
  --jh-text:#333333;
}
*{ box-sizing:border-box; }
body{
  font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;
  background:var(--jh-grey-100);
  margin:0;
  color:var(--jh-text);
}
.topbar{
  background:var(--jh-grey-900);
  border-bottom:4px solid var(--jh-orange);
  padding:18px 40px;
  display:flex;
  align-items:center;
  gap:14px;
}
.topbar .logo{
  width:32px;
  height:32px;
  flex-shrink:0;
}
.topbar h1{
  color:#fff;
  font-size:1.3rem;
  font-weight:600;
  margin:0;
  letter-spacing:.3px;
}
.topbar .spacer{
  flex:1;
}
.gh-link{
  display:flex;
  align-items:center;
  gap:8px;
  color:#fff;
  text-decoration:none;
  font-size:.9rem;
  font-weight:500;
  padding:7px 14px;
  border:1px solid rgba(255,255,255,.25);
  border-radius:6px;
  transition:background .15s ease, border-color .15s ease;
}
.gh-link:hover{
  background:rgba(243,118,38,.15);
  border-color:var(--jh-orange);
}
.gh-link svg{
  width:18px;
  height:18px;
  fill:#fff;
}
.container{
  max-width:1100px;
  margin:0 auto;
  padding:30px 20px 60px;
}
.search-bar{
  position:sticky;
  top:0;
  z-index:10;
  background:var(--jh-grey-100);
  padding:18px 0 6px;
  margin-bottom:10px;
}
.search-bar input{
  width:100%;
  padding:11px 16px;
  font-size:.95rem;
  border:1px solid var(--jh-border);
  border-radius:6px;
  background:#fff;
  color:var(--jh-text);
  outline:none;
  box-shadow:0 1px 3px rgba(0,0,0,.06);
  transition:border-color .15s ease, box-shadow .15s ease;
}
.search-bar input:focus{
  border-color:var(--jh-orange);
  box-shadow:0 0 0 3px rgba(243,118,38,.15);
}
.search-bar input::placeholder{
  color:#999;
}
.no-match-note{
  text-align:center;
  color:#999;
  font-style:italic;
  padding:30px 0;
  display:none;
}
tr.pkg-row.hidden{
  display:none;
}
details.no-visible-rows{
  display:none;
}
.card.no-visible-details{
  display:none;
}
.card{
  background:#fff;
  margin:0 0 26px;
  border:1px solid var(--jh-border);
  border-radius:6px;
  box-shadow:0 1px 3px rgba(0,0,0,.06);
  overflow:hidden;
}
.card .header{
  display:flex;
  justify-content:space-between;
  align-items:center;
  padding:16px 22px;
  background:var(--jh-grey-200);
  border-bottom:3px solid var(--jh-orange);
}
.card .header h2{
  margin:0;
  font-size:1.15rem;
  color:var(--jh-grey-900);
  font-weight:600;
}
.updated{
  color:#777;
  font-size:.8rem;
}
.card-body{
  padding:20px 22px 24px;
}
details{
  margin:14px 0 0;
  border:1px solid var(--jh-border);
  border-radius:6px;
  overflow:hidden;
}
details:first-child{
  margin-top:0;
}
details + details{
  margin-top:14px;
}
summary{
  cursor:pointer;
  list-style:none;
  padding:11px 16px;
  background:var(--jh-orange);
  color:#fff;
  font-weight:600;
  font-size:.9rem;
  text-transform:uppercase;
  letter-spacing:.04em;
  display:flex;
  align-items:center;
  justify-content:space-between;
  user-select:none;
}
summary::-webkit-details-marker{ display:none; }
summary::after{
  content:"▸";
  font-size:.85rem;
  transition:transform .15s ease;
}
details[open] summary::after{
  transform:rotate(90deg);
}
summary:hover{
  background:var(--jh-orange-dark);
}
table{
  width:100%;
  border-collapse:collapse;
  font-size:.9rem;
}
th{
  background:var(--jh-grey-200);
  color:var(--jh-grey-900);
  padding:9px 16px;
  text-align:left;
  font-weight:600;
  border-bottom:1px solid var(--jh-border);
}
td{
  padding:8px 16px;
  border-bottom:1px solid var(--jh-border);
}
tbody tr:last-child td{
  border-bottom:none;
}
tbody tr:nth-child(even){
  background:var(--jh-grey-100);
}
tbody tr:hover{
  background:#fdeee0;
}
td[colspan]{
  text-align:center;
  color:#999;
  font-style:italic;
  background:#fff !important;
}
</style>
</head>
<body>
<div class="topbar">
  <h1>dfaudit (Dockerfile Audit) </h1>
  <div class="spacer"></div>
  <a class="gh-link" href="https://github.com/taylanot/dfaudit" target="_blank" rel="noopener noreferrer">
    <svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"/>
    </svg>
    View on GitHub
  </a>
</div>
<div class="container">
<div class="search-bar">
  <input
    type="text"
    id="pkg-search"
    placeholder="Search for a package by name..."
    oninput="filterPackages(this.value)"
    autocomplete="off"
  >
</div>
<div class="no-match-note" id="no-match-note">No packages match your search.</div>
"##);

  let mut dirs = fs::read_dir(audit_root)?
    .filter_map(Result::ok)
    .collect::<Vec<_>>();
  dirs.sort_by_key(|e| e.path());

  for dir in dirs {
    let path = dir.path();
    if !path.is_dir() {
      continue;
    }
    let report = path.join("audit-report.json");
    if !report.exists() {
      continue;
    }
    let report_json = fs::read_to_string(&report)?;
    let audit: AuditReport = serde_json::from_str(&report_json)?;
    let modified = fs::metadata(&report)?.modified()?;
    let modified: DateTime<Local> = modified.into();
    let project = path
      .file_name()
      .unwrap()
      .to_string_lossy();

    html.push_str(&format!(
      r#"
<div class="card">
<div class="header">
  <h2>{}</h2>
  <div class="updated">Last Updated: {}</div>
</div>
<div class="card-body">
"#,
      project,
      modified.format("%Y-%m-%d %H:%M:%S"),
    ));

    let python_count = audit.python_packages.as_ref().map(|p| p.len()).unwrap_or(0);
    html.push_str(&format!(
      r#"
<details>
<summary>Show Python Packages ({})</summary>
<table>
<thead>
<tr>
  <th>Package</th>
  <th>Version</th>
</tr>
</thead>
<tbody>
"#,
      python_count
    ));

    if let Some(packages) = audit.python_packages {
      for package in packages {
        html.push_str(&format!(
          r#"
<tr class="pkg-row" data-name="{}">
  <td>{}</td>
  <td>{}</td>
</tr>
"#,
          package.name.to_lowercase(),
          package.name,
          package.version
        ));
      }
    } else {
      html.push_str(
        r#"
<tr>
  <td colspan="2">No Python packages found.</td>
</tr>
"#,
      );
    }
    html.push_str(
      r#"
</tbody>
</table>
</details>
"#,
    );

    if let Some(packages) = audit.r_packages {
      html.push_str(&format!(
        r#"
<details>
<summary>Show R Packages ({})</summary>
<table>
<thead>
<tr>
  <th>Package</th>
  <th>Version</th>
</tr>
</thead>
<tbody>
"#,
        packages.len()
      ));
      for package in packages {
        html.push_str(&format!(
          r#"
<tr class="pkg-row" data-name="{}">
  <td>{}</td>
  <td>{}</td>
</tr>
"#,
          package.name.to_lowercase(),
          package.name,
          package.version
        ));
      }
      html.push_str(
        r#"
</tbody>
</table>
</details>
"#,
      );
    }

    html.push_str("</div></div>");
  }

  html.push_str(
    r##"
</div>
<script>
function filterPackages(query) {
  const q = query.trim().toLowerCase();
  const rows = document.querySelectorAll("tr.pkg-row");
  let anyMatchOverall = false;

  rows.forEach((row) => {
    const name = row.getAttribute("data-name") || "";
    const matches = q === "" || name.includes(q);
    row.classList.toggle("hidden", !matches);
    if (matches) anyMatchOverall = true;
  });

  document.querySelectorAll("details").forEach((details) => {
    const visibleRows = details.querySelectorAll("tr.pkg-row:not(.hidden)");
    const hasRows = details.querySelectorAll("tr.pkg-row").length > 0;
    if (hasRows) {
      details.classList.toggle("no-visible-rows", visibleRows.length === 0);
      details.open = q !== "" && visibleRows.length > 0;
    }
  });

  document.querySelectorAll(".card").forEach((card) => {
    const visibleRows = card.querySelectorAll("tr.pkg-row:not(.hidden)");
    const hasAnyRows = card.querySelectorAll("tr.pkg-row").length > 0;
    card.classList.toggle(
      "no-visible-details",
      q !== "" && hasAnyRows && visibleRows.length === 0
    );
  });

  document.getElementById("no-match-note").style.display =
    q !== "" && !anyMatchOverall ? "block" : "none";
}
</script>
</body>
</html>
"##,
  );

  fs::write(audit_root.join("index.html"), html)?;
  Ok(())
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

      eprintln!("X Build failed: {}", failure);

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
  
  html_report(&cli.output); 

  Ok(())
}
