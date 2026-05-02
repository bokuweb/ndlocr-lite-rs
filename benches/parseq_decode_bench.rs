use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ndlocr_lite_rs::infer::parseq;

fn bench_parseq_decode(c: &mut Criterion) {
    let charset = vec!['あ'; 512];
    let mut group = c.benchmark_group("parseq_decode");
    for &(t, csz) in &[
        (32usize, 512usize),
        (96usize, 512usize),
        (192usize, 512usize),
    ] {
        let flat = make_flat_logits(t, csz);
        let id = format!("T{t}_C{csz}");
        group.bench_with_input(
            BenchmarkId::new("legacy_allocating", &id),
            &flat,
            |b, data| {
                b.iter(|| legacy_allocating_predict(black_box(data), t, csz, black_box(&charset)))
            },
        );
        group.bench_with_input(BenchmarkId::new("flat", &id), &flat, |b, data| {
            b.iter(|| {
                parseq::predict_text_from_flat_logits(
                    black_box(data),
                    black_box(t),
                    black_box(csz),
                    black_box(&charset),
                )
            })
        });
    }
    group.finish();
}

fn legacy_allocating_predict(
    flat: &[f32],
    timesteps: usize,
    classes: usize,
    charset: &[char],
) -> anyhow::Result<String> {
    let logits = flat[..(timesteps * classes)]
        .chunks(classes)
        .map(|row| row.to_vec())
        .collect::<Vec<_>>();
    parseq::predict_text_from_logits(&logits, charset)
}

fn make_flat_logits(t: usize, c: usize) -> Vec<f32> {
    (0..t)
        .flat_map(|ti| (0..c).map(move |ci| ((ti * 131 + ci * 17) % 1000) as f32 / 1000.0))
        .collect::<Vec<_>>()
}

criterion_group!(benches, bench_parseq_decode);
criterion_main!(benches);
