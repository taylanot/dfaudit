//! ============================================================
//! File: smoke.rs
//! Author: taylanot
//! ============================================================

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
  Command::cargo_bin("dfaudit").expect("binary should be built by cargo")
}

/// No test in this file should ever see a panic message on stderr —
/// clean failures should look like log::error! output, not a Rust panic.
fn no_panic() -> impl Predicate<str> {
  predicate::str::contains("panicked at").not()
}

#[test]
fn smoke_help_and_exit() {
  bin()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn smoke_version_and_exit() {
  bin()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn smoke_reject_flag() {
  bin().arg("--this-flag-does-not-exist").assert().failure();
}

#[test]
fn smoke_without_argument() {
  // Every field on Cli is optional/has a default, so this is a valid
  // invocation. Behavior of explore::dfiles::get_files with file=None,
  // path=None is unspecified from the outside, so we only assert it
  // doesn't panic and doesn't hang.
  let output_dir = tempfile::tempdir().expect("failed to create temp dir");

  bin().current_dir(output_dir.path()).assert().stderr(no_panic());
}

#[test]
fn smoke_nonexistent_file() {
  let output_dir =
    tempfile::tempdir().expect("failed to create temp output dir");
  let missing_file = output_dir.path().join("Dockerfile.does-not-exist");

  bin()
    .arg("--quiet")
    .arg("--file")
    .arg(&missing_file)
    .arg("--output")
    .arg(output_dir.path().join("audit"))
    .assert()
    .stderr(no_panic());
}

#[test]
fn smoke_no_dockerfile() {
  let empty_dir = tempfile::tempdir().expect("failed to create temp dir");
  let output_dir =
    tempfile::tempdir().expect("failed to create temp output dir");

  bin()
    .arg("--quiet")
    .arg("--path")
    .arg(empty_dir.path())
    .arg("--output")
    .arg(output_dir.path().join("audit"))
    .assert()
    .stderr(no_panic());
}

#[test]
fn log_and_moveon() {
  // A syntactically-present but invalid Dockerfile should make
  // engine.build() return Err, which main.rs should catch, log, and
  // move past — not propagate as a crash.
  let dir = tempfile::tempdir().expect("failed to create temp dir");
  let output_dir =
    tempfile::tempdir().expect("failed to create temp output dir");
  let dockerfile = dir.path().join("Dockerfile");
  std::fs::write(&dockerfile, "THIS IS NOT VALID DOCKERFILE SYNTAX\n")
    .expect("failed to write test Dockerfile");

  let assert = bin()
    .arg("--quiet")
    .arg("--file")
    .arg(&dockerfile)
    .arg("--image-name")
    .arg("dfaudit-smoke-test-image")
    .arg("--output")
    .arg(output_dir.path().join("audit"))
    .assert();

  assert.stderr(no_panic());
}

#[test]
fn smoke_verbosity_flag() {
  let output_dir = tempfile::tempdir().expect("failed to create temp dir");

  bin()
    .arg("-vvv")
    .arg("--path")
    .arg(output_dir.path())
    .arg("--output")
    .arg(output_dir.path().join("audit"))
    .assert()
    .stderr(no_panic());
}

#[test]
fn smoke_shhh_quite() {
  let output_dir = tempfile::tempdir().expect("failed to create temp dir");

  bin()
    .arg("--quiet")
    .arg("--path")
    .arg(output_dir.path())
    .arg("--output")
    .arg(output_dir.path().join("audit"))
    .assert()
    .stderr(no_panic());
}
