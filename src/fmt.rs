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
                    format!("{:.9}ms", duration_millis_f64(d))
                } else if nanos >= 1_000 {
                    format!("{:.9}µs", duration_micros_f64(d))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TimeFormat;

    #[test]
    fn test_duration_millis_f64() {
        assert_eq!(duration_millis_f64(Duration::new(0, 42)), 0.000042);
        assert_eq!(duration_millis_f64(Duration::new(1, 42)), 1000.000042);
    }

    #[test]
    fn test_duration_micros_f64() {
        assert_eq!(duration_micros_f64(Duration::new(0, 42)), 0.042);
        assert_eq!(duration_micros_f64(Duration::new(1, 42)), 1000000.042);
    }

    #[test]
    fn test_duration_nanos_u64() {
        assert_eq!(duration_nanos_u64(Duration::new(0, 42)), 42);
        assert_eq!(duration_nanos_u64(Duration::new(1, 42)), 1000000042);
    }

    #[test]
    fn test_duration_formatter_none() {
        let fmt = duration_formatter(None);
        assert_eq!(fmt(Duration::new(0, 42)), "42ns");
        assert_eq!(fmt(Duration::new(0, 42000)), "42.000000000µs");
        assert_eq!(fmt(Duration::new(0, 42000000)), "42.000000000ms");
        assert_eq!(fmt(Duration::new(42, 0)), "42.000000000s ");
    }

    #[test]
    fn test_duration_formatter_normal() {
        let fmt = duration_formatter(Some(TimeFormat::Normal));
        assert_eq!(fmt(Duration::new(0, 42)), "42ns");
        assert_eq!(fmt(Duration::new(0, 42000)), "42.000000000µs");
        assert_eq!(fmt(Duration::new(0, 42000000)), "42.000000000ms");
        assert_eq!(fmt(Duration::new(42, 0)), "42.000000000s ");
    }

    #[test]
    fn test_duration_formatter_seconds() {
        let fmt = duration_formatter(Some(TimeFormat::Seconds));
        assert_eq!(fmt(Duration::new(0, 42)), "0.000000042s");
        assert_eq!(fmt(Duration::new(1, 42)), "1.000000042s");
    }

    #[test]
    fn test_duration_formatter_milli() {
        let fmt = duration_formatter(Some(TimeFormat::Milli));
        assert_eq!(fmt(Duration::new(0, 42)), "0.000042ms");
        assert_eq!(fmt(Duration::new(1, 42)), "1000.000042ms");
    }

    #[test]
    fn test_duration_formatter_micro() {
        let fmt = duration_formatter(Some(TimeFormat::Micro));
        assert_eq!(fmt(Duration::new(0, 42)), "0.042µs");
        assert_eq!(fmt(Duration::new(1, 42)), "1000000.042µs");
    }

    #[test]
    fn test_duration_formatter_nano() {
        let fmt = duration_formatter(Some(TimeFormat::Nano));
        assert_eq!(fmt(Duration::new(0, 42)), "42ns");
        assert_eq!(fmt(Duration::new(1, 42)), "1000000042ns");
    }
}