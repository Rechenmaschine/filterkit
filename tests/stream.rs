//! Tests for variable-rate stream processors.

use filterkit::stream::{Decimator, Interpolator};
use filterkit::{Reset, StreamProcessor};

#[test]
fn decimator_passes_every_mth_sample_when_taps_identity() {
    // Identity filter (only one nonzero tap = b0) acts like simple
    // downsampling. With factor 3, we expect inputs at indices 2, 5, 8,
    // ... in the output.
    let taps = [1.0_f32];
    let mut d = Decimator::new(&taps, 3);

    let input: Vec<f32> = (0..30).map(|i| i as f32).collect();
    let mut out = vec![0.0; 10];
    let status = d.process_stream(&input, &mut out);

    assert_eq!(status.consumed, 30);
    assert_eq!(status.produced, 10);
    // factor = 3 starts emitting at index 2.
    let expected: Vec<f32> = (2..=29).step_by(3).map(|i| i as f32).collect();
    assert_eq!(out, expected);
}

#[test]
fn decimator_handles_split_calls() {
    let taps = [1.0_f32];
    let mut d = Decimator::new(&taps, 4);

    let input1: Vec<f32> = (0..6).map(|i| i as f32).collect();
    let input2: Vec<f32> = (6..12).map(|i| i as f32).collect();
    let mut out = vec![0.0; 8];

    let s1 = d.process_stream(&input1, &mut out);
    let s2 = d.process_stream(&input2, &mut out[s1.produced..]);

    assert_eq!(s1.consumed + s2.consumed, 12);
    assert_eq!(s1.produced + s2.produced, 3); // indices 3, 7, 11
    assert_eq!(&out[..3], &[3.0, 7.0, 11.0]);
}

#[test]
fn interpolator_factor_one_is_identity() {
    let taps = [1.0_f32];
    let mut interp = Interpolator::new(&taps, 1);

    let input = [1.0_f32, 2.0, 3.0, 4.0];
    let mut out = vec![0.0; 4];
    let status = interp.process_stream(&input, &mut out);
    assert_eq!(status.consumed, 4);
    assert_eq!(status.produced, 4);
    assert_eq!(out, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn interpolator_resumes_across_output_truncation() {
    // Output buffer is smaller than one input's L=3 sub-phases (only 2
    // slots). The interpolator must consume the input, emit 2 outputs,
    // and remember to emit the third on the next call without taking
    // a new input.
    let taps = [1.0_f32];
    let mut interp = Interpolator::new(&taps, 3);

    let input = [5.0_f32];
    let mut out1 = [0.0_f32; 2];
    let s1 = interp.process_stream(&input, &mut out1);
    assert_eq!(s1.produced, 2);
    assert_eq!(s1.consumed, 1);
    assert_eq!(out1, [5.0, 0.0]);

    // Second call with empty input should drain the remaining phase.
    let mut out2 = [0.0_f32; 4];
    let s2 = interp.process_stream(&[], &mut out2);
    assert_eq!(s2.consumed, 0);
    assert_eq!(s2.produced, 1);
    assert_eq!(out2[0], 0.0);
}

#[test]
fn interpolator_inserts_zeros_for_zero_filter() {
    // Single-tap = 1 prototype + factor 3 should give: input, 0, 0, input, 0, 0, ...
    let taps = [1.0_f32];
    let mut interp = Interpolator::new(&taps, 3);
    let input = [5.0_f32, 7.0];
    let mut out = vec![0.0; 6];
    let status = interp.process_stream(&input, &mut out);
    assert_eq!(status.consumed, 2);
    assert_eq!(status.produced, 6);
    assert_eq!(out, vec![5.0, 0.0, 0.0, 7.0, 0.0, 0.0]);
}

#[test]
fn decimator_reset_clears_state() {
    let taps = [0.5_f32, 0.5];
    let mut d = Decimator::new(&taps, 2);
    let mut out = vec![0.0; 4];
    let xs = [1.0_f32, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    d.process_stream(&xs, &mut out);
    d.reset();
    let mut out2 = vec![0.0; 1];
    d.process_stream(&[0.0, 0.0], &mut out2);
    assert_eq!(out2[0], 0.0);
}

#[cfg(feature = "alloc")]
#[test]
fn polyphase_resampler_2_to_1_matches_decimator() {
    use filterkit::stream::PolyphaseResampler;

    // L = 1, M = 2 with single-tap prototype = simple decimator-by-2.
    let taps = [1.0_f32];
    let mut p = PolyphaseResampler::new(&taps, 1, 2);
    let input: Vec<f32> = (0..10).map(|i| i as f32).collect();
    let mut out = vec![0.0; 5];
    let status = p.process_stream(&input, &mut out);
    // We may stop reading input once the output buffer is full.
    assert!(status.consumed >= 9);
    assert_eq!(status.produced, 5);
    // L=1, M=2 takes inputs 0, 2, 4, 6, 8 (single tap prototype).
    assert_eq!(out, vec![0.0, 2.0, 4.0, 6.0, 8.0]);
}
