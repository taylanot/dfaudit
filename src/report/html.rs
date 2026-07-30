//! ============================================================
//! File: html.rs
//! Author: taylanot
//! ============================================================

use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

use crate::audit::models::AuditReport;

pub fn generate(audit_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
.description{
  margin-bottom:20px;
  padding:12px 16px;
  background:#fff7ed;
  border-left:4px solid var(--jh-orange);
  border-radius:4px;
  color:var(--jh-grey-900);
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

  /*
   * Walk all audit directories
   */
  for entry in
    walkdir::WalkDir::new(audit_root).into_iter().filter_map(Result::ok)
  {
    if !entry.file_type().is_file() {
      continue;
    }

    if entry.file_name() != "audit-report.json" {
      continue;
    }

    let report_path = entry.path();

    let contents = fs::read_to_string(report_path)?;

    let audit: AuditReport = serde_json::from_str(&contents)?;

    let modified = fs::metadata(report_path)?.modified()?;

    let modified: DateTime<Local> = modified.into();

    let project = report_path
      .parent()
      .and_then(|p| p.file_name())
      .unwrap()
      .to_string_lossy();

    let description = audit
      .description
      .as_deref()
      .unwrap_or("No description available");

    html.push_str(&format!(
      r#"
    <div class="card">
    <div class="header">
      <h2>{}</h2>
      <div class="updated">Last Updated: {}</div>
    </div>

    <div class="card-body">

    <div class="description">
      <strong>Description:</strong> {}
    </div>
    "#,
      project,
      modified.format("%Y-%m-%d %H:%M:%S"),
      description,
    ));

    write_packages(&mut html, "Python Packages", audit.python_packages);

    write_packages(&mut html, "R Packages", audit.r_packages);

    html.push_str(
      r#"

</div>

</div>

"#,
    );
  }

  html.push_str(
    r##"

</div>


<script>


function filterPackages(value){

  let query =
    value.toLowerCase();


  document
    .querySelectorAll(".pkg")
    .forEach(row=>{


      let name =
        row.dataset.name;


      if(
        name.includes(query)
        ||
        query === ""
      ){

        row.classList.remove(
          "hidden"
        );

      }
      else{

        row.classList.add(
          "hidden"
        );

      }

    });

}


</script>


</body>

</html>

"##,
  );

  fs::write(audit_root.join("index.html"), html)?;

  Ok(())
}

fn write_packages(
  html: &mut String,
  title: &str,
  packages: Option<Vec<crate::audit::models::Package>>,
) {
  let packages = match packages {
    Some(p) => p,

    None => return,
  };

  html.push_str(&format!(
    r#"

<details>

<summary>
{} ({})
</summary>


<table>

<thead>

<tr>
<th>Name</th>
<th>Version</th>
</tr>

</thead>


<tbody>

"#,
    title,
    packages.len()
  ));

  for package in packages {
    html.push_str(&format!(
      r#"

<tr class="pkg" data-name="{}">

<td>
{}
</td>

<td>
{}
</td>

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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::audit::models::{AuditReport, Package};
  use std::fs;
  use tempfile::tempdir;

  fn write_report(root: &Path, project: &str, report: &AuditReport) {
    let dir = root.join(project);

    fs::create_dir_all(&dir).unwrap();

    let json = serde_json::to_string_pretty(report).unwrap();

    fs::write(dir.join("audit-report.json"), json).unwrap();
  }

  #[test]
  fn generates_index_html() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write_report(temp.path(), "my-project", &report);

    generate(temp.path()).unwrap();

    assert!(temp.path().join("index.html").exists());
  }

  #[test]
  fn includes_project_name() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write_report(temp.path(), "backend-api", &report);

    generate(temp.path()).unwrap();

    let html = fs::read_to_string(temp.path().join("index.html")).unwrap();

    assert!(html.contains("backend-api"));
  }

  #[test]
  fn includes_package_information() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      python_packages: Some(vec![Package {
        name: "requests".into(),
        version: "2.32.0".into(),
      }]),
      r_packages: None,
      description: None,
    };

    write_report(temp.path(), "python-app", &report);

    generate(temp.path()).unwrap();

    let html = fs::read_to_string(temp.path().join("index.html")).unwrap();

    assert!(html.contains("requests"));
    assert!(html.contains("2.32.0"));
    assert!(html.contains("Python Packages"));
  }

  #[test]
  fn ignores_non_report_files() {
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("random.json"), "{}").unwrap();

    generate(temp.path()).unwrap();

    let html = fs::read_to_string(temp.path().join("index.html")).unwrap();

    assert!(!html.contains("random"));
  }

  #[test]
  fn fails_on_invalid_report_json() {
    let temp = tempdir().unwrap();

    let project = temp.path().join("broken");

    fs::create_dir_all(&project).unwrap();

    fs::write(project.join("audit-report.json"), "not valid json").unwrap();

    let result = generate(temp.path());

    assert!(result.is_err());
  }

  #[test]
  fn generates_multiple_projects() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      description: None,
      python_packages: None,
      r_packages: None,
    };

    write_report(temp.path(), "project-a", &report);

    write_report(temp.path(), "project-b", &report);

    generate(temp.path()).unwrap();

    let html = fs::read_to_string(temp.path().join("index.html")).unwrap();

    assert!(html.contains("project-a"));
    assert!(html.contains("project-b"));
  }
  #[test]
  fn includes_description() {
    let temp = tempdir().unwrap();

    let report = AuditReport {
      description: Some("A web server image".into()),
      python_packages: None,
      r_packages: None,
    };

    write_report(temp.path(), "docker-app", &report);

    generate(temp.path()).unwrap();

    let html = fs::read_to_string(temp.path().join("index.html")).unwrap();

    assert!(html.contains("A web server image"));
  }
}
