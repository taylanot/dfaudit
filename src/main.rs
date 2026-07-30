//! ============================================================
//! File: main.rs
//! Author: taylanot
//! ============================================================

use dfaudit::audit;
use dfaudit::cli::Cli;
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

  logging::init(cli.verbose, cli.quiet);

  let files = explore::dfiles::get_files(&cli)?;

  let engine = Podman::new(cli.verbose);

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

    let mut report = audit::run(&engine, &cli.image_name)?;

    report.description = descrip::get_description(&file)?;

    report::json::write(&report, &cli.output, &file)?;

    engine.remove(&cli.image_name)?;
  }

  if !failures.is_empty() {
    log::error!("{} build(s) failed:", failures.len());

    for failure in failures {
      log::error!("  {}", failure);
    }
  }

  report::html::generate(&cli.output)?;

  Ok(())
}
