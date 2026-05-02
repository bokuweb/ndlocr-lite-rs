//! ellisii NdlocrBackend と同じ呼び出し方で 1 枚の画像を OCR し、
//! 各フェーズの所要時間を出すベンチマーク。
//!
//! 使い方:
//!   cargo run --release --features onnx --example bench_ellisii_like -- \
//!     --image ndlocr/resource/digidepo_3048008_0025.jpg \
//!     --runs 2

use anyhow::{Result, bail};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    image: PathBuf,
    #[arg(long, default_value = "ndlocr/src/model/deim-s-1024x1024.onnx")]
    det_model: PathBuf,
    #[arg(
        long,
        default_value = "ndlocr/src/model/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"
    )]
    model100: PathBuf,
    #[arg(
        long,
        default_value = "ndlocr/src/model/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"
    )]
    model30: PathBuf,
    #[arg(
        long,
        default_value = "ndlocr/src/model/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx"
    )]
    model50: PathBuf,
    #[arg(long, default_value_t = false)]
    cascade: bool,
    #[arg(long, default_value = "ndlocr/src/config/NDLmoji.yaml")]
    charset: PathBuf,
    #[arg(long, default_value_t = 0.3)]
    det_conf: f32,
    #[arg(long, default_value_t = 4)]
    parallelism: usize,
    #[arg(long, default_value_t = 2)]
    runs: usize,
}

#[cfg(not(feature = "onnx"))]
fn main() -> Result<()> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

#[cfg(feature = "onnx")]
fn main() -> Result<()> {
    use anyhow::Context;
    use ndlocr_lite_rs::infer::cached::{ParseqCascadePool, ParseqPool};
    use ndlocr_lite_rs::infer::deim;
    use ndlocr_lite_rs::infer::deim_cached::DeimSession;
    use ndlocr_lite_rs::io as nd_io;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use ndlocr_lite_rs::pipeline::reading_order::sort_bboxes_in_reading_order;
    use std::time::Instant;

    let args = Args::parse();
    if !args.image.is_file() {
        bail!("image not found: {}", args.image.display());
    }

    let mut pool: Option<ParseqPool> = None;
    let mut cascade: Option<ParseqCascadePool> = None;
    if args.cascade {
        let t = Instant::now();
        cascade = Some(
            ParseqCascadePool::load(
                &args.model30,
                &args.model50,
                &args.model100,
                &args.charset,
                args.parallelism,
            )
            .context("cascade pool load")?,
        );
        eprintln!(
            "[setup] cascade pool load (parallelism={}) = {:.2}s",
            args.parallelism,
            t.elapsed().as_secs_f64()
        );
    } else {
        let t = Instant::now();
        pool = Some(
            ParseqPool::load(&args.model100, &args.charset, args.parallelism)
                .context("parseq pool load")?,
        );
        eprintln!(
            "[setup] parseq pool load (parallelism={}) = {:.2}s",
            args.parallelism,
            t.elapsed().as_secs_f64()
        );
    }
    let t = Instant::now();
    let deim_session = DeimSession::load(&args.det_model).context("deim session load")?;
    eprintln!(
        "[setup] deim session load (input {}x{}) = {:.2}s",
        deim_session.input_size().0,
        deim_session.input_size().1,
        t.elapsed().as_secs_f64()
    );

    for run in 0..args.runs {
        eprintln!("\n=== run {run} ===");
        let t_total = Instant::now();

        let t = Instant::now();
        let img = nd_io::load_rgb_u8(&args.image)?;
        eprintln!(
            "[image] load {}x{} = {:.3}s",
            img.width,
            img.height,
            t.elapsed().as_secs_f64()
        );

        let t = Instant::now();
        let _dets_uncached = if run == 0 {
            // 比較用に 1 度だけ uncached も走らせる
            let r = deim::detect_rgb_u8(
                &args.det_model,
                &img.data,
                img.width,
                img.height,
                args.det_conf,
            )?;
            eprintln!("[deim uncached] detect = {:.3}s", t.elapsed().as_secs_f64());
            r
        } else {
            Vec::new()
        };
        let t = Instant::now();
        let dets = deim_session.detect_rgb_u8(&img.data, img.width, img.height, args.det_conf)?;
        eprintln!(
            "[deim cached] detect ({} dets) = {:.3}s",
            dets.len(),
            t.elapsed().as_secs_f64()
        );

        let mut lines: Vec<([i32; 4], f32)> = dets
            .into_iter()
            .filter(|d| d.class_name.starts_with("line_"))
            .filter_map(|d| {
                let [x0, y0, x1, y1] = d.box_xyxy;
                if x0 < 0 || y0 < 0 || x0 >= x1 || y0 >= y1 {
                    return None;
                }
                if (x1 as usize) > img.width || (y1 as usize) > img.height {
                    return None;
                }
                Some(([x0, y0, x1, y1], d.pred_char_count))
            })
            .collect();
        let mut bboxes: Vec<[i32; 4]> = lines.iter().map(|(b, _)| *b).collect();
        sort_bboxes_in_reading_order(&mut bboxes);
        // ソート後 idx を維持するため再構築
        lines.sort_by_key(|(b, _)| bboxes.iter().position(|x| x == b).unwrap_or(usize::MAX));
        let n30 = lines.iter().filter(|(_, p)| (p - 3.0).abs() < 0.2).count();
        let n50 = lines.iter().filter(|(_, p)| (p - 2.0).abs() < 0.2).count();
        let n100 = lines.len() - n30 - n50;
        eprintln!(
            "[lines] {} line_* (cascade buckets: 30={}, 50={}, 100={})",
            lines.len(),
            n30,
            n50,
            n100
        );

        let t = Instant::now();
        let crops: Vec<_> = lines
            .iter()
            .map(|(b, _)| {
                let (x0, y0, x1, y1) = (b[0] as usize, b[1] as usize, b[2] as usize, b[3] as usize);
                crop_rgb_u8(&img.data, img.width, img.height, BBox::new(x0, y0, x1, y1)).unwrap()
            })
            .collect();
        eprintln!("[crop] = {:.3}s", t.elapsed().as_secs_f64());

        let t = Instant::now();
        let recs = if let Some(c) = cascade.as_ref() {
            let inputs: Vec<(&[u8], usize, usize, Option<f32>)> = crops
                .iter()
                .zip(lines.iter())
                .map(|(c, (_, p))| (c.data.as_slice(), c.width, c.height, Some(*p)))
                .collect();
            c.recognize_batch_with_buckets_rgb_u8(&inputs)?
        } else {
            let inputs: Vec<(&[u8], usize, usize)> = crops
                .iter()
                .map(|c| (c.data.as_slice(), c.width, c.height))
                .collect();
            pool.as_ref().unwrap().recognize_batch_rgb_u8(&inputs)?
        };
        let label = if cascade.is_some() {
            "cascade"
        } else {
            "parseq-100"
        };
        eprintln!(
            "[parseq {label} batch] {} lines = {:.3}s",
            recs.len(),
            t.elapsed().as_secs_f64()
        );

        eprintln!("[total] = {:.3}s", t_total.elapsed().as_secs_f64());
    }
    Ok(())
}
