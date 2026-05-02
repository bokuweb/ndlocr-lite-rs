use anyhow::{Result, bail};
use std::path::Path;

#[cfg(feature = "onnx")]
use crate::cli::RecognizePageArgs;
use crate::cli::{Cli, Command, LineOrientation, MockPageArgs};
#[cfg(feature = "onnx")]
use crate::infer::cached::{ParseqPool, default_parseq_parallelism};
use crate::infer::{deim, parseq};
use crate::output::artifacts::save_page_artifacts;
use crate::output::docx::save_text_as_docx;
use crate::pipeline::connect::RecognizedLine;
use crate::pipeline::crop::{BBox, crop_rgb_u8, expand_bbox_xyxy_clamped};
use crate::pipeline::line_segment::detect_textline_bands_naive;
use crate::pipeline::reading_order::sort_lines_in_reading_order;
use crate::pipeline::run_page::{PageInput, run_page};
use crate::postprocess::dict::PostprocessDict;
use crate::postprocess::page_rules::apply_structural_rules;
use crate::postprocess::rule_pack::RulePack;

pub fn build_mock_line_detections(
    width: usize,
    height: usize,
    requested_count: usize,
    confidence: f32,
) -> Vec<deim::Detection> {
    let mx = (width / 50).max(1);
    let my = (height / 50).max(1);
    let (x0, x1) = if width > 2 * mx {
        (mx, width - mx)
    } else {
        (0, width.max(1))
    };
    let inner_top = my.min(height.saturating_sub(1));
    let inner_bottom = height.saturating_sub(my).max(inner_top + 1);
    let inner_h = (inner_bottom - inner_top).max(1);
    let count = requested_count.max(1).min(inner_h);

    (0..count)
        .map(|i| {
            let y0 = inner_top + i * inner_h / count;
            let mut y1 = inner_top + (i + 1) * inner_h / count;
            if y1 <= y0 {
                y1 = (y0 + 1).min(height.max(1));
            }
            deim::Detection {
                class_index: 1,
                confidence,
                box_xyxy: [x0 as i32, y0 as i32, x1 as i32, y1 as i32],
                pred_char_count: 100.0,
                class_name: "line_main".to_string(),
            }
        })
        .collect()
}

pub fn normalize_line_count(requested_count: usize, image_height: usize) -> usize {
    requested_count.max(1).min(image_height.max(1))
}

pub fn normalize_line_confidence(requested_confidence: f32) -> f32 {
    requested_confidence.clamp(0.0, 1.0)
}

struct MockConfig {
    line_text: String,
    line_count: usize,
    line_confidence: f32,
    output_stem: Option<String>,
    line_orientation: Option<LineOrientation>,
}

impl MockConfig {
    fn from_args(args: &MockPageArgs, image_height: usize) -> Self {
        Self {
            line_text: args
                .line_text
                .clone()
                .unwrap_or_else(|| "mock100".to_string()),
            line_count: normalize_line_count(args.line_count.unwrap_or(1), image_height),
            line_confidence: normalize_line_confidence(args.line_confidence.unwrap_or(1.0)),
            output_stem: args.output_stem.clone(),
            line_orientation: args.line_orientation.clone(),
        }
    }

    fn validate(self) -> Result<Self> {
        if self.line_text.trim().is_empty() {
            bail!("line_text must not be empty");
        }
        if self
            .output_stem
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
        {
            bail!("output_stem must not be empty");
        }
        Ok(self)
    }
}

#[cfg(feature = "onnx")]
struct CachedPageRecognizer {
    pool100: ParseqPool,
    pool30: Option<ParseqPool>,
    pool50: Option<ParseqPool>,
}

#[cfg(feature = "onnx")]
impl CachedPageRecognizer {
    fn load(args: &RecognizePageArgs) -> Result<Self> {
        let parallelism = default_parseq_parallelism();
        let pool100 = ParseqPool::load(&args.model, &args.charset, parallelism)?;
        let pool30 = if args.model30.is_file() {
            Some(ParseqPool::load(&args.model30, &args.charset, parallelism)?)
        } else {
            None
        };
        let pool50 = if args.model50.is_file() {
            Some(ParseqPool::load(&args.model50, &args.charset, parallelism)?)
        } else {
            None
        };
        Ok(Self {
            pool100,
            pool30,
            pool50,
        })
    }

    fn recognize_for_page(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
        pred_char_count_hint: Option<f32>,
        args: &RecognizePageArgs,
    ) -> Result<parseq::RecognizeResult> {
        let recognized = self.recognize_line_with_optional_cascade(
            args.enable_cascade,
            args.cascade_threshold_30_to_50,
            args.cascade_threshold_50_to_100,
            pred_char_count_hint,
            rgb,
            width,
            height,
        )?;
        let recognized = self.maybe_split_long_line_and_recognize(
            recognized,
            rgb,
            width,
            height,
            args.split_long_lines,
            args.split_long_line_char_threshold,
        )?;
        self.maybe_rerank_line_quality(
            recognized,
            rgb,
            width,
            height,
            args.quality_boost,
            args.quality_boost_min_delta,
            args.split_long_lines,
            args.split_long_line_char_threshold,
            args.cascade_threshold_30_to_50,
            args.cascade_threshold_50_to_100,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn recognize_line_with_optional_cascade(
        &self,
        enable_cascade: bool,
        th_30_to_50: usize,
        th_50_to_100: usize,
        pred_char_count_hint: Option<f32>,
        rgb: &[u8],
        width: usize,
        height: usize,
    ) -> Result<parseq::RecognizeResult> {
        if !enable_cascade {
            return self.pool100.recognize_rgb_u8(rgb, width, height);
        }

        let pred_char_bucket = pred_char_count_hint
            .map(normalize_pred_char_bucket)
            .unwrap_or_else(|| estimate_pred_char_bucket(width, height, th_30_to_50, th_50_to_100));

        let mut out = if pred_char_bucket == 3.0 {
            self.recognize_30_or_100(rgb, width, height)?
        } else if pred_char_bucket == 2.0 {
            self.recognize_50_or_100(rgb, width, height)?
        } else {
            self.pool100.recognize_rgb_u8(rgb, width, height)?
        };

        if pred_char_bucket == 3.0 && out.text.chars().count() >= th_30_to_50 {
            out = self.recognize_50_or_100(rgb, width, height)?;
        }
        if out.text.chars().count() >= th_50_to_100 {
            out = self.pool100.recognize_rgb_u8(rgb, width, height)?;
        }
        Ok(out)
    }

    fn maybe_split_long_line_and_recognize(
        &self,
        recognized: parseq::RecognizeResult,
        rgb: &[u8],
        width: usize,
        height: usize,
        split_long_lines: bool,
        split_long_line_char_threshold: usize,
    ) -> Result<parseq::RecognizeResult> {
        if !split_long_lines {
            return Ok(recognized);
        }
        let char_count = recognized.text.chars().count();
        if char_count < split_long_line_char_threshold || width < height.saturating_mul(4) {
            return Ok(recognized);
        }
        let mid = width / 2;
        if mid == 0 || mid >= width {
            return Ok(recognized);
        }
        let left = crop_rgb_u8(rgb, width, height, BBox::new(0, 0, mid, height))?;
        let right = crop_rgb_u8(rgb, width, height, BBox::new(mid, 0, width, height))?;
        let left_rec = self
            .pool100
            .recognize_rgb_u8(&left.data, left.width, left.height)?;
        let right_rec = self
            .pool100
            .recognize_rgb_u8(&right.data, right.width, right.height)?;

        let left_len = left_rec.text.chars().count();
        let right_len = right_rec.text.chars().count();
        if left_len == 0 && right_len == 0 {
            return Ok(recognized);
        }
        let text = format!("{}{}", left_rec.text, right_rec.text);
        let total_len = (left_len + right_len) as f32;
        let mean_confidence = if total_len > 0.0 {
            (left_rec.mean_confidence * left_len as f32
                + right_rec.mean_confidence * right_len as f32)
                / total_len
        } else {
            recognized.mean_confidence
        };
        Ok(parseq::RecognizeResult {
            text,
            mean_confidence,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn maybe_rerank_line_quality(
        &self,
        base: parseq::RecognizeResult,
        rgb: &[u8],
        width: usize,
        height: usize,
        quality_boost: bool,
        quality_boost_min_delta: f32,
        split_long_lines: bool,
        split_long_line_char_threshold: usize,
        cascade_th_30_to_50: usize,
        cascade_th_50_to_100: usize,
    ) -> Result<parseq::RecognizeResult> {
        if !quality_boost {
            return Ok(base);
        }

        let mut best = base.clone();
        let mut best_score = quality_score(&base.text, base.mean_confidence);

        let direct100 = self.pool100.recognize_rgb_u8(rgb, width, height)?;
        let direct100_score = quality_score(&direct100.text, direct100.mean_confidence);
        if direct100_score > best_score + quality_boost_min_delta {
            best = direct100.clone();
            best_score = direct100_score;
        }

        let bucket =
            estimate_pred_char_bucket(width, height, cascade_th_30_to_50, cascade_th_50_to_100);

        if bucket == 3.0
            && let Some(pool30) = self.pool30.as_ref()
        {
            let direct30 = pool30.recognize_rgb_u8(rgb, width, height)?;
            let direct30_score = quality_score(&direct30.text, direct30.mean_confidence);
            if direct30_score > best_score + quality_boost_min_delta {
                best = direct30;
                best_score = direct30_score;
            }
        }

        if bucket == 2.0
            && let Some(pool50) = self.pool50.as_ref()
        {
            let direct50 = pool50.recognize_rgb_u8(rgb, width, height)?;
            let direct50_score = quality_score(&direct50.text, direct50.mean_confidence);
            if direct50_score > best_score + quality_boost_min_delta {
                best = direct50;
                best_score = direct50_score;
            }
        }

        let split100 = self.maybe_split_long_line_and_recognize(
            direct100,
            rgb,
            width,
            height,
            split_long_lines,
            split_long_line_char_threshold,
        )?;
        let split100_score = quality_score(&split100.text, split100.mean_confidence);
        if split100_score > best_score + quality_boost_min_delta {
            best = split100;
        }

        Ok(best)
    }

    fn recognize_30_or_100(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
    ) -> Result<parseq::RecognizeResult> {
        match self.pool30.as_ref() {
            Some(pool) => pool.recognize_rgb_u8(rgb, width, height),
            None => self.pool100.recognize_rgb_u8(rgb, width, height),
        }
    }

    fn recognize_50_or_100(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
    ) -> Result<parseq::RecognizeResult> {
        match self.pool50.as_ref() {
            Some(pool) => pool.recognize_rgb_u8(rgb, width, height),
            None => self.pool100.recognize_rgb_u8(rgb, width, height),
        }
    }
}

pub fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Detect(args) => {
            deim::smoke_detect(&args.model, &args.image)?;
            println!("DEIM model load succeeded.");
        }
        Command::Recognize(args) => {
            let text = parseq::recognize_image(&args.model, &args.image, &args.charset)?;
            println!("{text}");
        }
        Command::RecognizePage(args) => {
            let post_dict = if let Some(path) = args.post_dict.as_ref() {
                Some(PostprocessDict::load_yaml(path)?)
            } else {
                None
            };
            let rule_pack = if let Some(path) = args.rule_pack.as_ref() {
                Some(RulePack::load_yaml(path)?)
            } else {
                None
            };
            let img = crate::io::load_rgb_u8(&args.image)?;
            let deim_lines = if args.use_deim_detection {
                match deim::detect_rgb_u8(
                    &args.det_model,
                    &img.data,
                    img.width,
                    img.height,
                    args.det_conf_threshold,
                ) {
                    Ok(dets) => dets
                        .into_iter()
                        .filter(|d| d.class_name.starts_with("line_"))
                        .collect::<Vec<_>>(),
                    Err(err) => {
                        eprintln!("warn: deim detection failed, fallback to naive: {err}");
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };
            let line_detections = if deim_lines.is_empty() {
                detect_textline_bands_naive(
                    &img.data,
                    img.width,
                    img.height,
                    args.binarize_threshold,
                )
                .into_iter()
                .map(|[x0, y0, x1, y1]| deim::Detection {
                    class_index: 1,
                    confidence: 1.0,
                    box_xyxy: [x0 as i32, y0 as i32, x1 as i32, y1 as i32],
                    pred_char_count: estimate_pred_char_bucket(
                        x1.saturating_sub(x0),
                        y1.saturating_sub(y0),
                        args.cascade_threshold_30_to_50,
                        args.cascade_threshold_50_to_100,
                    ),
                    class_name: "line_main".to_string(),
                })
                .collect::<Vec<_>>()
            } else {
                deim_lines
            };
            #[cfg(feature = "onnx")]
            let cached_parseq = CachedPageRecognizer::load(&args)?;
            let mut recognized_lines = Vec::new();
            let pad = args.line_crop_padding as usize;
            for det in line_detections {
                let [x0, y0, x1, y1] = [
                    det.box_xyxy[0].max(0) as usize,
                    det.box_xyxy[1].max(0) as usize,
                    det.box_xyxy[2].max(0) as usize,
                    det.box_xyxy[3].max(0) as usize,
                ];
                if x0 >= x1 || y0 >= y1 || x1 > img.width || y1 > img.height {
                    continue;
                }
                let (x0, y0, x1, y1) =
                    expand_bbox_xyxy_clamped(x0, y0, x1, y1, pad, img.width, img.height);
                let crop =
                    crop_rgb_u8(&img.data, img.width, img.height, BBox::new(x0, y0, x1, y1))?;
                #[cfg(feature = "onnx")]
                let recognized = cached_parseq.recognize_for_page(
                    &crop.data,
                    crop.width,
                    crop.height,
                    Some(det.pred_char_count),
                    &args,
                )?;
                #[cfg(not(feature = "onnx"))]
                let recognized = {
                    let recognized = recognize_line_with_optional_cascade(
                        &args.model,
                        &args.model30,
                        &args.model50,
                        args.enable_cascade,
                        args.cascade_threshold_30_to_50,
                        args.cascade_threshold_50_to_100,
                        Some(det.pred_char_count),
                        &crop.data,
                        crop.width,
                        crop.height,
                        &args.charset,
                    )?;
                    let recognized = maybe_split_long_line_and_recognize(
                        recognized,
                        &args.model,
                        &crop.data,
                        crop.width,
                        crop.height,
                        &args.charset,
                        args.split_long_lines,
                        args.split_long_line_char_threshold,
                    )?;
                    maybe_rerank_line_quality(
                        recognized,
                        &args.model,
                        &args.model30,
                        &args.model50,
                        &crop.data,
                        crop.width,
                        crop.height,
                        &args.charset,
                        args.quality_boost,
                        args.quality_boost_min_delta,
                        args.split_long_lines,
                        args.split_long_line_char_threshold,
                        args.cascade_threshold_30_to_50,
                        args.cascade_threshold_50_to_100,
                    )?
                };
                if recognized.mean_confidence >= args.min_line_confidence
                    && !recognized.text.trim().is_empty()
                {
                    let text = if let Some(dict) = post_dict.as_ref() {
                        dict.apply(&recognized.text)
                    } else {
                        recognized.text
                    };
                    recognized_lines.push(RecognizedLine {
                        bbox_xyxy: [x0 as i32, y0 as i32, x1 as i32, y1 as i32],
                        text,
                        confidence: recognized.mean_confidence,
                        is_vertical: (y1.saturating_sub(y0)) > (x1.saturating_sub(x0)),
                    });
                }
            }
            sort_lines_in_reading_order(&mut recognized_lines);
            let mut lines: Vec<String> = recognized_lines.into_iter().map(|l| l.text).collect();
            if args.structure_rules {
                lines = apply_structural_rules(&lines);
            }
            if let Some(pack) = rule_pack.as_ref() {
                lines = pack.apply(&lines);
            }
            let body = lines.join("\n");
            if let Some(path) = args.output_txt.as_ref() {
                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, &body)?;
            }
            if let Some(path) = args.output_docx.as_ref() {
                save_text_as_docx(&lines, path)?;
            }
            println!("{body}");
        }
        Command::MockPage(args) => {
            let img = crate::io::load_rgb_u8(&args.image)?;
            let config = MockConfig::from_args(&args, img.height).validate()?;
            let detections = build_mock_line_detections(
                img.width,
                img.height,
                config.line_count,
                config.line_confidence,
            );
            let mut out = run_page(
                PageInput {
                    rgb: &img.data,
                    width: img.width,
                    height: img.height,
                    detections: &detections,
                },
                |_| "mock30".to_string(),
                |_| "mock50".to_string(),
                |_| config.line_text.clone(),
            )?;
            if let Some(orientation) = config.line_orientation {
                let is_vertical = matches!(orientation, LineOrientation::Vertical);
                for l in &mut out.lines {
                    l.is_vertical = is_vertical;
                }
            }
            std::fs::create_dir_all(&args.output_dir)?;
            let stem = config.output_stem.as_deref().unwrap_or_else(|| {
                args.image
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output")
            });
            let json_path = args.output_dir.join(format!("{stem}.json"));
            let xml_path = args.output_dir.join(format!("{stem}.xml"));
            let txt_path = args.output_dir.join(format!("{stem}.txt"));
            save_page_artifacts(
                &out.lines,
                &out.texts,
                img.width,
                img.height,
                &args.image.to_string_lossy(),
                args.image
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("input.jpg"),
                Path::new(&json_path),
                Path::new(&xml_path),
                Path::new(&txt_path),
            )?;
        }
    }
    Ok(())
}

#[cfg(not(feature = "onnx"))]
fn recognize_line_with_optional_cascade(
    model100: &Path,
    model30: &Path,
    model50: &Path,
    enable_cascade: bool,
    th_30_to_50: usize,
    th_50_to_100: usize,
    pred_char_count_hint: Option<f32>,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset: &Path,
) -> Result<parseq::RecognizeResult> {
    if !enable_cascade {
        return parseq::recognize_rgb_u8_with_score(model100, rgb, width, height, charset);
    }

    // Fallback to the 100-model when cascade models are unavailable.
    let model30 = if model30.is_file() { model30 } else { model100 };
    let model50 = if model50.is_file() { model50 } else { model100 };
    let pred_char_bucket = pred_char_count_hint
        .map(normalize_pred_char_bucket)
        .unwrap_or_else(|| estimate_pred_char_bucket(width, height, th_30_to_50, th_50_to_100));

    let mut out = if pred_char_bucket == 3.0 {
        parseq::recognize_rgb_u8_with_score(model30, rgb, width, height, charset)?
    } else if pred_char_bucket == 2.0 {
        parseq::recognize_rgb_u8_with_score(model50, rgb, width, height, charset)?
    } else {
        parseq::recognize_rgb_u8_with_score(model100, rgb, width, height, charset)?
    };

    if pred_char_bucket == 3.0 && out.text.chars().count() >= th_30_to_50 {
        out = parseq::recognize_rgb_u8_with_score(model50, rgb, width, height, charset)?;
    }
    if out.text.chars().count() >= th_50_to_100 {
        out = parseq::recognize_rgb_u8_with_score(model100, rgb, width, height, charset)?;
    }
    Ok(out)
}

fn normalize_pred_char_bucket(v: f32) -> f32 {
    if (v - 3.0).abs() < 0.2 {
        3.0
    } else if (v - 2.0).abs() < 0.2 {
        2.0
    } else {
        100.0
    }
}

fn estimate_line_char_count(width: usize, height: usize) -> usize {
    let h = height.max(1);
    let ratio = width as f32 / h as f32;
    // Empirical scale for Japanese page-line crops.
    (ratio * 2.5).round().max(1.0) as usize
}

fn estimate_pred_char_bucket(
    width: usize,
    height: usize,
    th_30_to_50: usize,
    th_50_to_100: usize,
) -> f32 {
    let estimated_chars = estimate_line_char_count(width, height);
    if estimated_chars <= th_30_to_50 {
        3.0
    } else if estimated_chars <= th_50_to_100 {
        2.0
    } else {
        100.0
    }
}

#[cfg(not(feature = "onnx"))]
fn maybe_split_long_line_and_recognize(
    recognized: parseq::RecognizeResult,
    model100: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset: &Path,
    split_long_lines: bool,
    split_long_line_char_threshold: usize,
) -> Result<parseq::RecognizeResult> {
    if !split_long_lines {
        return Ok(recognized);
    }
    let char_count = recognized.text.chars().count();
    if char_count < split_long_line_char_threshold || width < height.saturating_mul(4) {
        return Ok(recognized);
    }
    let mid = width / 2;
    if mid == 0 || mid >= width {
        return Ok(recognized);
    }
    let left = crop_rgb_u8(rgb, width, height, BBox::new(0, 0, mid, height))?;
    let right = crop_rgb_u8(rgb, width, height, BBox::new(mid, 0, width, height))?;
    let left_rec = parseq::recognize_rgb_u8_with_score(
        model100,
        &left.data,
        left.width,
        left.height,
        charset,
    )?;
    let right_rec = parseq::recognize_rgb_u8_with_score(
        model100,
        &right.data,
        right.width,
        right.height,
        charset,
    )?;

    let left_len = left_rec.text.chars().count();
    let right_len = right_rec.text.chars().count();
    if left_len == 0 && right_len == 0 {
        return Ok(recognized);
    }
    let text = format!("{}{}", left_rec.text, right_rec.text);
    let total_len = (left_len + right_len) as f32;
    let mean_confidence = if total_len > 0.0 {
        (left_rec.mean_confidence * left_len as f32 + right_rec.mean_confidence * right_len as f32)
            / total_len
    } else {
        recognized.mean_confidence
    };
    Ok(parseq::RecognizeResult {
        text,
        mean_confidence,
    })
}

#[cfg(not(feature = "onnx"))]
fn maybe_rerank_line_quality(
    base: parseq::RecognizeResult,
    model100: &Path,
    model30: &Path,
    model50: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset: &Path,
    quality_boost: bool,
    quality_boost_min_delta: f32,
    split_long_lines: bool,
    split_long_line_char_threshold: usize,
    cascade_th_30_to_50: usize,
    cascade_th_50_to_100: usize,
) -> Result<parseq::RecognizeResult> {
    if !quality_boost {
        return Ok(base);
    }

    let mut best = base.clone();
    let mut best_score = quality_score(&base.text, base.mean_confidence);

    let direct100 = parseq::recognize_rgb_u8_with_score(model100, rgb, width, height, charset)?;
    let direct100_score = quality_score(&direct100.text, direct100.mean_confidence);
    if direct100_score > best_score + quality_boost_min_delta {
        best = direct100.clone();
        best_score = direct100_score;
    }

    let bucket =
        estimate_pred_char_bucket(width, height, cascade_th_30_to_50, cascade_th_50_to_100);

    // 短い行（256 系モデル向け）: カスケードが 50/100 に振り替わったあとも 30 単体がマシな場合がある。
    if bucket == 3.0 && model30.is_file() {
        let direct30 = parseq::recognize_rgb_u8_with_score(model30, rgb, width, height, charset)?;
        let direct30_score = quality_score(&direct30.text, direct30.mean_confidence);
        if direct30_score > best_score + quality_boost_min_delta {
            best = direct30;
            best_score = direct30_score;
        }
    }

    // 中幅行（384 系モデル向け）: カスケードが 100 に振り替わったあとも 50 単体がマシな場合がある。
    if bucket == 2.0 && model50.is_file() {
        let direct50 = parseq::recognize_rgb_u8_with_score(model50, rgb, width, height, charset)?;
        let direct50_score = quality_score(&direct50.text, direct50.mean_confidence);
        if direct50_score > best_score + quality_boost_min_delta {
            best = direct50;
            best_score = direct50_score;
        }
    }

    let split100 = maybe_split_long_line_and_recognize(
        direct100,
        model100,
        rgb,
        width,
        height,
        charset,
        split_long_lines,
        split_long_line_char_threshold,
    )?;
    let split100_score = quality_score(&split100.text, split100.mean_confidence);
    if split100_score > best_score + quality_boost_min_delta {
        best = split100;
    }

    Ok(best)
}

fn quality_score(text: &str, mean_confidence: f32) -> f32 {
    let mut total = 0usize;
    let mut jp_like = 0usize;
    let mut digits = 0usize;
    let mut symbols = 0usize;
    let mut max_repeat = 1usize;
    let mut run = 0usize;
    let mut prev = '\0';
    let mut stack = Vec::new();
    let mut unmatched_closing = 0usize;

    for ch in text.chars() {
        total += 1;
        if is_japanese_like(ch) {
            jp_like += 1;
        }
        if ch.is_ascii_digit() {
            digits += 1;
        } else if !ch.is_alphanumeric() && !is_japanese_like(ch) && !ch.is_whitespace() {
            symbols += 1;
        }

        if ch == prev {
            run += 1;
        } else {
            run = 1;
            prev = ch;
        }
        if run > max_repeat {
            max_repeat = run;
        }

        match ch {
            '(' | '（' | '「' | '『' | '【' => stack.push(ch),
            ')' if pop_match(&mut stack, '(').is_none() => unmatched_closing += 1,
            '）' if pop_match(&mut stack, '（').is_none() => unmatched_closing += 1,
            '」' if pop_match(&mut stack, '「').is_none() => unmatched_closing += 1,
            '』' if pop_match(&mut stack, '『').is_none() => unmatched_closing += 1,
            '】' if pop_match(&mut stack, '【').is_none() => unmatched_closing += 1,
            _ => {}
        }
    }

    if total == 0 {
        return -1.0;
    }
    let lenf = total as f32;
    let jp_ratio = jp_like as f32 / lenf;
    let digit_ratio = digits as f32 / lenf;
    let symbol_ratio = symbols as f32 / lenf;
    let repeat_penalty = max_repeat.saturating_sub(3) as f32;
    let unmatched_penalty = (unmatched_closing + stack.len()) as f32;

    mean_confidence + 0.30 * jp_ratio
        - 0.18 * digit_ratio
        - 0.18 * symbol_ratio
        - 0.04 * repeat_penalty
        - 0.06 * unmatched_penalty
}

fn pop_match(stack: &mut Vec<char>, open: char) -> Option<char> {
    if stack.last().copied() == Some(open) {
        return stack.pop();
    }
    None
}

fn is_japanese_like(ch: char) -> bool {
    matches!(
        ch,
        '\u{3040}'..='\u{30ff}' // hiragana + katakana
            | '\u{3400}'..='\u{4dbf}' // CJK ext A
            | '\u{4e00}'..='\u{9fff}' // CJK unified
            | '々'
            | '。'
            | '、'
            | '「'
            | '」'
            | '（'
            | '）'
    )
}

#[cfg(test)]
mod tests {
    use super::{estimate_pred_char_bucket, quality_score};

    #[test]
    fn estimate_pred_char_bucket_returns_30_class_for_short_lines() {
        let b = estimate_pred_char_bucket(240, 64, 25, 45);
        assert!((b - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn estimate_pred_char_bucket_returns_50_class_for_medium_lines() {
        let b = estimate_pred_char_bucket(800, 64, 25, 45);
        assert!((b - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn estimate_pred_char_bucket_returns_100_class_for_long_lines() {
        let b = estimate_pred_char_bucket(1600, 64, 25, 45);
        assert!((b - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn quality_score_prefers_japanese_sentence_over_digit_noise() {
        let jp = quality_score("秘密保持契約書", 0.70);
        let noisy = quality_score("(199.866 (1999.88) (198) 196", 0.70);
        assert!(jp > noisy);
    }
}
