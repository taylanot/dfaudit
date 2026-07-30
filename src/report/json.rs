//! ============================================================
//! File: json.rs
//! Author: taylanot
//! ============================================================

use std::{fs, path::Path};

use crate::audit::models::AuditReport;

pub fn write(
  report: &AuditReport,
  output: &Path,
  dockerfile: &Path,
) -> Result<(), String> {
  let parent = dockerfile.parent().ok_or("cannot find parent directory")?;

  let name = parent.file_name().ok_or("cannot get directory name")?;

  let directory = output.join(name);

  fs::create_dir_all(&directory).map_err(|e| e.to_string())?;

  let file = directory.join("audit-report.json");

  let json =
    serde_json::to_string_pretty(report).map_err(|e| e.to_string())?;

  fs::write(file, json).map_err(|e| e.to_string())?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::audit::models::AuditReport;
  use std::fs;
  use tempfile::tempdir;

  #[test]
  fn writes_audit_report() {
    let temp = tempdir().unwrap();

    let output = temp.path().join("output");

    let dockerfile = temp.path().join("my-image").join("Dockerfile");

    fs::create_dir_all(dockerfile.parent().unwrap()).unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write(&report, &output, &dockerfile).unwrap();

    let report_file = output.join("my-image").join("audit-report.json");

    assert!(report_file.exists());
  }

  #[test]
  fn writes_valid_json() {
    let temp = tempdir().unwrap();

    let output = temp.path().join("output");

    let dockerfile = temp.path().join("project").join("Dockerfile");

    fs::create_dir_all(dockerfile.parent().unwrap()).unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write(&report, &output, &dockerfile).unwrap();

    let file = output.join("project").join("audit-report.json");

    let contents = fs::read_to_string(file).unwrap();

    let parsed: AuditReport = serde_json::from_str(&contents).unwrap();

    assert_eq!(parsed.python_packages, None);
    assert_eq!(parsed.r_packages, None);
  }

  #[test]
  fn fails_when_dockerfile_has_no_parent() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    // Path with no parent component
    let dockerfile = Path::new("");

    let result = write(&report, temp.path(), dockerfile);

    assert!(result.is_err());
  }

  #[test]
  fn uses_dockerfile_parent_directory_name() {
    let temp = tempdir().unwrap();

    let output = temp.path().join("reports");

    let dockerfile = temp.path().join("my-container").join("Dockerfile");

    fs::create_dir_all(dockerfile.parent().unwrap()).unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write(&report, &output, &dockerfile).unwrap();

    assert!(output.join("my-container").join("audit-report.json").exists());
  }
}
