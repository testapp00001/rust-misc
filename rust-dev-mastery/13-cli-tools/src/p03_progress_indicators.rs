//! # Progress Indicators
//!
//! Long-running CLI operations need visual feedback. This lesson covers progress
//! bars, spinners, multi-progress displays, and custom templates — all implemented
//! from scratch to understand the underlying mechanics.
//!
//! ## Key Concepts
//! - Terminal escape codes for cursor control
//! - Progress bar rendering with ETA calculation
//! - Spinner animation frames
//! - Multi-progress with nested bars
//! - Custom progress templates
//! - Threading considerations for progress updates

use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Progress Bar
// ---------------------------------------------------------------------------

/// A progress bar that tracks completion of a task.
#[derive(Debug)]
pub struct ProgressBar {
    total: u64,
    current: u64,
    start: Instant,
    prefix: String,
    suffix: String,
    width: usize,
    _fill_char: char,
    _empty_char: char,
    style: ProgressStyle,
}

#[derive(Debug, Clone)]
pub enum ProgressStyle {
    Bar,
    Arrow,
    Hash,
    Block,
    Custom { fill: char, empty: char },
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: 0,
            start: Instant::now(),
            prefix: String::new(),
            suffix: String::new(),
            width: 40,
            _fill_char: '=',
            _empty_char: ' ',
            style: ProgressStyle::Bar,
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Advance the progress by `n` steps.
    pub fn inc(&mut self, n: u64) {
        self.current = (self.current + n).min(self.total);
    }

    /// Set the progress to an absolute value.
    pub fn set_position(&mut self, pos: u64) {
        self.current = pos.min(self.total);
    }

    /// Mark as complete.
    pub fn finish(&mut self) {
        self.current = self.total;
    }

    /// Get the current percentage (0.0 - 1.0).
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f64 / self.total as f64
        }
    }

    /// Get the elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Estimate the remaining time.
    pub fn eta(&self) -> Option<Duration> {
        if self.current == 0 || self.current >= self.total {
            return None;
        }
        let elapsed = self.elapsed().as_secs_f64();
        let rate = self.current as f64 / elapsed;
        let remaining = (self.total - self.current) as f64;
        Some(Duration::from_secs_f64(remaining / rate))
    }

    /// Calculate the current throughput (items per second).
    pub fn per_second(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.current as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Format the ETA as a human-readable string.
    pub fn format_eta(&self) -> String {
        match self.eta() {
            Some(eta) => {
                let secs = eta.as_secs();
                if secs < 60 {
                    format!("{secs}s")
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                }
            }
            None => "done".into(),
        }
    }

    /// Render the progress bar as a string.
    pub fn render(&self) -> String {
        let fraction = self.fraction();
        let filled = (fraction * self.width as f64) as usize;
        let empty = self.width - filled;

        let (fill, empty_char) = match &self.style {
            ProgressStyle::Bar => ('=', ' '),
            ProgressStyle::Arrow => ('=', '>'),
            ProgressStyle::Hash => ('#', ' '),
            ProgressStyle::Block => ('\u{2588}', '\u{2591}'),
            ProgressStyle::Custom { fill, empty } => (*fill, *empty),
        };

        let bar: String = format!(
            "{}{}",
            fill.to_string().repeat(filled),
            empty_char.to_string().repeat(empty)
        );

        let pct = fraction * 100.0;
        let rate = self.per_second();

        format!(
            "{} [{}] {}/{} ({:.1}%) {:.1}/s ETA: {}",
            self.prefix,
            bar,
            self.current,
            self.total,
            pct,
            rate,
            self.format_eta()
        )
    }

    pub fn is_finished(&self) -> bool {
        self.current >= self.total
    }

    pub fn current(&self) -> u64 {
        self.current
    }

    pub fn total(&self) -> u64 {
        self.total
    }
}

// ---------------------------------------------------------------------------
// 2. Spinner
// ---------------------------------------------------------------------------

/// A spinner for indeterminate progress.
#[derive(Debug)]
pub struct Spinner {
    frames: Vec<&'static str>,
    current_frame: usize,
    message: String,
    start: Instant,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            frames: vec!["|", "/", "-", "\\"],
            current_frame: 0,
            message: message.into(),
            start: Instant::now(),
        }
    }

    pub fn with_frames(mut self, frames: Vec<&'static str>) -> Self {
        self.frames = frames;
        self
    }

    pub fn dots() -> Self {
        Self::new("").with_frames(vec!["   ", ".  ", ".. ", "..."])
    }

    pub fn braille() -> Self {
        Self::new("").with_frames(vec![
            "\u{2801}", "\u{2802}", "\u{2804}", "\u{2840}",
            "\u{2880}", "\u{2820}", "\u{2810}", "\u{2808}",
        ])
    }

    /// Advance to the next frame and return the rendered string.
    pub fn tick(&mut self) -> String {
        let frame = self.frames[self.current_frame % self.frames.len()];
        self.current_frame += 1;
        if self.message.is_empty() {
            frame.to_string()
        } else {
            format!("{frame} {}", self.message)
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }
}

// ---------------------------------------------------------------------------
// 3. Multi-Progress
// ---------------------------------------------------------------------------

/// Manages multiple progress bars displayed simultaneously.
#[derive(Debug)]
pub struct MultiProgress {
    bars: Vec<ProgressBar>,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self { bars: Vec::new() }
    }

    /// Add a new progress bar and return its index.
    pub fn add(&mut self, bar: ProgressBar) -> usize {
        self.bars.push(bar);
        self.bars.len() - 1
    }

    /// Get a mutable reference to a bar by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ProgressBar> {
        self.bars.get_mut(index)
    }

    /// Render all progress bars.
    pub fn render_all(&self) -> Vec<String> {
        self.bars.iter().map(|bar| bar.render()).collect()
    }

    /// Check if all bars are finished.
    pub fn all_finished(&self) -> bool {
        self.bars.iter().all(|b| b.is_finished())
    }

    /// Get overall progress (average of all bars).
    pub fn overall_fraction(&self) -> f64 {
        if self.bars.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.bars.iter().map(|b| b.fraction()).sum();
        sum / self.bars.len() as f64
    }

    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }
}

// ---------------------------------------------------------------------------
// 4. Progress Template Engine
// ---------------------------------------------------------------------------

/// A simple template engine for custom progress bar formatting.
pub struct ProgressTemplate {
    template: String,
}

impl ProgressTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Render the template with the given progress bar state.
    pub fn render(&self, bar: &ProgressBar) -> String {
        let fraction = bar.fraction();
        let pct = (fraction * 100.0) as u32;
        let filled = (fraction * 40.0) as usize;
        let empty = 40 - filled;
        let bar_str = format!("{}{}", "=".repeat(filled), " ".repeat(empty));
        let eta = bar.format_eta();
        let elapsed = format_duration(bar.elapsed());

        self.template
            .replace("{bar}", &bar_str)
            .replace("{percent}", &pct.to_string())
            .replace("{current}", &bar.current().to_string())
            .replace("{total}", &bar.total().to_string())
            .replace("{eta}", &eta)
            .replace("{elapsed}", &elapsed)
            .replace("{per_sec}", &format!("{:.1}", bar.per_second()))
    }
}

/// Format a duration as a human-readable string.
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

// ---------------------------------------------------------------------------
// 5. Counter (Simple Progress)
// ---------------------------------------------------------------------------

/// A lightweight counter that renders as a simple line.
#[derive(Debug)]
pub struct Counter {
    total: u64,
    current: u64,
    label: String,
}

impl Counter {
    pub fn new(total: u64, label: impl Into<String>) -> Self {
        Self {
            total,
            current: 0,
            label: label.into(),
        }
    }

    pub fn inc(&mut self) {
        self.current = (self.current + 1).min(self.total);
    }

    pub fn render(&self) -> String {
        format!(
            "{}: {}/{} ({:.0}%)",
            self.label,
            self.current,
            self.total,
            self.fraction() * 100.0
        )
    }

    fn fraction(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f64 / self.total as f64
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_basic() {
        let mut bar = ProgressBar::new(100);
        assert_eq!(bar.current(), 0);
        assert_eq!(bar.total(), 100);
        assert!(!bar.is_finished());

        bar.inc(50);
        assert_eq!(bar.current(), 50);
        assert!(!bar.is_finished());

        bar.finish();
        assert!(bar.is_finished());
        assert_eq!(bar.fraction(), 1.0);
    }

    #[test]
    fn test_progress_bar_fraction() {
        let mut bar = ProgressBar::new(200);
        assert_eq!(bar.fraction(), 0.0);

        bar.inc(100);
        assert!((bar.fraction() - 0.5).abs() < f64::EPSILON);

        bar.inc(50);
        assert!((bar.fraction() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let bar = ProgressBar::new(0);
        assert_eq!(bar.fraction(), 1.0);
        assert!(bar.is_finished());
    }

    #[test]
    fn test_progress_bar_set_position() {
        let mut bar = ProgressBar::new(100);
        bar.set_position(75);
        assert_eq!(bar.current(), 75);

        // Cannot exceed total
        bar.set_position(200);
        assert_eq!(bar.current(), 100);
    }

    #[test]
    fn test_progress_bar_render() {
        let mut bar = ProgressBar::new(100).with_prefix("Downloading");
        bar.inc(50);
        let rendered = bar.render();
        assert!(rendered.contains("Downloading"));
        assert!(rendered.contains("50/100"));
        assert!(rendered.contains("50.0%"));
    }

    #[test]
    fn test_progress_bar_styles() {
        let mut bar = ProgressBar::new(10).with_style(ProgressStyle::Arrow);
        bar.inc(5);
        let rendered = bar.render();
        assert!(rendered.contains('>'));

        let mut bar = ProgressBar::new(10).with_style(ProgressStyle::Hash);
        bar.inc(5);
        let rendered = bar.render();
        assert!(rendered.contains('#'));
    }

    #[test]
    fn test_progress_bar_eta_none_when_zero() {
        let bar = ProgressBar::new(100);
        assert!(bar.eta().is_none());
    }

    #[test]
    fn test_progress_bar_format_eta_done() {
        let mut bar = ProgressBar::new(100);
        bar.finish();
        assert_eq!(bar.format_eta(), "done");
    }

    #[test]
    fn test_spinner_basic() {
        let mut spinner = Spinner::new("Loading");
        let frame1 = spinner.tick();
        assert!(frame1.contains("|"));
        assert!(frame1.contains("Loading"));

        let frame2 = spinner.tick();
        assert!(frame2.contains("/"));

        let frame3 = spinner.tick();
        assert!(frame3.contains("-"));
    }

    #[test]
    fn test_spinner_dots() {
        let mut spinner = Spinner::dots();
        assert_eq!(spinner.tick(), "   ");
        assert_eq!(spinner.tick(), ".  ");
        assert_eq!(spinner.tick(), ".. ");
        assert_eq!(spinner.tick(), "...");
        assert_eq!(spinner.tick(), "   "); // cycles
    }

    #[test]
    fn test_spinner_set_message() {
        let mut spinner = Spinner::new("start");
        assert!(spinner.tick().contains("start"));

        spinner.set_message("end");
        assert!(spinner.tick().contains("end"));
    }

    #[test]
    fn test_multi_progress() {
        let mut multi = MultiProgress::new();
        let idx1 = multi.add(ProgressBar::new(100));
        let idx2 = multi.add(ProgressBar::new(50));

        multi.get_mut(idx1).unwrap().inc(50);
        multi.get_mut(idx2).unwrap().inc(25);

        let rendered = multi.render_all();
        assert_eq!(rendered.len(), 2);
        assert!(!multi.all_finished());

        let overall = multi.overall_fraction();
        assert!((overall - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multi_progress_all_finished() {
        let mut multi = MultiProgress::new();
        let idx1 = multi.add(ProgressBar::new(10));
        let idx2 = multi.add(ProgressBar::new(20));

        multi.get_mut(idx1).unwrap().finish();
        multi.get_mut(idx2).unwrap().finish();

        assert!(multi.all_finished());
        assert!((multi.overall_fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multi_progress_empty() {
        let multi = MultiProgress::new();
        assert!(multi.all_finished());
        assert_eq!(multi.overall_fraction(), 0.0);
        assert_eq!(multi.bar_count(), 0);
    }

    #[test]
    fn test_progress_template() {
        let template = ProgressTemplate::new("{prefix} {bar} {percent}%");
        let mut bar = ProgressBar::new(100).with_prefix("Test");
        bar.inc(50);

        // Template doesn't have {prefix} placeholder in our simple impl,
        // but it handles {bar} and {percent}
        let t2 = ProgressTemplate::new("[{bar}] {percent}%");
        let rendered = t2.render(&bar);
        assert!(rendered.contains("50%"));
        assert!(rendered.contains("["));
    }

    #[test]
    fn test_progress_template_all_placeholders() {
        let template =
            ProgressTemplate::new("{bar} {percent}% {current}/{total} ETA:{eta} elapsed:{elapsed}");
        let mut bar = ProgressBar::new(100);
        bar.inc(42);
        let rendered = template.render(&bar);
        assert!(rendered.contains("42%"));
        assert!(rendered.contains("42/100"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_counter() {
        let mut counter = Counter::new(10, "items");
        assert_eq!(counter.render(), "items: 0/10 (0%)");

        counter.inc();
        counter.inc();
        counter.inc();
        assert_eq!(counter.render(), "items: 3/10 (30%)");
    }

    #[test]
    fn test_progress_bar_per_second() {
        let mut bar = ProgressBar::new(100);
        bar.inc(10);
        std::thread::sleep(Duration::from_millis(100));
        let rate = bar.per_second();
        assert!(rate > 0.0);
    }

    #[test]
    fn test_progress_bar_width() {
        let mut bar = ProgressBar::new(10).with_width(20);
        bar.inc(5);
        let rendered = bar.render();
        // Width of 20 should be visible in the bar section
        assert!(rendered.contains("["));
        assert!(rendered.contains("]"));
    }
}
