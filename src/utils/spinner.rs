use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};

pub fn with_spinner(start_msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner:.green} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
    spinner.set_style(style);
    spinner.set_message(start_msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));

    thread::sleep(Duration::from_millis(800));

    spinner
}
