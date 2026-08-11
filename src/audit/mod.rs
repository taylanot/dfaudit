//! ============================================================
//! File: mod.rs
//! Author: taylanot
//! ============================================================
pub mod models;
mod python;
mod r;
use crate::container::traits::ContainerEngine;
use models::AuditReport;
pub fn run<C: ContainerEngine + ?Sized>(
  engine: &C,
  image: &str,
) -> Result<AuditReport, String> {
  Ok(AuditReport {
    python_packages: python::audit(engine, image)?,
    r_packages: r::audit(engine, image)?,
    description: None,
  })
}
#[cfg(test)]
pub(crate) mod test_utils;
