use std::io::Write;

use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressMsg {}

impl Drop for ProgressMsg {
    fn drop(&mut self) {
        println!(" done.");
    }
}

pub fn msg(msg: &str) -> ProgressMsg {
    print!("{}...", msg);
    std::io::stdout().flush().unwrap();
    ProgressMsg {}
}

pub trait MaxProgress {
    fn max_progress(self) -> u64;
}

impl MaxProgress for u64 {
    fn max_progress(self) -> u64 {
        self
    }
}

impl MaxProgress for usize {
    fn max_progress(self) -> u64 {
        self as u64
    }
}

impl MaxProgress for &std::fs::File {
    fn max_progress(self) -> u64 {
        self.metadata().unwrap().len()
    }
}

pub fn progress_bar(max: impl MaxProgress) -> ProgressBar {
    let progress = ProgressBar::new(max.max_progress());
    progress.set_style(
        ProgressStyle::default_bar()
            .template(" [{elapsed_precise}] [{bar:40.cyan/blue}] {percent:3}% ({eta} left)")
            .progress_chars("#>-"),
    );
    progress
}
