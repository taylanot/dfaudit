mod cli;
mod container;
mod explore;
mod audit;
mod report;
mod logging;
mod progress;

use clap::Parser;
use cli::Cli;
use container::podman::Podman;
use container::traits::ContainerEngine;
use progress::Spinner;


fn main() -> Result<(), Box<dyn std::error::Error>> {

  let cli = Cli::parse();


  logging::init(
    cli.verbose,
    cli.quiet,
  );


  let files =
    explore::dfiles::get_files(
      &cli
    )?;


  let engine =
    Podman::new(
      cli.verbose
    );


  let mut failures =
    Vec::new();



  for file in files {


    log::info!(
      "Building image from '{}'",
      file.display()
    );



    let spinner =
      Spinner::new(
        cli.verbose,
        "Building image..."
      );



    let build_result =
      engine.build(
        &file
      );



    spinner.finish();



    if let Err(err) = build_result {


      log::error!(
        "Build failed for '{}': {}",
        file.display(),
        err
      );


      failures.push(
        format!(
          "{}: {}",
          file.display(),
          err
        )
      );


      continue;

    }



    let report =
      audit::run(
        &engine,
        crate::cli::IMAGE_NAME,
      )?;



    report::json::write(
      &report,
      &cli.output,
      &file,
    )?;



    engine.remove(
      crate::cli::IMAGE_NAME
    )?;

  }



  if !failures.is_empty() {


    log::error!(
      "{} build(s) failed:",
      failures.len()
    );


    for failure in failures {

      log::error!(
        "  {}",
        failure
      );

    }

  }


  report::html::generate(
    &cli.output
  )?;



  Ok(())

}
