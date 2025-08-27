#[cfg(feature = "cli")]
use indicatif::{ ProgressBar, ProgressStyle };
use std::{ time::Duration };

#[cfg(feature = "cli")]
pub fn with_spinner<T, F>(start_msg: &str, f: F) -> ProgressBar where F: FnOnce() -> T {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    spinner.set_message(start_msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));

    let _ = f();

    spinner
}
