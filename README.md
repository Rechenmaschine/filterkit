# filterkit

`filterkit` is a collection of signal-processing primitives for Rust, including common filters, delays, combinators, and related utilities.

This crate is aimed at general-purpose use and favors a small, consistent API over specialized implementations for individual domains.

`filterkit` is not intended as a high-performance audio DSP library or as a replacement for specialized signal-processing crates. The focus is on common cases that come up in everyday use.

> [!IMPORTANT]
> The API is still work in progress and may change.

## Quick start

For sample-by-sample processing:

```rust
use filterkit::{processors::{Delay, Ema}, SampleProcessor};

let mut smoother = Ema::new(0.1_f32);
let mut delay = Delay::<f32, 4>::new();

let sample = 1.0_f32;
let smoothed = smoother.process_sample(sample);
let delayed = delay.process_sample(sample);
```

## Included

- FIR and IIR filters, biquads, SOS cascades, direct forms, gain, delay, EMA,
  first-order, and state-space processors.
- Combinators for combining processors.
- Design helpers, whole-signal filtering, adaptive filters, and an optional
  linear Kalman filter.
- `no_std` and no-alloc configurations.

The `design` feature is enabled by default; `kalman` is opt-in.

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or
the [MIT license](LICENSE-MIT) at your option.
