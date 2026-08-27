# filterkit

A Rust library for composing typed DSP filters and signal processors.

```toml
[dependencies]
filterkit = "0.1"
```

## Who it is for

For Rust projects that need a filter, not a DSP framework. It puts common
filters behind one interface and handles their state and composition, so you
can swap filters without rewiring the rest of the pipeline. It is a
general-purpose starting point, not a fully customizable or peak-throughput
audio library.

## Core model

Design specs produce coefficients or models; processors consume those values
and maintain their own runtime state. This keeps optional design and analysis
code separate from the processing core, while allowing prepared filters to be
reused.

Several processing traits cover common filter and signal-processing patterns:

- [`SampleProcessor`](src/traits/sample.rs): one input sample produces one output sample.
- [`BlockProcessor`](src/traits/block.rs): block-based processing.
- [`StreamProcessor`](src/traits/stream.rs): variable-rate input and output.
- [`WholeSignalProcessor`](src/traits/whole.rs): processing over a complete finite signal.
- [`AdaptiveProcessor`](src/traits/mod.rs): processors whose coefficients update from the signal.

`ProcessorExt` provides composition helpers such as `.then(...)`.

## Included

- FIR and IIR filters, including biquads, SOS cascades, direct forms, gain,
  delay, EMA, and state-space processors.
- Combinators for chains, parallel branches, sums, mapping, taps, bypass, and
  wet/dry mixing.
- Decimators, interpolators, polyphase resampling, and forward/backward
  whole-signal filtering.
- Design helpers for moving averages, exponential averages, windowed-sinc
  FIRs, and RBJ biquads.
- Optional LMS/NLMS adaptive filters and a linear Kalman filter.

The `design` feature is enabled by default. `kalman` is opt-in. Disable
default features for the `no_std`, no-alloc configuration.

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

Composition uses the same sample-processing interface:

```rust
use filterkit::{ProcessorExt, SampleProcessor, processors::{Biquad, Gain}};
use filterkit::design::BiquadLowpassSpec;

let lowpass = Biquad::new(BiquadLowpassSpec { f0: 0.10, q: 0.707 }.design().unwrap());
let mut chain = lowpass.then(Gain::new(0.7_f64));

let y = chain.process_sample(0.5);
```

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or
the [MIT license](LICENSE-MIT) at your option.
