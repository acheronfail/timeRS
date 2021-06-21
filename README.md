# timeRS

Like GNU `time` but in Rust!

Some features:

* better precision than the built-in `time`
* configurable unit outputs (seconds, milliseconds, microseconds, etc)
* returns the exit code of the timed process, or the signal (if it was stopped via a signal)

## Usage

See help

```bash
$ timers --help
```

Basically, just use this as you would use the `time` built-in that's included in most shells.

## Installation

Install with `cargo`:

```bash
$ cargo install timers
```
