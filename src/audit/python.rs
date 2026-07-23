use crate::container::traits::ContainerEngine;

use super::models::Package;



pub fn audit<C: ContainerEngine>(
  engine:&C,
  image:&str,
) -> Result<Option<Vec<Package>>,String> {


  let output =
    engine.run(
      image,
      &[
        "python3",
        "-m",
        "pip",
        "list",
        "--format=json",
      ],
    );


  let output =
    match output {

      Ok(data) => data,

      Err(_) => {
        return Ok(None)
      }
    };



  let packages =
    serde_json::from_slice::<Vec<Package>>(
      &output
    )
    .map_err(
      |e|format!(
        "failed parsing python packages: {}",
        e
      )
    )?;



  Ok(Some(packages))
}
