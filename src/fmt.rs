use std::time::Duration;
use crate::cli::TimeFormat;

/// Returns the milliseconds, with nanoseconds contained in an `f64`.
fn duration_millis_f64(d: Duration) -> f64 {
    (d.as_secs() * 1_000) as f64 + (d.subsec_nanos() as f64 / 1_000_000.0)
}

/// Returns the microseconds, with nanoseconds contained in an `f64`.
fn duration_micros_f64(d: Duration) -> f64 {
    (d.as_secs() * 1_000_000) as f64 + (d.subsec_nanos() as f64 / 1_000.0)
}

/// The lowest form of measurement, as such it does not return an `f64` but a `u64`.
fn duration_nanos_u64(d: Duration) -> u64 {
    (d.as_secs() * 1_000_000_000) + (d.subsec_nanos() as u64)
}

/// Returns a formatter closure for the given `TimeFormat`.
pub fn duration_formatter(time_format: Option<TimeFormat>) -> impl Fn(Duration) -> String {
    match time_format {
        None | Some(TimeFormat::Normal) => |d: Duration| {
            if d.as_secs() > 0 {
                format!("{:.9}s ", d.as_secs_f64())
            } else {
                let nanos = d.subsec_nanos();
                if nanos >= 1_000_000 {
                    format!("{:.6}ms", duration_millis_f64(d))
                } else if nanos >= 1_000 {
                    format!("{:.3}µs", duration_micros_f64(d))
                } else {
                    format!("{}ns", duration_nanos_u64(d))
                }
            }
        },
        Some(TimeFormat::Seconds) => |d: Duration| format!("{:.9}s", d.as_secs_f64()),
        Some(TimeFormat::Milli) => |d: Duration| format!("{:.6}ms", duration_millis_f64(d)),
        Some(TimeFormat::Micro) => |d: Duration| format!("{:.3}µs", duration_micros_f64(d)),
        Some(TimeFormat::Nano) => |d: Duration| format!("{}ns", duration_nanos_u64(d)),
    }
}