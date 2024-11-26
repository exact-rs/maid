use indicatif::{ProgressBar, ProgressStyle};
use macros_rs::fmt::fmtstr;
use std::sync::OnceLock;

static PROGRESS_BAR: OnceLock<ProgressBar> = OnceLock::new();

pub(crate) fn get<'p>() -> Option<&'p ProgressBar> { PROGRESS_BAR.get() }

pub(crate) fn init<'p>(ticks: Vec<&str>, template: &str, tick: u64) -> &'p ProgressBar {
    PROGRESS_BAR.get_or_init(|| {
        let pb = ProgressBar::new_spinner();
        let tick_str: Vec<&str> = ticks.into_iter().map(|item| fmtstr!("{item} ")).collect();

        pb.enable_steady_tick(std::time::Duration::from_millis(tick));
        pb.set_style(ProgressStyle::with_template(template).unwrap().tick_strings(&*tick_str));

        return pb;
    })
}

pub(crate) fn finish() {
    if let Some(pb) = PROGRESS_BAR.get() {
        pb.finish_and_clear();
    }
}
