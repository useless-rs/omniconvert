//! Parser for ffmpeg `-progress pipe:1` machine output plus `Duration:`
//! stderr headers. Pure functions so the protocol is unit-testable without
//! running ffmpeg.

/// Microseconds.
pub type Micros = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressEvent {
    Time(Micros),
    End,
}

/// Parse one `-progress` line. Returns an event only for keys we act on.
pub fn parse_progress_line(line: &str) -> Option<ProgressEvent> {
    let (k, v) = line.split_once('=')?;
    match k.trim() {
        "out_time_us" => v.trim().parse::<i64>().ok().map(|us| {
            ProgressEvent::Time(us.max(0) as Micros)
        }),
        "progress" if v.trim() == "end" => Some(ProgressEvent::End),
        _ => None,
    }
}

/// Parse ffmpeg time expressions: `HH:MM:SS[.frac]`, `MM:SS[.frac]`,
/// `SS[.frac]`, or raw `123456us` / `200ms` / `5s`.
pub fn parse_time(s: &str) -> Option<Micros> {
    let s = s.trim();
    if s.is_empty() || s == "N/A" {
        return None;
    }
    if let Some(num) = s.strip_suffix("us") {
        return num.parse::<f64>().ok().map(|v| v as Micros);
    }
    if let Some(num) = s.strip_suffix("ms") {
        return num.parse::<f64>().ok().map(|v| (v * 1000.0) as Micros);
    }
    if let Some(num) = s.strip_suffix('s') {
        if !num.contains(':') {
            return num.parse::<f64>().ok().map(|v| (v * 1_000_000.0) as Micros);
        }
    }
    let mut total = 0.0;
    let mut factor = 1.0;
    for part in s.split(':').rev() {
        total += part.parse::<f64>().ok()? * factor;
        factor *= 60.0;
    }
    Some((total * 1_000_000.0) as Micros)
}

/// Scan ffmpeg stderr text for the first `Duration: ...` header.
pub fn parse_duration(stderr: &str) -> Option<Micros> {
    for line in stderr.lines() {
        if let Some(rest) = line.split("Duration:").nth(1) {
            let token = rest.split(',').next().unwrap_or("").trim();
            if let Some(us) = parse_time(token) {
                if us > 0 {
                    return Some(us);
                }
            }
        }
    }
    None
}

/// Whole-percent 0..=100, or None when the total is unknown/zero.
pub fn percent(done_us: Micros, total_us: Micros) -> Option<u32> {
    if total_us == 0 {
        return None;
    }
    Some(((done_us as f64 / total_us as f64) * 100.0).floor() as u32)
}

pub fn fmt_hms(us: Micros) -> String {
    let s = us / 1_000_000;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_lines() {
        assert_eq!(parse_progress_line("out_time_us=1234567"), Some(ProgressEvent::Time(1234567)));
        assert_eq!(parse_progress_line("out_time_us=-233"), Some(ProgressEvent::Time(0)));
        assert_eq!(parse_progress_line("progress=end"), Some(ProgressEvent::End));
        assert_eq!(parse_progress_line("progress=continue"), None);
        assert_eq!(parse_progress_line("frame=123"), None);
        assert_eq!(parse_progress_line("speed=2.5x"), None);
        assert_eq!(parse_progress_line("garbage"), None);
    }

    #[test]
    fn times() {
        assert_eq!(parse_time("00:00:10.00"), Some(10_000_000));
        assert_eq!(parse_time("01:02:03.5"), Some(3723_500_000));
        assert_eq!(parse_time("23.189"), Some(23_189_000));
        assert_eq!(parse_time("200ms"), Some(200_000));
        assert_eq!(parse_time("200000us"), Some(200_000));
        assert_eq!(parse_time("N/A"), None);
        assert_eq!(parse_time(""), None);
    }

    #[test]
    fn duration_headers() {
        let stderr = "  Duration: 00:00:10.00, start: 0.000000, bitrate: 1 kb/s\n  Stream #0:0: Video";
        assert_eq!(parse_duration(stderr), Some(10_000_000));
        assert_eq!(parse_duration("no duration here"), None);
        assert_eq!(parse_duration("  Duration: N/A, start: 0"), None);
    }

    #[test]
    fn percents() {
        assert_eq!(percent(5_000_000, 10_000_000), Some(50));
        assert_eq!(percent(0, 10_000_000), Some(0));
        assert_eq!(percent(10_000_000, 10_000_000), Some(100));
        assert_eq!(percent(5, 0), None);
    }
}
