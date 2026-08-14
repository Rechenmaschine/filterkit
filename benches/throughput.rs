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

criterion_group!(benches, bench_biquad, bench_fir_32, bench_sos_n,);
criterion_main!(benches);
