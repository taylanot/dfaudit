//! ============================================================
//! File: docker.rs
//! Author: taylanot
//! ============================================================

use std::{
  path::Path,
  process::{Command, Stdio},
};

use log::{debug, info};

use super::traits::ContainerEngine;

pub struct Docker {
  verbose: u8,
}

impl Docker {
  pub fn new(verbose: u8) -> Self {
    Docker {
      verbose,
    }
  }

  fn ready(&self) -> Result<(), String> {
    info!("Checking Docker installation");

    let mut command = Command::new("docker");

    command.arg("--version");

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if !status.success() {
      return Err("Docker is not installed or is unavailable.".into());
    }

    Ok(())
  }
}

impl ContainerEngine for Docker {
  fn build(&self, file: &Path) -> Result<(), String> {
    self.ready()?;

    info!("Building image from '{}'", file.display());

    let mut command = Command::new("docker");

    command
      .args(["build", "-t", "temp-image", "--format", "docker"])
      .arg("-f")
      .arg(file);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Build completed successfully");

      Ok(())
    } else {
      Err("Docker build failed.".into())
    }
  }

  fn run(&self, image: &str, command: &[&str]) -> Result<Vec<u8>, String> {
    self.ready()?;

    debug!("Running '{}' with {:?}", image, command);

    let output = Command::new("docker")
      .arg("run")
      .arg("--rm")
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
    info!("Removing temporary image '{}'...", image);

    let mut command = Command::new("docker");

    command.args(["rmi", image, "--force"]);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Temporary image removed.");

      Ok(())
    } else {
      Err("Failed to remove temporary image.".into())
    }
  }

  fn clean(&self) -> Result<(), String> {
    info!("Pruning the cache");

    let mut command = Command::new("docker");

    command.args(["system", "prune", "-af"]);

    if self.verbose <= 1 {
      command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
      info!("Pruned!");
      Ok(())
    } else {
      Err(format!("docker system prune failed with status: {status}"))
    }
  }
}
