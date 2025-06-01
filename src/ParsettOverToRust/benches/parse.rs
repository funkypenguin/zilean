use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use parsett_rust::{parse_title};

fn bench_single_title(c: &mut Criterion) {
    let sample = "Oppenheimer.2023.IMAX.2160p.BluRay.x265.10bit.HDR.DTS-HD.MA.5.1-SAMPLE";

    c.bench_function("parse_title_single", |b| {
        b.iter(|| {
            let _ = parse_title(black_box(sample));
        });
    });
}
criterion_group!(benches, bench_single_title);
criterion_main!(benches);