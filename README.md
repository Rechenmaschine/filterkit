# filterkit

A Rust library for composing typed DSP filters and signal processors.

```toml
[dependencies]
filterkit = "0.1"
```

## Who it is for

For Rust applications that process sampled signals and need reusable
processors with explicit runtime state. The crate can be used with a
`no_std`, no-alloc configuration or with heap-backed processors through the
`alloc` feature.

## Core model

Filter representations and runtime state are separate. Processing is grouped
by shape:

- `SampleProcessor`: one input sample produces one output sample.
- `BlockProcessor`: block-based processing.
- `StreamProcessor`: variable-rate input and output.
- `WholeSignalProcessor`: processing over a complete finite signal.
- `AdaptiveProcessor`: processors whose coefficients update from the signal.

Included components cover FIR and IIR filters, filter combinators, decimators
and resamplers, whole-signal forward/backward filtering, adaptive LMS/NLMS,
and an optional linear Kalman filter. Design helpers cover moving averages,
exponential averages, windowed-sinc FIRs, and RBJ biquads.

The `design` feature is enabled by default. `kalman` is opt-in and pulls in
`nalgebra`; `alloc` enables heap-backed processors. Disable default features
for the `no_std`, no-alloc configuration.

## Quick start

```rust
use filterkit::{SampleProcessor, processors::Biquad};
use filterkit::design::BiquadLowpassSpec;

let coeffs = BiquadLowpassSpec { f0: 2_000.0 / 48_000.0, q: 0.707 }
    .design()
    .unwrap();
let mut filter = Biquad::new(coeffs);

let y = filter.process_sample(1.0_f64);
```

Processors can be composed with `ProcessorExt`:

```rust
use filterkit::{ProcessorExt, SampleProcessor, processors::{Biquad, Gain}};
use filterkit::design::BiquadLowpassSpec;

let lp = Biquad::new(BiquadLowpassSpec { f0: 0.10, q: 0.707 }.design().unwrap());
let mut chain = lp.then(Gain::new(0.7_f64));

let y = chain.process_sample(0.5);
```

## Examples

Run an example with:

```text
cargo run --example biquad_lowpass
cargo run --example combinator_chain
cargo run --example zero_phase_filtfilt
cargo run --example kalman_tracking --features kalman
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
