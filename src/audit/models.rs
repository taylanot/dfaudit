//! ============================================================
//! File: models.rs
//! Author: taylanot
//! ============================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditReport {
  pub description: Option<String>,

  pub python_packages: Option<Vec<Package>>,

  pub r_packages: Option<Vec<Package>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Package {
  pub name: String,

  pub version: String,
}
