//! 孤立した行頭小記号を、モデルに収まる本文の文脈へ戻す。
use super::{BlankGap, LineChunk};

/// 認識文字を変更せず、行頭小記号と次の本文区間をまとめる。
pub(super) fn restore_leading_symbol(
    rgb: &[u8],
    size: (usize, usize),
    threshold: u8,
    chunks: &mut Vec<LineChunk>,
    gaps: &[BlankGap],
    input: (usize, usize),
) {
    let (width, height) = size;
    let Some((left, right)) = chunks.first().zip(chunks.get(1)) else {
        return;
    };
    if height < 16
        || left.x0 != 0
        || left.x1 != right.x0
        || !right.space_before
        || left.x1 as u128 * 2 > height as u128 * 3
        || right.x1 as u128 * input.1 as u128 > height as u128 * input.0 as u128
        || gaps.partition_point(|gap| gap.midpoint < right.x1) >= super::MAX_CHUNK_INK_RUNS
    {
        return;
    }
    let ink = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < u16::from(threshold) * 3
    };
    let (mut x0, mut y0, mut x1, mut y1, mut area) = (left.x1, height, 0, 0, 0usize);
    let (mut body_top, mut body_bottom) = (height, 0);
    for y in 0..height {
        for x in 0..left.x1 {
            if ink(x, y) {
                x0 = x0.min(x);
                x1 = x1.max(x + 1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
                area += 1;
            }
        }
        for x in right.x0..right.x1 {
            if ink(x, y) {
                body_top = body_top.min(y);
                body_bottom = body_bottom.max(y + 1);
            }
        }
    }
    if area < 3
        || x1 - x0 > height / 2
        || y1 - y0 > height / 2
        || body_bottom.saturating_sub(body_top) < height - height / 4
    {
        return;
    }
    let mark_width = x1 - x0;
    let mut visited = vec![false; mark_width * (y1 - y0)];
    let seed = (x0..x1).find(|&x| ink(x, y0)).unwrap();
    let mut pending = vec![(seed, y0)];
    visited[seed - x0] = true;
    let mut connected = 0;
    while let Some((x, y)) = pending.pop() {
        connected += 1;
        for ny in y.saturating_sub(1).max(y0)..=(y + 1).min(y1 - 1) {
            for nx in x.saturating_sub(1).max(x0)..=(x + 1).min(x1 - 1) {
                let index = (ny - y0) * mark_width + nx - x0;
                if !visited[index] && ink(nx, ny) {
                    visited[index] = true;
                    pending.push((nx, ny));
                }
            }
        }
    }
    if connected == area {
        chunks[0].x1 = chunks[1].x1;
        chunks.remove(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 24px行で、丸点と通常本文を別々の区間へ配置する。
    fn sample() -> (Vec<u8>, Vec<LineChunk>) {
        let mut rgb = vec![255; 360 * 24 * 3];
        for (xs, ys) in [(2..10, 9..17), (36..175, 2..22), (190..350, 2..22)] {
            for y in ys {
                for x in xs.clone() {
                    rgb[(y * 360 + x) * 3..(y * 360 + x + 1) * 3].fill(0);
                }
            }
        }
        let chunks = [
            (0, 24, false, 0),
            (24, 180, true, 26),
            (180, 360, false, 10),
        ]
        .into_iter()
        .map(|(x0, x1, space_before, gap_before)| LineChunk {
            x0,
            x1,
            space_before,
            gap_before,
        })
        .collect();
        (rgb, chunks)
    }

    /// 最初の小記号を本文へ戻し、後続境界と全画素を保つ。
    #[test]
    fn leading_symbol_joins_fitting_body_context() {
        let (rgb, mut chunks) = sample();
        let gaps = super::super::blank_gaps(&rgb, 360, 24, 220);
        restore_leading_symbol(&rgb, (360, 24), 220, &mut chunks, &gaps, (256, 24));
        assert_eq!(chunks.len(), 2);
        assert_eq!((chunks[0].x0, chunks[0].x1), (0, 180));
        assert_eq!((chunks[1].x0, chunks[1].x1), (180, 360));
        assert!(!chunks[1].space_before);
        let mut pixels = Vec::new();
        // 各行について区間を順に並べ直すと元画像になる。
        for y in 0..24 {
            for chunk in &chunks {
                pixels.extend_from_slice(&rgb[(y * 360 + chunk.x0) * 3..(y * 360 + chunk.x1) * 3]);
            }
        }
        assert_eq!(pixels, rgb);
    }

    /// 普通の先頭文字や離れた小画線を、単独小記号として扱わない。
    #[test]
    fn unsupported_context_keeps_original_boundaries() {
        for case in 0..6 {
            let (mut rgb, mut chunks) = sample();
            match case {
                0 => {
                    for y in 1..23 {
                        for x in 2..10 {
                            rgb[(y * 360 + x) * 3..(y * 360 + x + 1) * 3].fill(0);
                        }
                    }
                }
                1 => rgb[(4 * 360 + 4) * 3..(4 * 360 + 5) * 3].fill(0),
                2 => chunks[1].space_before = false,
                3 => chunks[1].x0 += 1,
                4 => {
                    for y in 2..22 {
                        for x in 36..175 {
                            rgb[(y * 360 + x) * 3..(y * 360 + x + 1) * 3].fill(255);
                        }
                    }
                }
                5 => chunks[0].x0 = 1,
                _ => unreachable!(),
            }
            let original: Vec<_> = chunks
                .iter()
                .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                .collect();
            let gaps = super::super::blank_gaps(&rgb, 360, 24, 220);
            restore_leading_symbol(&rgb, (360, 24), 220, &mut chunks, &gaps, (256, 24));
            assert_eq!(
                chunks
                    .iter()
                    .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                    .collect::<Vec<_>>(),
                original,
                "case={case}"
            );
        }
    }

    /// 公開実画像の2つの丸点を、それぞれ後続本文と同じ区間へ入れる。
    #[test]
    fn jma_bullets_keep_following_body_context() {
        let image = image::load_from_memory(include_bytes!(
            "../../../tests/fixtures/line_segment/jma-notice-body.png"
        ))
        .unwrap()
        .to_rgb8();
        let (w, h) = (image.width() as usize, image.height() as usize);
        let bands =
            crate::pipeline::line_segment::detect_textline_bands_naive(image.as_raw(), w, h, 220);
        for target in [1013, 1128] {
            let b = bands.iter().find(|b| b[1].abs_diff(target) < 5).unwrap();
            let crop = crate::pipeline::crop::crop_rgb_u8(
                image.as_raw(),
                w,
                h,
                crate::pipeline::crop::BBox::new(b[0], b[1], b[2], b[3]),
            )
            .unwrap();
            let chunks = super::super::split_line_for_input(
                &crop.data,
                crop.width,
                crop.height,
                220,
                256,
                24,
            )
            .unwrap();
            assert!(chunks[0].x1 > crop.height * 4, "isolated mark: {chunks:?}");
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(24))]
        /// 拡大・余白移動後も、実モデル容量の境界でだけ結合し、全画素を保存する。
        #[test]
        fn scaled_context_respects_model_capacity(scale in 1usize..4, margin in 0usize..6, model_width in 178usize..188) {
            let (source,mut chunks)=sample();
            let (w,h)=(360*scale+margin,24*scale);
            let mut rgb=vec![255;w*h*3];
            for y in 0..h { for x in 0..360*scale {
                let to=(y*w+x+margin)*3;
                let from=((y/scale)*360+x/scale)*3;
                rgb[to..to+3].copy_from_slice(&source[from..from+3]);
            }}
            for c in &mut chunks {
                c.x0=c.x0*scale+margin;
                c.x1=c.x1*scale+margin;
                c.gap_before*=scale;
            }
            chunks[0].x0=0;
            let gaps=super::super::blank_gaps(&rgb,w,h,220);
            let fits=(180*scale+margin)*24<=h*model_width;
            restore_leading_symbol(&rgb,(w,h),220,&mut chunks,&gaps,(model_width,24));
            proptest::prop_assert_eq!(chunks.len(),if fits {2} else {3});
            proptest::prop_assert_eq!(chunks[0].x0,0);
            proptest::prop_assert_eq!(chunks.last().unwrap().x1,w);
            proptest::prop_assert!(chunks.windows(2).all(|cs|cs[0].x1==cs[1].x0));
            // 結合する関数は不変RGB参照しか受けず、区間の和集合は元画像全体。
            proptest::prop_assert_eq!(chunks.iter().map(|c|c.x1-c.x0).sum::<usize>(),w);
        }
    }
}
