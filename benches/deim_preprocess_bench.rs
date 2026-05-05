use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ndlocr_lite_rs::infer::deim;

fn bench_deim_preprocess(c: &mut Criterion) {
    let mut group = c.benchmark_group("deim_preprocess");
    for &(w, h, iw, ih) in &[
        (595usize, 842usize, 800usize, 800usize),
        (1240usize, 1754usize, 800usize, 800usize),
        (1754usize, 1240usize, 800usize, 800usize),
    ] {
        let rgb = make_rgb(w, h);
        let id = format!("{w}x{h}_to_{iw}x{ih}");
        group.bench_with_input(BenchmarkId::new("legacy_direct", &id), &rgb, |b, data| {
            b.iter(|| legacy_direct_preprocess(black_box(data), w, h, iw, ih))
        });
        group.bench_with_input(BenchmarkId::new("current", &id), &rgb, |b, data| {
            b.iter(|| deim::preprocess_rgb_u8(black_box(data), w, h, iw, ih))
        });
    }
    group.finish();
}

fn legacy_direct_preprocess(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<deim::DeimPreprocessOutput> {
    let expected = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("image size overflow"))?;
    if rgb.len() != expected || input_width == 0 || input_height == 0 {
        anyhow::bail!("invalid input");
    }
    let max_wh = width.max(height);
    let mut out = vec![0.0_f32; 3 * input_width * input_height];
    let plane = input_width * input_height;
    let mean = [0.485_f32, 0.456_f32, 0.406_f32];
    let std = [0.229_f32, 0.224_f32, 0.225_f32];
    for y in 0..input_height {
        let sy = y * max_wh / input_height;
        let in_h = sy < height;
        for x in 0..input_width {
            let sx = x * max_wh / input_width;
            let i = y * input_width + x;
            if in_h && sx < width {
                let s = (sy * width + sx) * 3;
                out[i] = (rgb[s] as f32 / 255.0 - mean[0]) / std[0];
                out[plane + i] = (rgb[s + 1] as f32 / 255.0 - mean[1]) / std[1];
                out[plane * 2 + i] = (rgb[s + 2] as f32 / 255.0 - mean[2]) / std[2];
            } else {
                out[i] = -mean[0] / std[0];
                out[plane + i] = -mean[1] / std[1];
                out[plane * 2 + i] = -mean[2] / std[2];
            }
        }
    }
    Ok(deim::DeimPreprocessOutput {
        tensor: out,
        padded_wh: max_wh,
    })
}

fn make_rgb(w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            out[i] = ((x * 17 + y * 11) % 255) as u8;
            out[i + 1] = ((x * 7 + y * 13) % 255) as u8;
            out[i + 2] = ((x * 19 + y * 3) % 255) as u8;
        }
    }
    out
}

criterion_group!(benches, bench_deim_preprocess);
criterion_main!(benches);
