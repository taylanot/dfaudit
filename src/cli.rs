use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
  #[arg(short = 'f', long = "file")]
  pub file: Option<PathBuf>,

  #[arg(short = 'p', long = "path")]
  pub path: Option<PathBuf>,

  #[arg(short, long, default_value = "audit")]
  pub output: PathBuf,

  /// Increase logging verbosity
  #[arg( short, long, action = clap::ArgAction::Count)]
  pub verbose: u8,

  #[arg(short, long)]
  pub quiet: bool,

  #[arg(short, long, default_value = "temp-image")]
  pub image_name: String,
}
