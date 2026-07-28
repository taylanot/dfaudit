use std::fs;
use std::process::Command;

use dfaudit::container::podman::Podman;
use dfaudit::container::traits::ContainerEngine;

fn podman_available() -> bool {
  Command::new("podman")
    .arg("--version")
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

#[test]
fn integ_all_good() {
  if !podman_available() {
    eprintln!("Skipping test: Podman is not available");
    return;
  }

  let temp = tempfile::tempdir().expect("failed to create temp directory");

  let containerfile = temp.path().join("Containerfile");

  fs::write(
    &containerfile,
    r#"
        FROM alpine
        CMD ["echo", "hello"]
        "#,
  )
  .expect("failed to create Containerfile");

  let podman = Podman::new(0);

  podman.build(&containerfile).expect("build failed");

  let output = podman.run("temp-image", &[]).expect("run failed");

  assert_eq!(String::from_utf8_lossy(&output).trim(), "hello");

  podman.remove("temp-image").expect("remove failed");
}

#[test]
fn integ_missing_file() {
  if !podman_available() {
    eprintln!("Skipping test: Podman is not available");
    return;
  }

  let podman = Podman::new(0);

  let result = podman.build(std::path::Path::new("missing/Containerfile"));

  assert!(result.is_err());
}
