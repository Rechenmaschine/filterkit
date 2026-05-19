# filterkit

Composable DSP filters and processors for embedded and desktop Rust.

```toml
[dependencies]
filterkit = "0.1"
```

## What it is

A small, shape-oriented DSP toolkit organised around three ideas:

- **Representation** describes the system (FIR taps, biquad coefficients,
  SOS cascades, transfer functions, state-space models).
- **State** stores the runtime memory, kept *separate* from the
  representation so coefficient blocks can be `&'static` or shared.
- **Processor** runs the system against samples.

Execution traits are intentionally small and split by *shape* of
processing rather than by filter family:

| Trait                  | Use for                                                                    |
| ---------------------- | -------------------------------------------------------------------------- |
| `SampleProcessor`      | causal same-rate filters (FIR, biquad, SOS, gain, delay, integrators)      |
| `BlockProcessor`       | block-native algorithms (FFT conv, overlap-add, SIMD batches)              |
| `StreamProcessor`      | variable-rate ops (decimator, interpolator, polyphase resampler, framers)  |
| `WholeSignalProcessor` | whole-array, possibly non-causal (filtfilt, batch spectral)                |
| `AdaptiveProcessor`    | time-varying, signal-driven (LMS, NLMS)                                    |

## Features

| Feature   | Default | What you get                                                          |
| --------- | ------- | --------------------------------------------------------------------- |
| `std`     | yes     | Standard library; implies `alloc`.                                    |
| `alloc`   | yes\*   | `Vec`-backed dynamic processors (`FirDyn`, `SosDyn`, resampler, LMS). |
| `design`  | yes     | Designers: RBJ biquads, windowed-sinc FIR, moving average.            |

`*` `alloc` is on by default because `std` implies it. With
`default-features = false` you get a pure `no_std`, no-alloc core
suitable for microcontrollers.

## Example: design and run a biquad lowpass

```rust
use filterkit::{SampleProcessor, processors::Biquad};
use filterkit::design::BiquadLowpassSpec;

let coeffs = BiquadLowpassSpec { f0: 2_000.0 / 48_000.0, q: 0.707 }
    .design()
    .unwrap();
let mut filter = Biquad::new(coeffs);

let y = filter.process_sample(1.0_f64);
```

## Example: compose filters with combinators

```rust
use filterkit::{ProcessorExt, SampleProcessor, processors::{Biquad, Gain}};
use filterkit::design::{BiquadHighpassSpec, BiquadLowpassSpec};

let hp = Biquad::new(BiquadHighpassSpec { f0: 0.01, q: 0.707 }.design().unwrap());
let lp = Biquad::new(BiquadLowpassSpec  { f0: 0.10, q: 0.707 }.design().unwrap());
let mut chain = hp.then(lp).then(Gain::new(0.7_f64));

let y = chain.process_sample(0.5);
```

## Worked examples

Each `cargo run --example <name>` is self-contained:

- `biquad_lowpass` — single biquad on a sum-of-sines test signal
- `fir_moving_average` — 5-tap MA, designed and run
- `ema_smoothing` — one-pole EMA, three equivalent parameterisations
- `combinator_chain` — HP → LP → gain, RMS swept across the audio band
- `lms_noise_cancel` — adaptive cancellation; converges to plant taps
- `polyphase_resample` — 48 kHz → 44.1 kHz with a windowed-sinc prototype
- `zero_phase_filtfilt` — compare causal vs forward/backward output

## Scope of 0.1

What's in:
- `Reset`, `Prepare`, `Retune`, `Design` traits.
- All five execution traits + extension methods for `SampleProcessor`.
- Concrete processors: `Gain`, `Delay`, `OnePole` (EMA / one-pole IIR),
  `Fir`, `Biquad`, `SosCascade`, `DirectFormI`, `StateSpaceProcessor`,
  plus heap-backed `FirDyn` and `SosDyn`.
- Stream: const-sized `Decimator`, `Interpolator`, and heap-backed
  `PolyphaseResampler`.
- Whole-signal: `ForwardBackward` zero-phase filtering with SciPy-style
  padding and steady-state pass initialisation.
- Adaptive: `Lms`, `Nlms`.
- Design: RBJ biquads (LP/HP/BP/notch), windowed-sinc FIR, moving
  average, exponential moving average (direct α / time constant /
  cutoff Hz); plus `magnitude_at` / `phase_at` for verification.

What's deliberately out (planned for ≥0.2):
- FFT-based block convolution (`BlockProcessor` is currently
  trait-only).
- Higher-order filter design (Butterworth, Chebyshev, Elliptic,
  `tf_to_sos`, `zpk_to_sos`).
- ZPK representation.
- Multi-channel processors.
- Gustafsson initial-condition method for `filtfilt`.
- RLS adaptive.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
