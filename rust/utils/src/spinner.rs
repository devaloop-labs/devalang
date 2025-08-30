#[cfg(feature = "cli")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "cli")]
use std::time::Duration;

#[cfg(feature = "cli")]
pub fn start_spinner(start_msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(start_msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));

    std::thread::sleep(Duration::from_millis(750));

    spinner
}
