use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ndlocr_lite_rs::infer::parseq;

fn bench_parseq_preprocess(c: &mut Criterion) {
    let mut group = c.benchmark_group("parseq_preprocess");
    for &(w, h, iw, ih) in &[
        (160usize, 32usize, 384usize, 32usize),
        (32usize, 160usize, 384usize, 32usize),
        (1240usize, 1754usize, 384usize, 32usize),
    ] {
        let rgb = make_rgb(w, h);
        let id = format!("{w}x{h}_to_{iw}x{ih}");
        group.bench_with_input(BenchmarkId::new("legacy_copying", &id), &rgb, |b, data| {
            b.iter(|| legacy_preprocess_copying(black_box(data), w, h, iw, ih))
        });
        group.bench_with_input(BenchmarkId::new("current", &id), &rgb, |b, data| {
            b.iter(|| parseq::preprocess_rgb_u8(black_box(data), w, h, iw, ih))
        });
    }
    group.finish();
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

fn legacy_preprocess_copying(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let (rotated, rw, rh) = if height > width {
        rotate_ccw_rgb_u8(rgb, width, height)
    } else {
        (rgb.to_vec(), width, height)
    };
    let resized = resize_nearest_rgb_u8(&rotated, rw, rh, input_width, input_height)?;
    let mut out = vec![0.0_f32; 3 * input_width * input_height];
    let plane = input_width * input_height;
    for y in 0..input_height {
        for x in 0..input_width {
            let s = (y * input_width + x) * 3;
            let i = y * input_width + x;
            out[i] = resized[s] as f32 / 127.5 - 1.0;
            out[plane + i] = resized[s + 1] as f32 / 127.5 - 1.0;
            out[plane * 2 + i] = resized[s + 2] as f32 / 127.5 - 1.0;
        }
    }
    Ok(out)
}

fn rotate_ccw_rgb_u8(rgb: &[u8], width: usize, height: usize) -> (Vec<u8>, usize, usize) {
    let new_w = height;
    let new_h = width;
    let mut out = vec![0_u8; rgb.len()];
    for y in 0..height {
        for x in 0..width {
            let nx = y;
            let ny = width - 1 - x;
            let s = (y * width + x) * 3;
            let d = (ny * new_w + nx) * 3;
            out[d..d + 3].copy_from_slice(&rgb[s..s + 3]);
        }
    }
    (out, new_w, new_h)
}

fn resize_nearest_rgb_u8(
    rgb: &[u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> anyhow::Result<Vec<u8>> {
    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        anyhow::bail!("invalid resize dimension");
    }
    let mut out = vec![0_u8; dst_w * dst_h * 3];
    for y in 0..dst_h {
        let sy = y * src_h / dst_h;
        for x in 0..dst_w {
            let sx = x * src_w / dst_w;
            let s = (sy * src_w + sx) * 3;
            let d = (y * dst_w + x) * 3;
            out[d..d + 3].copy_from_slice(&rgb[s..s + 3]);
        }
    }
    Ok(out)
}

criterion_group!(benches, bench_parseq_preprocess);
criterion_main!(benches);
