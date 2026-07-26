use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Spinner {
  bar: Option<ProgressBar>,
}

impl Spinner {
  pub fn new(verbose: u8, message: &str) -> Self {
    if verbose > 1 {
      return Spinner {
        bar: None,
      };
    }

    let bar = ProgressBar::new_spinner();

    bar.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());

    bar.set_message(message.to_string());

    bar.enable_steady_tick(Duration::from_millis(100));

    Spinner {
      bar: Some(bar),
    }
  }

  pub fn finish(self) {
    if let Some(bar) = self.bar {
      bar.finish_and_clear();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn spinner_in_debug_mode() {
    let spinner = Spinner::new(2, "testing");

    assert!(spinner.bar.is_none());
  }

  #[test]
  fn spinner_in_normal_mode() {
    let spinner = Spinner::new(0, "testing");

    assert!(spinner.bar.is_some());
  }

  #[test]
  fn spinner_with_info_logging() {
    let spinner = Spinner::new(1, "testing");

    assert!(spinner.bar.is_some());
  }

  #[test]
  fn without_spinner() {
    let spinner = Spinner::new(2, "testing");

    spinner.finish();
  }

  #[test]
  fn with_spinner() {
    let spinner = Spinner::new(0, "testing");

    spinner.finish();
  }
}
