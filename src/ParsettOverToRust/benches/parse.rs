use std::hint::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use parsett_rust::parse_title;

fn bench_parser(c: &mut Criterion) {
    let sample = "Oppenheimer.2023.IMAX.2160p.BluRay.x265.10bit.HDR.DTS-HD.MA.5.1-SAMPLE";

    c.bench_function("parse_title", |b| {
        b.iter(|| {
            let _ = parse_title(black_box(sample));
        });
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);