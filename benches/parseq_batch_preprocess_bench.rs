use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ndlocr_lite_rs::infer::parseq;
use rayon::prelude::*;

fn bench_parseq_batch_preprocess(c: &mut Criterion) {
    let mut group = c.benchmark_group("parseq_batch_preprocess");
    for &(batch, w, h, iw, ih) in &[
        (8usize, 160usize, 32usize, 384usize, 32usize),
        (32usize, 160usize, 32usize, 384usize, 32usize),
        (32usize, 32usize, 160usize, 384usize, 32usize),
    ] {
        let images = make_images(batch, w, h);
        let id = format!("N{batch}_{w}x{h}_to_{iw}x{ih}");
        group.bench_with_input(
            BenchmarkId::new("allocating_then_copy", &id),
            &images,
            |b, data| b.iter(|| allocating_then_copy(black_box(data), w, h, iw, ih)),
        );
        group.bench_with_input(BenchmarkId::new("direct_slot", &id), &images, |b, data| {
            b.iter(|| direct_slot(black_box(data), w, h, iw, ih))
        });
        group.bench_with_input(
            BenchmarkId::new("parallel_allocating_then_copy", &id),
            &images,
            |b, data| b.iter(|| parallel_allocating_then_copy(black_box(data), w, h, iw, ih)),
        );
        group.bench_with_input(
            BenchmarkId::new("parallel_direct_slot", &id),
            &images,
            |b, data| b.iter(|| parallel_direct_slot(black_box(data), w, h, iw, ih)),
        );
        group.bench_with_input(BenchmarkId::new("old_policy", &id), &images, |b, data| {
            b.iter(|| old_policy(black_box(data), w, h, iw, ih))
        });
        group.bench_with_input(
            BenchmarkId::new("current_policy", &id),
            &images,
            |b, data| b.iter(|| current_policy(black_box(data), w, h, iw, ih)),
        );
    }
    group.finish();
}

fn allocating_then_copy(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let plane = 3 * input_width * input_height;
    let mut batch = vec![0.0f32; images.len() * plane];
    for (slot, rgb) in batch.chunks_mut(plane).zip(images) {
        let tensor = parseq::preprocess_rgb_u8(rgb, width, height, input_width, input_height)?;
        slot.copy_from_slice(&tensor);
    }
    Ok(batch)
}

fn parallel_allocating_then_copy(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let plane = 3 * input_width * input_height;
    let mut batch = vec![0.0f32; images.len() * plane];
    batch
        .par_chunks_mut(plane)
        .zip(images.par_iter())
        .try_for_each(|(slot, rgb)| -> anyhow::Result<()> {
            let tensor = parseq::preprocess_rgb_u8(rgb, width, height, input_width, input_height)?;
            slot.copy_from_slice(&tensor);
            Ok(())
        })?;
    Ok(batch)
}

fn direct_slot(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let plane = 3 * input_width * input_height;
    let mut batch = vec![0.0f32; images.len() * plane];
    let mut scratch = parseq::PreprocessScratch::new();
    for (slot, rgb) in batch.chunks_mut(plane).zip(images) {
        parseq::preprocess_rgb_u8_into_with_scratch(
            slot,
            rgb,
            width,
            height,
            input_width,
            input_height,
            &mut scratch,
        )?;
    }
    Ok(batch)
}

fn parallel_direct_slot(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    let plane = 3 * input_width * input_height;
    let mut batch = vec![0.0f32; images.len() * plane];
    batch
        .par_chunks_mut(plane)
        .zip(images.par_iter())
        .try_for_each_init(
            parseq::PreprocessScratch::new,
            |scratch, (slot, rgb)| -> anyhow::Result<()> {
                parseq::preprocess_rgb_u8_into_with_scratch(
                    slot,
                    rgb,
                    width,
                    height,
                    input_width,
                    input_height,
                    scratch,
                )
            },
        )?;
    Ok(batch)
}

fn old_policy(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    if images.len() < 16 || height > width {
        parallel_direct_slot(images, width, height, input_width, input_height)
    } else {
        parallel_allocating_then_copy(images, width, height, input_width, input_height)
    }
}

fn current_policy(
    images: &[Vec<u8>],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> anyhow::Result<Vec<f32>> {
    if images.len() < 16 {
        parallel_direct_slot(images, width, height, input_width, input_height)
    } else {
        parallel_allocating_then_copy(images, width, height, input_width, input_height)
    }
}

fn make_images(batch: usize, w: usize, h: usize) -> Vec<Vec<u8>> {
    (0..batch)
        .map(|n| {
            let mut out = vec![0u8; w * h * 3];
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) * 3;
                    out[i] = ((x * 17 + y * 11 + n * 5) % 255) as u8;
                    out[i + 1] = ((x * 7 + y * 13 + n * 3) % 255) as u8;
                    out[i + 2] = ((x * 19 + y * 3 + n * 7) % 255) as u8;
                }
            }
            out
        })
        .collect()
}

criterion_group!(benches, bench_parseq_batch_preprocess);
criterion_main!(benches);
