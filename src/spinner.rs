use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::time::Duration;

const TICK_INTERVAL: Duration = Duration::from_millis(80);

pub struct Spinner {
    bar: ProgressBar,
}

impl Spinner {
    pub fn start(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_draw_target(ProgressDrawTarget::stderr());
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .expect("template should be valid"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(TICK_INTERVAL);
        Self { bar }
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish(self) {
        self.bar.finish_and_clear();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_starts_and_finishes_without_panic() {
        let spinner = Spinner::start("test message");
        spinner.set_message("updated message");
        spinner.finish();
    }
}
