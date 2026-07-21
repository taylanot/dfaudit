use clap::Parser;
use std::process::Command;

/// Get the data 
#[derive(Parser)]
struct Cli {
  #[arg(short = 'f', long = "file")]
  file: String,

  #[arg(short = 't', long = "tag", default_value = "Dockerfile")]
  tag: String,

  #[arg(default_value = ".")]
  context: String,
}


/// Build a container image with podman
fn build_image(cli: &Cli) {

  let build_status = Command::new("podman")
      .args(["build", "-t", "temp-image", "-f", &cli.file, &cli.context])
      .status()
      .expect("failed to run podman");

  if !build_status.success() {
      eprintln!("podman build failed with status: {build_status}");
      std::process::exit(1);
  }

  println!("Build succeeded! Image tagged as '{}'", cli.tag);

  println!("Now deleting the temporary image! Image tagged as temp-image");
  
  let remove_status= Command::new("podman")
      .args(["rmi", "temp-image"])
      .status()
      .expect("failed to run podman rmi");

  if !remove_status.success() {
      eprintln!("podman rmi failed with status: {remove_status}");
      std::process::exit(1);
  }

}

/// Main function that will: 
/// 1. read the cli
/// 2. search docker/container-files
/// 3. build images
/// 4. audit packages
/// 5. output an html file
fn main(){
  let cli = Cli::parse();
  
  build_image(&cli) 
}
