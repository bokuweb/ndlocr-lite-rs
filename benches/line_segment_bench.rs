use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

fn bench_line_segment(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_segment");
    for &(width, height) in &[(595usize, 842usize), (1240usize, 1754usize)] {
        let rgb = synth_page(width, height);
        let id = format!("{width}x{height}");
        group.bench_with_input(BenchmarkId::new("naive", &id), &rgb, |b, data| {
            b.iter(|| detect_textline_bands_naive(black_box(data), width, height, 220))
        });
        group.bench_with_input(BenchmarkId::new("fast", &id), &rgb, |b, data| {
            b.iter(|| detect_textline_bands_fast(black_box(data), width, height, 220))
        });
    }
    group.finish();
}

fn synth_page(width: usize, height: usize) -> Vec<u8> {
    let mut rgb = vec![255u8; width * height * 3];
    let line_h = (height / 45).max(12);
    let gap = (height / 80).max(8);
    let mut y = gap * 2;
    let mut line_idx = 0usize;
    while y + line_h < height.saturating_sub(gap * 2) {
        let mut x = (width / 12) + ((line_idx * 17) % (width / 18).max(1));
        while x + 24 < width.saturating_sub(width / 12) {
            draw_rect(
                &mut rgb,
                width,
                x,
                y,
                16 + (line_idx % 9),
                line_h.saturating_sub(3),
            );
            x += 28 + (line_idx % 11);
        }
        y += line_h + gap;
        line_idx += 1;
    }
    rgb
}

fn draw_rect(rgb: &mut [u8], width: usize, x: usize, y: usize, w: usize, h: usize) {
    let height = rgb.len() / (width * 3);
    for yy in y..(y + h).min(height) {
        for xx in x..(x + w).min(width) {
            let i = (yy * width + xx) * 3;
            rgb[i] = 0;
            rgb[i + 1] = 0;
            rgb[i + 2] = 0;
        }
    }
}

criterion_group!(benches, bench_line_segment);
criterion_main!(benches);
