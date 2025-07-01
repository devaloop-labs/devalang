use indicatif::{ ProgressBar, ProgressStyle };
use std::{ time::Duration };

pub fn with_spinner<T, F>(start_msg: &str, f: F) -> T where F: FnOnce() -> T {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    spinner.set_message(start_msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));

    let result = f();

    spinner.finish_and_clear();

    result
}
