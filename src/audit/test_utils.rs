//! ============================================================
//! File: test_utils.rs
//! Author: taylanot
//! ============================================================

use std::collections::HashMap;
use std::path::Path;

use crate::container::traits::ContainerEngine;

pub struct MockEngine {
  responses: HashMap<Vec<String>, Result<Vec<u8>, String>>,
}

impl MockEngine {
  pub fn new() -> Self {
    Self {
      responses: HashMap::new(),
    }
  }

  pub fn with_response(
    mut self,
    command: &[&str],
    response: Result<Vec<u8>, String>,
  ) -> Self {
    self
      .responses
      .insert(command.iter().map(|s| s.to_string()).collect(), response);

    self
  }
}

impl ContainerEngine for MockEngine {
  fn build(&self, _file: &Path) -> Result<(), String> {
    Ok(())
  }

  fn run(&self, _image: &str, command: &[&str]) -> Result<Vec<u8>, String> {
    self
      .responses
      .get(&command.iter().map(|s| s.to_string()).collect::<Vec<_>>())
      .expect("unexpected command")
      .clone()
  }

  fn remove(&self, _image: &str) -> Result<(), String> {
    Ok(())
  }

  fn clean(&self) -> Result<(), String> {
    Ok(())
  }
}
