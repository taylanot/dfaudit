use crate::container::traits::ContainerEngine;

use super::models::Package;

pub fn audit<C: ContainerEngine>(
    engine: &C,
    image: &str,
) -> Result<Option<Vec<Package>>, String> {
    let output = engine.run(
        image,
        &["Rscript", "-e", "installed.packages()[,c('Package','Version')]"],
    );

    let output = match output {
        Ok(data) => data,

        Err(_) => return Ok(None),
    };

    let stdout = String::from_utf8_lossy(&output);

    let packages = stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();

            if parts.len() >= 2 {
                Some(Package {
                    name: parts[0].to_string(),

                    version: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(Some(packages))
}
