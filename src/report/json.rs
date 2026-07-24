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
