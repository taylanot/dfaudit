//! ============================================================
//! File: podman.rs
//! Author: taylanot
//! ============================================================

use std::{
  env,
  path::Path,
  process::{Command, Stdio},
};

use log::{debug, info};

use super::traits::ContainerEngine;

pub struct Podman {
  verbose: u8,
}

impl Podman {
  pub fn new(verbose: u8) -> Self {
    Podman {
      verbose,
    }
  }

  fn ready(&self) -> Result<(), String> {
    info!("Checking Podman installation");

    let mut command = Command::new("podman");

    command.arg("--version");

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if !status.success() {
      return Err("Podman is not installed or is unavailable.".into());
    }

    self.start_machine()?;

    Ok(())
  }

  fn start_machine(&self) -> Result<(), String> {
    let os = env::consts::OS;

    // Linux does not use Podman Machine.
    if os != "macos" && os != "windows" {
      debug!("Podman Machine not required on {}", os);

      return Ok(());
    }

    info!("Checking Podman machine status");

    let output = Command::new("podman")
      .args(["machine", "info"])
      .output()
      .map_err(|e| e.to_string())?;

    if !output.status.success() {
      return Err("Failed to query Podman machine.".into());
    }

    let machine_info = String::from_utf8_lossy(&output.stdout);

    let running =
      machine_info.lines().any(|line| line.trim() == "machinestate: Running");

    if running {
      info!("Podman machine is already running");

      return Ok(());
    }

    info!("Starting Podman machine");

    let mut command = Command::new("podman");

    command.args(["machine", "start"]);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Podman machine started");

      Ok(())
    } else {
      Err("Failed to start Podman machine.".into())
    }
  }
}

impl ContainerEngine for Podman {
  fn build(&self, file: &Path) -> Result<(), String> {
    self.ready()?;

    info!("Building image from '{}'", file.display());

    let mut command = Command::new("podman");

    command.args(["build", "-t", "temp-image"]).arg("-f").arg(file);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Build completed successfully");

      Ok(())
    } else {
      Err("Podman build failed.".into())
    }
  }

  fn run(&self, image: &str, command: &[&str]) -> Result<Vec<u8>, String> {
    self.ready()?;

    debug!("Running '{}' with {:?}", image, command);

    let output = Command::new("podman")
      .arg("run")
      .arg("--rm")
      .arg("--rmi")
      .arg(image)
      .args(command)
      .output()
      .map_err(|e| e.to_string())?;

    if output.status.success() {
      Ok(output.stdout)
    } else {
      Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
  }

  fn remove(&self, image: &str) -> Result<(), String> {
    info!("Removing temporary image '{}'", image);

    let mut command = Command::new("podman");

    command.args(["rmi", image, "--force"]);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Temporary image removed");

      Ok(())
    } else {
      Err("Failed to remove temporary image.".into())
    }
  }
}
