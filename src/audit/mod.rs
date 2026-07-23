mod python;
mod r;

pub mod models;

use crate::container::traits::ContainerEngine;

use models::AuditReport;

pub fn run<C: ContainerEngine>(
    engine: &C,
    image: &str,
) -> Result<AuditReport, String> {
    Ok(AuditReport {
        python_packages: python::audit(engine, image)?,

        r_packages: r::audit(engine, image)?,
    })
}
