//! `PageRecognizer` のページ並列スループットを既存の "1 ページずつ
//! `DeimSession::detect` + `ParseqPool::recognize_batch`" 直列ループと
//! 比較するベンチマーク用の小さい example。
//!
//! 走らせ方:
//! ```sh
//! cargo run --example bench_page_recognizer --features onnx --release -- \
//!   --det-model models/deim-s-1024x1024.onnx \
//!   --parseq-model models/parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx \
//!   --charset path/to/NDLmoji.yaml \
//!   --image tests/fixtures/scaned0.png \
//!   --pages 8 --parallelism 4
//! ```

use anyhow::{Result, bail};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    det_model: PathBuf,
    #[arg(long)]
    parseq_model: PathBuf,
    #[arg(long)]
    charset: PathBuf,
    #[arg(long)]
    image: PathBuf,
    #[arg(long, default_value_t = 4)]
    pages: usize,
    #[arg(long, default_value_t = 2)]
    parallelism: usize,
    #[arg(long, default_value_t = 0.3)]
    det_conf: f32,
}

#[cfg(not(feature = "onnx"))]
fn main() -> Result<()> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

#[cfg(feature = "onnx")]
fn main() -> Result<()> {
    use ndlocr_lite_rs::infer::cached::ParseqPool;
    use ndlocr_lite_rs::infer::deim_cached::DeimPool;
    use ndlocr_lite_rs::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
    use ndlocr_lite_rs::io as nd_io;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use ndlocr_lite_rs::pipeline::reading_order::sort_bboxes_in_reading_order;
    use std::time::Instant;

    let args = Args::parse();
    if !args.image.is_file() {
        bail!("image not found: {}", args.image.display());
    }

    let img = nd_io::load_rgb_u8(&args.image)?;
    let pages: Vec<(&[u8], usize, usize)> = (0..args.pages)
        .map(|_| (img.data.as_slice(), img.width, img.height))
        .collect();
    eprintln!(
        "[setup] image {}x{}, {} pages, parallelism={}",
        img.width, img.height, args.pages, args.parallelism
    );

    // ===== 直列パス: DeimPool(1) + ParseqPool(parallelism) を 1 ページずつ叩く =====
    let t = Instant::now();
    let deim = DeimPool::load(&args.det_model, 1)?;
    let parseq = ParseqPool::load(&args.parseq_model, &args.charset, args.parallelism)?;
    eprintln!("[serial] pools loaded in {:.2}s", t.elapsed().as_secs_f64());

    let opts = PageRecognizerOptions {
        det_conf_threshold: args.det_conf,
        ..Default::default()
    };

    let t = Instant::now();
    let mut serial_total_lines = 0usize;
    for (rgb, w, h) in &pages {
        let dets = deim.detect_rgb_u8(rgb, *w, *h, opts.det_conf_threshold)?;
        let mut bboxes: Vec<[i32; 4]> = dets
            .into_iter()
            .filter(|d| d.class_name.starts_with("line_"))
            .filter_map(|d| {
                let [x0, y0, x1, y1] = d.box_xyxy;
                if x0 < 0 || y0 < 0 || x0 >= x1 || y0 >= y1 {
                    return None;
                }
                if (x1 as usize) > *w || (y1 as usize) > *h {
                    return None;
                }
                Some([x0, y0, x1, y1])
            })
            .collect();
        sort_bboxes_in_reading_order(&mut bboxes);
        let crops: Vec<_> = bboxes
            .iter()
            .map(|b| {
                let bb = BBox::new(b[0] as usize, b[1] as usize, b[2] as usize, b[3] as usize);
                crop_rgb_u8(rgb, *w, *h, bb)
            })
            .collect::<Result<_>>()?;
        let inputs: Vec<(&[u8], usize, usize)> = crops
            .iter()
            .map(|c| (c.data.as_slice(), c.width, c.height))
            .collect();
        let recs = parseq.recognize_batch_rgb_u8(&inputs)?;
        serial_total_lines += recs.len();
    }
    let serial_elapsed = t.elapsed().as_secs_f64();
    eprintln!(
        "[serial] {} pages -> {} lines in {:.2}s ({:.2}s/page)",
        args.pages,
        serial_total_lines,
        serial_elapsed,
        serial_elapsed / args.pages as f64
    );

    // ===== 並列パス: PageRecognizer =====
    let t = Instant::now();
    let recognizer = PageRecognizer::load(
        &args.det_model,
        &args.parseq_model,
        &args.charset,
        args.parallelism,
    )?;
    eprintln!(
        "[parallel] PageRecognizer loaded in {:.2}s (parallelism={})",
        t.elapsed().as_secs_f64(),
        recognizer.parallelism()
    );

    let t = Instant::now();
    let results = recognizer.recognize_pages_rgb_u8(&pages, opts)?;
    let parallel_elapsed = t.elapsed().as_secs_f64();
    let parallel_lines: usize = results.iter().map(|p| p.lines.len()).sum();
    eprintln!(
        "[parallel] {} pages -> {} lines in {:.2}s ({:.2}s/page)",
        args.pages,
        parallel_lines,
        parallel_elapsed,
        parallel_elapsed / args.pages as f64
    );

    eprintln!(
        "\n[summary] speedup = {:.2}x  (serial {:.2}s -> parallel {:.2}s)",
        serial_elapsed / parallel_elapsed,
        serial_elapsed,
        parallel_elapsed
    );

    Ok(())
}
