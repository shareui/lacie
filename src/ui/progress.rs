use crate::diag::DiagError;
use crate::ui::color::{visible_len, Styler};
use crate::ui::cursor::CursorGuard;
use std::io::{stdout, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub fn get_terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

pub struct ProgressMetrics {
    pub fraction: Option<f64>,
    pub current_mb: f64,
    pub total_mb: Option<f64>,
    pub speed_mb: f64,
}

pub fn make_metrics(current_bytes: u64, total_bytes: Option<u64>, speed_mb: f64) -> ProgressMetrics {
    let current_mb = current_bytes as f64 / 1_048_576.0;
    if let Some(total) = total_bytes {
        let total_mb = total as f64 / 1_048_576.0;
        let fraction = if total > 0 {
            (current_bytes as f64 / total as f64).min(1.0)
        } else {
            1.0
        };
        ProgressMetrics {
            fraction: Some(fraction),
            current_mb,
            total_mb: Some(total_mb),
            speed_mb,
        }
    } else {
        ProgressMetrics {
            fraction: None,
            current_mb,
            total_mb: None,
            speed_mb,
        }
    }
}

fn build_indeterminate_bar(bar_width: usize, tick: usize, is_done: bool) -> String {
    if is_done {
        return std::format!(" [{}]", "#".repeat(bar_width));
    }
    let block_len = 5.min(bar_width);
    let cycle = (bar_width.saturating_sub(block_len)) * 2;
    let pos = if cycle == 0 {
        0
    } else {
        let t = tick % cycle;
        if t <= bar_width - block_len {
            t
        } else {
            cycle - t
        }
    };
    let left = pos;
    let right = bar_width - left - block_len;
    std::format!(" [{}{}{}]", " ".repeat(left), "#".repeat(block_len), " ".repeat(right))
}

pub fn build_progress_line(
    prefix: &str,
    metrics: &ProgressMetrics,
    min_prefix_width: usize,
    tick: usize,
    is_done: bool,
    styler: &Styler,
) -> String {
    let term_width = get_terminal_width();
    let vis_prefix = visible_len(prefix);
    let mut padded_prefix = if vis_prefix < min_prefix_width {
        let diff = min_prefix_width - vis_prefix;
        std::format!("{}{}", prefix, " ".repeat(diff))
    } else {
        prefix.to_string()
    };

    let avail = term_width.saturating_sub(min_prefix_width.max(vis_prefix) + 1).max(5);
    let speed_str = std::format!("  {:.1} MB/s", metrics.speed_mb);

    if let (Some(fraction), Some(total_mb)) = (metrics.fraction, metrics.total_mb) {
        let size_str = std::format!("  {:.2}/{:.2} MB", metrics.current_mb, total_mb);
        let pct_str = std::format!(" {:3}%", (fraction * 100.0) as u32);
        let stats_len = speed_str.len() + size_str.len() + pct_str.len();
        let bar_space = avail.saturating_sub(stats_len + 2);

        if bar_space >= 6 {
            let bar_width = if min_prefix_width > 0 { bar_space.min(50) } else { bar_space };
            let filled = if is_done { bar_width } else { ((fraction * bar_width as f64) as usize).min(bar_width) };
            let bar_str = std::format!(" [{}{}]", "#".repeat(filled), " ".repeat(bar_width - filled));
            let gap = avail.saturating_sub(bar_str.len() + stats_len);
            if gap > 0 {
                padded_prefix.push_str(&" ".repeat(gap));
            }
            std::format!("{}{}{}{}{}", padded_prefix, bar_str, pct_str, size_str, styler.dim(&speed_str))
        } else {
            let (speed_out, size_out, pct_out) = truncate_stats(avail, &speed_str, &size_str, &pct_str);
            let sub_len = speed_out.len() + size_out.len() + pct_out.len();
            let gap = avail.saturating_sub(sub_len);
            if gap > 0 {
                padded_prefix.push_str(&" ".repeat(gap));
            }
            std::format!("{}{}{}{}", padded_prefix, pct_out, size_out, styler.dim(speed_out))
        }
    } else {
        let size_str = std::format!("  {:.2} MB", metrics.current_mb);
        let stats_len = speed_str.len() + size_str.len();
        let bar_space = avail.saturating_sub(stats_len + 2);
        if bar_space >= 6 {
            let bar_width = if min_prefix_width > 0 { bar_space.min(50) } else { bar_space };
            let bar_str = build_indeterminate_bar(bar_width, tick, is_done);
            let gap = avail.saturating_sub(bar_str.len() + stats_len);
            if gap > 0 {
                padded_prefix.push_str(&" ".repeat(gap));
            }
            std::format!("{}{}{}{}", padded_prefix, bar_str, size_str, styler.dim(&speed_str))
        } else {
            let (speed_out, size_out) = truncate_stats_no_total(avail, &speed_str, &size_str);
            let sub_len = speed_out.len() + size_out.len();
            let gap = avail.saturating_sub(sub_len);
            if gap > 0 {
                padded_prefix.push_str(&" ".repeat(gap));
            }
            std::format!("{}{}{}", padded_prefix, size_out, styler.dim(speed_out))
        }
    }
}

fn truncate_stats<'a>(avail: usize, speed: &'a str, size: &'a str, pct: &'a str) -> (&'a str, &'a str, &'a str) {
    let mut s = speed;
    let mut sz = size;
    let mut p = pct;
    if avail < s.len() + sz.len() + p.len() {
        s = "";
    }
    if avail < sz.len() + p.len() {
        sz = "";
    }
    if avail < p.len() {
        p = "";
    }
    (s, sz, p)
}

fn truncate_stats_no_total<'a>(avail: usize, speed: &'a str, size: &'a str) -> (&'a str, &'a str) {
    let mut s = speed;
    let mut sz = size;
    if avail < s.len() + sz.len() {
        s = "";
    }
    if avail < sz.len() {
        sz = "";
    }
    (s, sz)
}

pub fn run_with_download_progress<F>(
    prefix: &str,
    total_bytes: Option<u64>,
    dest_path: &Path,
    min_prefix_width: usize,
    styler: &Styler,
    download_fn: F,
) -> Result<(), DiagError>
where
    F: FnOnce() -> Result<(), DiagError> + Send,
{
    let _guard = CursorGuard::new();
    let done = std::sync::atomic::AtomicBool::new(false);
    let done_ref = &done;
    let mut download_res: Result<(), DiagError> = Ok(());
    let start_time = std::time::Instant::now();
    let mut speed_mb = 14.2;

    std::thread::scope(|s| {
        let worker = s.spawn(|| {
            let res = download_fn();
            done_ref.store(true, std::sync::atomic::Ordering::SeqCst);
            res
        });

        let mut last_bytes = 0u64;
        let mut last_time = std::time::Instant::now();
        let mut tick = 0usize;

        while !done_ref.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(30));
            tick += 1;
            let current_bytes = std::fs::metadata(dest_path).map(|m| m.len()).unwrap_or(0);
            let elapsed = last_time.elapsed().as_secs_f64();
            if elapsed >= 0.15 {
                let bytes_diff = current_bytes.saturating_sub(last_bytes);
                speed_mb = (bytes_diff as f64 / 1_048_576.0) / elapsed;
                last_bytes = current_bytes;
                last_time = std::time::Instant::now();
            }
            let metrics = make_metrics(current_bytes, total_bytes, speed_mb);
            let line = build_progress_line(prefix, &metrics, min_prefix_width, tick, false, styler);
            print!("\r{}\x1b[K", line);
            let _ = stdout().flush();
        }

        download_res = worker.join().unwrap();
    });

    download_res?;
    let final_bytes = std::fs::metadata(dest_path).map(|m| m.len()).unwrap_or(0);
    let total_elapsed = start_time.elapsed().as_secs_f64();
    if total_elapsed > 0.001 {
        let measured = (final_bytes as f64 / 1_048_576.0) / total_elapsed;
        if measured > 0.1 {
            speed_mb = measured;
        }
    }
    if speed_mb <= 0.0 {
        speed_mb = 14.2;
    }

    let final_current = if let Some(tot) = total_bytes {
        tot.max(final_bytes)
    } else {
        final_bytes
    };
    let metrics = make_metrics(final_current, total_bytes, speed_mb);
    let line = build_progress_line(prefix, &metrics, min_prefix_width, 0, true, styler);
    print!("\r{}\x1b[K\n", line);
    let _ = stdout().flush();
    Ok(())
}

pub fn render_progress_bar(
    prefix: &str,
    total_mb: f64,
    min_prefix_width: usize,
    styler: &Styler,
) {
    let _guard = CursorGuard::new();
    let steps = 25;
    for i in 0..=steps {
        let fraction = (i as f64) / (steps as f64);
        let current_mb = total_mb * fraction;
        let metrics = ProgressMetrics {
            fraction: Some(fraction),
            current_mb,
            total_mb: Some(total_mb),
            speed_mb: 14.2,
        };
        let is_done = i == steps;
        let line = build_progress_line(
            prefix,
            &metrics,
            min_prefix_width,
            i,
            is_done,
            styler,
        );
        print!("\r{}\x1b[K", line);
        let _ = stdout().flush();
        sleep(Duration::from_millis(15));
    }
    println!();
}

pub fn render_spinner(text: &str, duration_ms: u64, styler: &Styler) {
    let _guard = CursorGuard::new();
    let frames = ["-", "\\", "|", "/"];
    let iterations = (duration_ms / 60).max(1);
    for i in 0..iterations {
        let frame = frames[(i as usize) % frames.len()];
        print!("\r  {} {}", styler.dim(frame), text);
        let _ = stdout().flush();
        sleep(Duration::from_millis(60));
    }
    print!("\r\x1b[K");
    let _ = stdout().flush();
}
