//! Sample-throughput benchmarks for the core processors.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use filterkit::coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs};
use filterkit::processors::{Biquad, Fir, SosCascade};
use filterkit::SampleProcessor;

const BLOCK: usize = 4096;

fn make_input() -> Vec<f32> {
    (0..BLOCK)
        .map(|i| (i as f32 * 0.137).sin() + 0.5 * (i as f32 * 0.41).cos())
        .collect()
}

fn bench_biquad(c: &mut Criterion) {
    let coeffs = BiquadCoeffs::new(0.3_f32, 0.5, 0.2, -0.4, 0.1);
    let input = make_input();
    let mut output = vec![0.0_f32; BLOCK];

    let mut g = c.benchmark_group("biquad");
    g.throughput(Throughput::Elements(BLOCK as u64));
    g.bench_function("process_into 4096", |b| {
        let mut bq = Biquad::new(coeffs);
        b.iter(|| {
            bq.process_into(black_box(&input), &mut output);
            black_box(&output[0]);
        });
    });
}

fn bench_fir_32(c: &mut Criterion) {
    let coeffs = FirCoeffs::new([1.0_f32 / 32.0; 32]);
    let input = make_input();
    let mut output = vec![0.0_f32; BLOCK];

    let mut g = c.benchmark_group("fir32");
    g.throughput(Throughput::Elements(BLOCK as u64));
    g.bench_function("process_into 4096", |b| {
        let mut fir = Fir::new(coeffs);
        b.iter(|| {
            fir.process_into(black_box(&input), &mut output);
            black_box(&output[0]);
        });
    });
}

fn bench_sos_n(c: &mut Criterion) {
    let section = BiquadCoeffs::new(0.3_f32, 0.5, 0.2, -0.4, 0.1);
    let input = make_input();
    let mut output = vec![0.0_f32; BLOCK];

    let mut g = c.benchmark_group("sos");
    g.throughput(Throughput::Elements(BLOCK as u64));

    // Run separate const-N benches so we can see per-section cost
    // grow (or, more interestingly, *not* grow linearly with N).
    {
        let coeffs = SosCoeffs::new([section; 1]);
        g.bench_function("N=1", |b| {
            let mut sos = SosCascade::new(coeffs);
            b.iter(|| {
                sos.process_into(black_box(&input), &mut output);
                black_box(&output[0]);
            });
        });
    }
    {
        let coeffs = SosCoeffs::new([section; 2]);
        g.bench_function("N=2", |b| {
            let mut sos = SosCascade::new(coeffs);
            b.iter(|| {
                sos.process_into(black_box(&input), &mut output);
                black_box(&output[0]);
            });
        });
    }
    {
        let coeffs = SosCoeffs::new([section; 4]);
        g.bench_function("N=4", |b| {
            let mut sos = SosCascade::new(coeffs);
            b.iter(|| {
                sos.process_into(black_box(&input), &mut output);
                black_box(&output[0]);
            });
        });
    }
    {
        let coeffs = SosCoeffs::new([section; 8]);
        g.bench_function("N=8", |b| {
            let mut sos = SosCascade::new(coeffs);
            b.iter(|| {
                sos.process_into(black_box(&input), &mut output);
                black_box(&output[0]);
            });
        });
    }
}

fn bench_dispatch_strategies(c: &mut Criterion) {
    use filterkit::design::{Lowpass, LowpassSpec};

    let input: Vec<f64> = (0..BLOCK)
        .map(|i| (i as f64 * 0.137).sin() + 0.5 * (i as f64 * 0.41).cos())
        .collect();
    let mut output = vec![0.0_f64; BLOCK];

    let spec = LowpassSpec { cutoff_hz: 200.0, sample_rate: 48_000.0, order: 2 };

    let mut g = c.benchmark_group("lowpass_dispatch");
    g.throughput(Throughput::Elements(BLOCK as u64));

    // 1. Concrete kernel (as_biquad). Zero-cost dispatch baseline.
    g.bench_function("as_biquad concrete", |b| {
        let mut bq = spec.as_biquad::<f64>().unwrap();
        b.iter(|| {
            bq.process_into(black_box(&input), &mut output);
            black_box(&output[0]);
        });
    });

    // 2a. Enum dispatch via build() -> Lowpass<T>.
    //     User-written hot loop — match runs INSIDE per sample.
    g.bench_function("Lowpass enum, match-per-sample loop", |b| {
        let mut lp: Lowpass<f64> = spec.build().unwrap();
        b.iter(|| {
            for (x, y) in input.iter().zip(output.iter_mut()) {
                *y = lp.process_sample(*x);
            }
            black_box(&output[0]);
        });
    });

    // 2b. Enum dispatch via build() -> Lowpass<T>.
    //     Same enum, but uses the overridden process_into that hoists
    //     the match OUT of the loop. This is the path the library
    //     wants you to use for block processing.
    g.bench_function("Lowpass enum, process_into (match hoisted)", |b| {
        let mut lp: Lowpass<f64> = spec.build().unwrap();
        b.iter(|| {
            lp.process_into(black_box(&input), &mut output);
            black_box(&output[0]);
        });
    });

    // 3. Boxed trait object. One vtable lookup per sample.
    g.bench_function("Box<dyn> trait object", |b| {
        let mut bx = spec.build_boxed::<f64>().unwrap();
        b.iter(|| {
            for (x, y) in input.iter().zip(output.iter_mut()) {
                *y = bx.process_sample(*x);
            }
            black_box(&output[0]);
        });
    });
}

criterion_group!(
    benches,
    bench_biquad,
    bench_fir_32,
    bench_sos_n,
    bench_dispatch_strategies,
);
criterion_main!(benches);
