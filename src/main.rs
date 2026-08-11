//! ============================================================
//! File: main.rs
//! Author: taylanot
//! ============================================================

use dfaudit::audit;
use dfaudit::cli::Cli;
use dfaudit::container::docker::Docker;
use dfaudit::container::podman::Podman;
use dfaudit::descrip;
use dfaudit::explore;
use dfaudit::logging;
use dfaudit::progress::Spinner;
use dfaudit::report;

use clap::Parser;
use dfaudit::container::traits::ContainerEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  let mut failed = false;

  logging::init(cli.verbose, cli.quiet);

  if cli.build {
    let files = explore::dfiles::get_files(&cli)?;

    let engine: Box<dyn ContainerEngine> = match cli.engine.as_str() {
      "podman" => Box::new(Podman::new(cli.verbose)),
      "docker" => Box::new(Docker::new(cli.verbose)),
      other => {
        return Err(format!("Unknown container engine: {}", other).into());
      }
    };

    let mut failures = Vec::new();

    for file in files {
      log::info!("Building image from '{}'", file.display());

      let spinner = Spinner::new(cli.verbose, "Building image...");

      let build_result = engine.build(&file);

      spinner.finish();

      if let Err(err) = build_result {
        log::error!("Build failed for '{}': {}", file.display(), err);
        failures.push(format!("{}: {}", file.display(), err));

        continue;
      }

      let mut report = audit::run(engine.as_ref(), &cli.image_name)?;

      report.description = descrip::get_description(&file)?;

      report::json::write(&report, &cli.output, &file)?;

      engine.remove(&cli.image_name)?;

      if cli.prune {
        let spinner = Spinner::new(cli.verbose, "Cleaning the system...");

        engine.clean()?;

        spinner.finish();
      }
    }

    if !failures.is_empty() {
      log::error!("{} build(s) failed:", failures.len());
      failed = true;
      for failure in failures {
        log::error!("  {}", failure);
      }
    }
  }

  if cli.html {
    log::info!("Generating html output '{}'", cli.output.display());
    report::html::generate(&cli.output)?;
  }

  if failed {
    std::process::exit(1);
  }

  Ok(())
}
