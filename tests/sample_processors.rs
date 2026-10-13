//! Tests for sample-by-sample processors.

use approx::assert_relative_eq;
use filterkit::processors::{Biquad, Fir};
use filterkit::{
    BiquadCoeffs, Delay, FirCoeffs, FirstOrder, FirstOrderCoeffs, Gain, ProcessorExt,
    SampleProcessor,
};

#[test]
fn delay_returns_inputs_n_steps_later() {
    let mut d: Delay<f32, 3> = Delay::new();
    let out = [1.0, 2.0, 3.0, 4.0, 5.0].map(|x| d.process_sample(x));
    assert_eq!(out, [0.0, 0.0, 0.0, 1.0, 2.0]);
}

#[test]
fn delay_zero_length_is_passthrough() {
    let mut d: Delay<f32, 0> = Delay::new();
    assert_eq!(d.process_sample(0.5), 0.5);
}

#[test]
fn fir_moving_average_matches_hand_computed() {
    let mut fir = Fir::new(FirCoeffs::new([1.0_f32 / 3.0; 3]));
    let out = [3.0, 6.0, 9.0, 12.0].map(|x| fir.process_sample(x));
    for (got, want) in out.into_iter().zip([1.0, 3.0, 6.0, 9.0]) {
        assert_relative_eq!(got, want, epsilon = 1e-6);
    }
}

#[test]
fn first_order_matches_difference_equation() {
    // y[n] = 0.25 x[n] + 0.5 x[n-1] + 0.25 y[n-1]
    let mut p = FirstOrder::new(FirstOrderCoeffs::new(0.25_f64, 0.5, -0.25));
    let out = [1.0, 0.0, 0.0, 0.0].map(|x| p.process_sample(x));
    for (got, want) in out.into_iter().zip([0.25, 0.5625, 0.140625, 0.03515625]) {
        assert_relative_eq!(got, want, epsilon = 1e-12);
    }
}

#[test]
fn in_place_matches_process_into() {
    let c = BiquadCoeffs::new(0.5_f32, 0.2, 0.1, -0.3, 0.1);
    let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut out = [0.0; 5];
    Biquad::new(c).process_into(&xs, &mut out);
    let mut buf = xs;
    Biquad::new(c).process_in_place(&mut buf);
    assert_eq!(out, buf);
}

#[test]
fn combinators() {
    assert_eq!(
        Gain::new(2.0_f32).then(Gain::new(3.0)).process_sample(1.0),
        6.0
    );
    assert_eq!(
        Gain::new(2.0_f32)
            .parallel(Gain::new(-1.0))
            .process_sample(4.0),
        (8.0, -4.0)
    );
    assert_eq!(
        Gain::new(2.0_f32).sum(Gain::new(3.0)).process_sample(1.5),
        7.5
    );
    assert_eq!(Gain::new(2.0_f32).map(|y| y + 1.0).process_sample(3.0), 7.0);
    // 0.75 * x + 0.25 * 4x
    assert_eq!(Gain::new(4.0_f32).wet_dry(0.25).process_sample(1.0), 1.75);

    let mut b = Gain::new(2.0_f32).bypass();
    assert_eq!(b.process_sample(1.0), 2.0);
    b.set_enabled(false);
    assert_eq!(b.process_sample(1.0), 1.0);
}

#[cfg(feature = "alloc")]
#[test]
fn fir_dyn_retune_keeps_history_only_for_same_length() {
    use filterkit::processors::FirDyn;
    use filterkit::Retune;

    let mut f = FirDyn::new(&[1.0_f64, 0.0]);
    f.process_sample(3.0);
    f.retune(vec![0.0, 1.0]);
    assert_eq!(f.process_sample(0.0), 3.0);

    f.retune(vec![0.0, 0.0, 1.0]);
    assert_eq!(f.taps(), &[0.0, 0.0, 1.0]);
    assert_eq!(
        [5.0, 0.0, 0.0].map(|x| f.process_sample(x)),
        [0.0, 0.0, 5.0]
    );
}

#[cfg(feature = "alloc")]
#[test]
fn sos_dyn_retune_resizes_state() {
    use filterkit::processors::SosDyn;
    use filterkit::Retune;

    let mut s = SosDyn::new(&[BiquadCoeffs::<f64>::identity()]);
    s.retune(vec![BiquadCoeffs::identity(); 3]);
    assert_eq!(s.sections().len(), 3);
    assert_eq!(s.process_sample(2.0), 2.0);
}
