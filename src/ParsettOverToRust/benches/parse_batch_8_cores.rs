use criterion::{Criterion, criterion_group, criterion_main};
use parsett_rust::parse_batch;
use rayon::ThreadPoolBuilder;
use std::hint::black_box;
use std::time::Duration;

fn bench_parse_batch(c: &mut Criterion) {
    ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .expect("Failed to initialize rayon");

    let sample = "Dune.2021.1080p.BluRay.x264.DTS-HD.MA.5.1-SAMPLE";
    let samples: Vec<&str> = std::iter::repeat(sample).take(10_000).collect();

    c.bench_function("parse_batch_8_threads_10_thousand_items", |b| {
        b.iter(|| {
            let _ = parse_batch(black_box(samples.clone()));
        });
    });
}

// Use this to configure Criterion globally
fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(10))
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = bench_parse_batch
}

criterion_main!(benches);
