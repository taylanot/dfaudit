use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Default)]
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

/// I believe this to be unnecessary but oke!
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_values() {
    let cli = Cli::parse_from(["dfaudit"]);

    assert_eq!(cli.output, PathBuf::from("audit"));
    assert_eq!(cli.image_name, "temp-image");
    assert_eq!(cli.verbose, 0);
    assert!(!cli.quiet);
    assert!(cli.file.is_none());
    assert!(cli.path.is_none());
  }

  #[test]
  fn file_argument() {
    let cli = Cli::parse_from(["dfaudit", "--file", "Dockerfile"]);

    assert_eq!(cli.file, Some(PathBuf::from("Dockerfile")));
  }

  #[test]
  fn path_argument() {
    let cli = Cli::parse_from(["dfaudit", "--path", "/tmp/project"]);

    assert_eq!(cli.path, Some(PathBuf::from("/tmp/project")));
  }

  #[test]
  fn custom_output_and_image() {
    let cli = Cli::parse_from([
      "dfaudit",
      "--output",
      "reports",
      "--image-name",
      "my-image",
    ]);

    assert_eq!(cli.output, PathBuf::from("reports"));

    assert_eq!(cli.image_name, "my-image");
  }

  #[test]
  fn verbose_flag() {
    let cli = Cli::parse_from(["dfaudit", "-vvv"]);

    assert_eq!(cli.verbose, 3);
  }

  #[test]
  fn quiet_flag() {
    let cli = Cli::parse_from(["dfaudit", "--quiet"]);

    assert!(cli.quiet);
  }
}
