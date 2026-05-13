//! Tests for sample-by-sample processors.

use approx::assert_relative_eq;
use filterkit::{
    BiquadCoeffs, Delay, FirCoeffs, Gain, OnePole, ProcessorExt, Reset, Retune, SampleFilter,
    SampleProcessor,
};
use filterkit::processors::{Biquad, Fir};

#[test]
fn gain_is_pure_multiplication() {
    let mut g = Gain::new(2.5_f32);
    assert_eq!(g.process_sample(1.0), 2.5);
    assert_eq!(g.process_sample(-2.0), -5.0);
    g.retune(0.0);
    assert_eq!(g.process_sample(99.0), 0.0);
}

#[test]
fn delay_returns_inputs_n_steps_later() {
    let mut d: Delay<f32, 3> = Delay::new();
    let inputs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let expected = [0.0, 0.0, 0.0, 1.0, 2.0];
    for (x, want) in inputs.into_iter().zip(expected) {
        let got = d.process_sample(x);
        assert_eq!(got, want);
    }
}

#[test]
fn delay_zero_length_is_passthrough() {
    let mut d: Delay<f32, 0> = Delay::new();
    for x in [0.5, -1.0, 2.0] {
        assert_eq!(d.process_sample(x), x);
    }
}

#[test]
fn fir_identity_impulse() {
    // FIR with b = [1, 0, 0, 0] is identity.
    let coeffs = FirCoeffs::new([1.0_f32, 0.0, 0.0, 0.0]);
    let mut fir = Fir::new(coeffs);
    let inputs = [3.0, -1.5, 0.25, 7.0];
    for x in inputs {
        assert_eq!(fir.process_sample(x), x);
    }
}

#[test]
fn fir_moving_average_matches_hand_computed() {
    // 3-tap MA with equal weights.
    let coeffs = FirCoeffs::new([1.0_f32 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    let mut fir = Fir::new(coeffs);
    let inputs = [3.0_f32, 6.0, 9.0, 12.0];
    let expected = [1.0, 3.0, 6.0, 9.0];
    for (x, want) in inputs.into_iter().zip(expected) {
        assert_relative_eq!(fir.process_sample(x), want, epsilon = 1e-6);
    }
}

#[test]
fn fir_reset_clears_state() {
    let coeffs = FirCoeffs::new([1.0_f32, 1.0, 1.0]);
    let mut fir = Fir::new(coeffs);
    fir.process_sample(10.0);
    fir.process_sample(20.0);
    fir.reset();
    // After reset, first sample with input=0 should yield 0.
    assert_eq!(fir.process_sample(0.0), 0.0);
}

#[test]
fn biquad_passthrough_when_identity() {
    let coeffs = BiquadCoeffs::identity();
    let mut bq = Biquad::new(coeffs);
    let inputs = [0.1_f32, 0.2, -0.3, 0.4];
    for x in inputs {
        assert_relative_eq!(bq.process_sample(x), x, epsilon = 1e-6);
    }
}

#[test]
fn biquad_in_place_matches_process_into() {
    let coeffs = BiquadCoeffs::new(0.5_f32, 0.0, 0.0, 0.0, 0.0);
    let mut bq_a = Biquad::new(coeffs);
    let mut bq_b = Biquad::new(coeffs);
    let xs: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut out_a = [0.0; 5];
    bq_a.process_into(&xs, &mut out_a);

    let mut buf = xs;
    bq_b.process_in_place(&mut buf);

    for i in 0..5 {
        assert_relative_eq!(out_a[i], buf[i], epsilon = 1e-6);
    }
}

#[test]
fn chain_composes_processors() {
    // Two gains in series multiply.
    let g1 = Gain::new(2.0_f32);
    let g2 = Gain::new(3.0_f32);
    let mut chain = g1.then(g2);
    assert_eq!(chain.process_sample(1.0), 6.0);
    assert_eq!(chain.process_sample(-1.5), -9.0);
}

#[test]
fn parallel_emits_tuple_outputs() {
    let mut p = Gain::new(2.0_f32).parallel(Gain::new(-1.0_f32));
    let (a, b) = p.process_sample(4.0);
    assert_eq!(a, 8.0);
    assert_eq!(b, -4.0);
}

#[test]
fn sum_adds_branch_outputs() {
    let mut s = Gain::new(2.0_f32).sum(Gain::new(3.0_f32));
    assert_eq!(s.process_sample(1.5), 7.5);
}

#[test]
fn bypass_disables_inner_processor() {
    let mut b = Gain::new(2.0_f32).bypass();
    assert_eq!(b.process_sample(1.0), 2.0);
    b.set_enabled(false);
    assert_eq!(b.process_sample(1.0), 1.0);
}

#[test]
fn wet_dry_blends() {
    // wet = 0.25, gain = 4x -> blend = 0.75*x + 0.25*(4*x) = 1.75*x
    let mut wd = Gain::new(4.0_f32).wet_dry(0.25);
    assert_relative_eq!(wd.process_sample(1.0), 1.75, epsilon = 1e-6);
}

#[test]
fn one_pole_alpha_one_is_passthrough() {
    let mut p = OnePole::new(1.0_f32);
    for x in [0.5, -0.3, 1.2, -2.0] {
        assert_relative_eq!(p.process_sample(x), x, epsilon = 1e-6);
    }
}

#[test]
fn one_pole_converges_to_dc_input() {
    // Step response of EMA reaches the input asymptotically.
    let mut p = OnePole::new(0.1_f32);
    let mut last = 0.0;
    for _ in 0..500 {
        last = p.process_sample(1.0);
    }
    assert!((last - 1.0).abs() < 1e-3, "EMA didn't converge: {last}");
}

#[test]
fn one_pole_with_state_continues_block() {
    // First block through a fresh filter, then second block through a
    // filter warm-started with the prior output — output must equal
    // running the whole signal through one filter.
    let signal: [f32; 20] = [
        0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5, -0.4,
        -0.3, -0.2, -0.1, 0.0,
    ];
    let alpha = 0.25_f32;
    let split = 8;

    let mut full = OnePole::new(alpha);
    let full_out: Vec<f32> = signal.iter().map(|&x| full.process_sample(x)).collect();

    let mut a = OnePole::new(alpha);
    let mut last = 0.0;
    for &x in &signal[..split] {
        last = a.process_sample(x);
    }
    let mut b = OnePole::with_state(alpha, last);
    for n in split..signal.len() {
        let y = b.process_sample(signal[n]);
        assert_relative_eq!(y, full_out[n], max_relative = 1e-6, epsilon = 1e-7);
    }
}

#[test]
fn map_post_processes_output() {
    let mut m = Gain::new(2.0_f32).map(|y: f32| y + 1.0);
    assert_eq!(m.process_sample(3.0), 7.0);
}
