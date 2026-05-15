//! Sketch of returning a kernel-erased filter as a `Box<dyn …>`
//! trait object, as an alternative to the [`Lowpass`] enum.
//!
//! Run with: `cargo run --example boxed_dispatch`
//!
//! [`Lowpass`]: filterkit::design::Lowpass

use filterkit::design::LowpassSpec;
use filterkit::traits::SampleProcessor;

fn main() {
    let fs = 48_000.0_f64;
    let fc = 200.0_f64;

    // The library hands you a `Box<dyn SampleProcessor<f64, Output = f64>>`.
    // The caller doesn't see whether it's a OnePole or a Biquad —
    // there's no enum to match against.
    let mut filters: Vec<Box<dyn SampleProcessor<f64, Output = f64>>> = vec![
        LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 1 }
            .build_boxed()
            .unwrap(),
        LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 2 }
            .build_boxed()
            .unwrap(),
    ];

    // You can still call the trait API on each box — auto-deref
    // routes method calls through the vtable.
    for (i, f) in filters.iter_mut().enumerate() {
        let mut last = 0.0_f64;
        for n in 0..2000 {
            let x = (n as f64 * 0.05).sin();
            last = f.process_sample(x);
        }
        println!("filter {i}: last sample = {last:+.4}");
    }

    // Reset is a supertrait of SampleProcessor, so it's reachable on
    // the boxed value as well:
    for f in filters.iter_mut() {
        f.reset();
    }
    println!("all filters reset");

    // ----- What you give up vs the enum: -----
    //
    // 1. Combinators. ProcessorExt requires `Sized`, so the following
    //    would NOT compile — boxes are unsized at the trait-object
    //    level:
    //
    //        let chained = boxed_filter.then(other);   // ❌
    //
    //    The enum variant `Lowpass<T>` is `Sized` and chains freely.
    //
    // 2. Pattern matching. You can't ask "is this a OnePole or a
    //    Biquad?" once it's a trait object — that information is
    //    gone behind the vtable.
    //
    // 3. Heap. Each filter is a separate allocation. Negligible per
    //    instance; meaningful at scale.
    //
    // ----- What you get: -----
    //
    // 1. Any future kernel (Sos, custom user processor, etc.) drops
    //    in without changing the return type or breaking callers.
    // 2. Heterogeneous collections — a `Vec<Box<dyn ...>>` of
    //    different filter kinds, as above.
}
