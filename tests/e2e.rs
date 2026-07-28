//! ============================================================
//! File: e2e.rs
//! Author: taylanot
//! ============================================================

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command as StdCommand;

fn bin() -> Command {
  Command::cargo_bin("dfaudit").expect("binary should be built by cargo")
}

fn no_panic() -> impl Predicate<str> {
  predicate::str::contains("panicked at").not()
}

/// Absolute path to tests/fixtures/a, resolved relative to the crate
/// root so this works regardless of the directory `cargo test` is
/// invoked from.
fn fixture_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join("a")
}

fn cleanup_image(image_name: &str) {
  let _ = StdCommand::new("podman").args(["rmi", "-f", image_name]).output();
}

#[test]
#[ignore = "requires podman; run with `cargo test -- --ignored`"]
fn e2e_audits_and_reports() {
  let fixture = fixture_dir();
  assert!(
    fixture.join("Containerfile").exists(),
    "expected {:?} to contain a Dockerfile — check the fixture path",
    fixture
  );

  let output_dir =
    tempfile::tempdir().expect("failed to create temp output dir");
  let image_name = "dfaudit-e2e-fixture-a";

  cleanup_image(image_name);

  let result = std::panic::catch_unwind(|| {
    bin()
      .arg("--path")
      .arg(&fixture)
      .arg("--image-name")
      .arg(image_name)
      .arg("--output")
      .arg(output_dir.path())
      .assert()
      .success()
      .stderr(no_panic());

    // Filenames from report::json::write / report::html::generate
    // aren't confirmed yet, so check by extension via glob rather
    // than an exact path. Tighten this once the real names are known.
    let json_pattern = output_dir.path().join("**/*.json");
    let html_pattern = output_dir.path().join("**/*.html");

    let json_reports: Vec<_> = glob::glob(json_pattern.to_str().unwrap())
      .expect("invalid glob pattern")
      .filter_map(Result::ok)
      .collect();
    let html_reports: Vec<_> = glob::glob(html_pattern.to_str().unwrap())
      .expect("invalid glob pattern")
      .filter_map(Result::ok)
      .collect();

    assert!(
      !json_reports.is_empty(),
      "expected at least one .json report in {:?}, found none",
      output_dir.path()
    );
    assert!(
      !html_reports.is_empty(),
      "expected at least one .html report in {:?}, found none",
      output_dir.path()
    );

    let json_content = std::fs::read_to_string(&json_reports[0])
      .expect("failed to read json report");
    let parsed: serde_json::Value = serde_json::from_str(&json_content)
      .expect("json report is not valid JSON");
    assert!(!parsed.is_null(), "json report parsed to null");
  });

  cleanup_image(image_name);

  if let Err(err) = result {
    std::panic::resume_unwind(err);
  }
}
