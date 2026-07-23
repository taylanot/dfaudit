use std::path::Path;

pub trait ContainerEngine {
    fn build(&self, file: &Path) -> Result<(), String>;

    fn run(&self, image: &str, command: &[&str]) -> Result<Vec<u8>, String>;

    fn remove(&self, image: &str) -> Result<(), String>;
}
