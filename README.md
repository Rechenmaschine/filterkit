# filterkit

A Rust library for composing typed DSP filters and signal processors.

```toml
[dependencies]
filterkit = "0.1"
```

## Who it is for

For Rust applications that need a practical, general-purpose DSP abstraction.
Use it when you would otherwise write the filter interface, state handling,
and composition yourself, or when you want to swap filters without changing
the rest of the pipeline. It covers common cases rather than targeting peak
throughput and full customization.

## Core model

Filter representations and runtime state are separate. This lets coefficient
data be shared while each processor keeps its own delay lines and state.

Processing is grouped by shape:

- `SampleProcessor`: one input sample produces one output sample.
- `BlockProcessor`: block-based processing.
- `StreamProcessor`: variable-rate input and output.
- `WholeSignalProcessor`: processing over a complete finite signal.
- `AdaptiveProcessor`: processors whose coefficients update from the signal.

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
