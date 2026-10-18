use ndlocr_lite_rs::pipeline::line_chunks::split_rotated_vertical_column_for_input;
use proptest::prelude::*;

/// 白列を挟む縦書き字形を、回転後の座標で作る。
fn glyphs(count: usize, advance: usize, height: usize) -> Vec<u8> {
    let width = count * advance;
    let mut rgb = vec![255; width * height * 3];
    for n in 0..count {
        for y in 0..height {
            for x in n * advance..(n + 1) * advance - 2 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    rgb
}

/// 文字数上限内の縦列は、横書きの単語間隔で小断片へ分解しない。
#[test]
fn preserves_short_vertical_context_and_rejects_unsplittable_columns() {
    let rgb = glyphs(18, 20, 20);
    let chunks = split_rotated_vertical_column_for_input(&rgb, 360, 20, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!((chunks[0].x0, chunks[0].x1), (0, 360));
    assert!(
        split_rotated_vertical_column_for_input(&vec![0; 500 * 20 * 3], 500, 20, 220, 256, 24)
            .is_err()
    );
    for (rgb, w, h, iw, ih) in [
        (&[][..], 0, 1, 256, 24),
        (&[0; 3][..], 1, 1, 0, 24),
        (&[][..], usize::MAX, 2, 256, 24),
    ] {
        assert!(split_rotated_vertical_column_for_input(rgb, w, h, 220, iw, ih).is_err());
    }
}

proptest! {
    /// 長い列でも本文画素を切らず全範囲を覆い、字数・比率を制限して空白を捏造しない。
    #[test]
    fn chunks_cover_all_pixels_without_inventing_word_spaces(count in 21usize..100, advance in 5usize..20) {
        let height = 20;
        let width = count * advance;
        let rgb = glyphs(count, advance, height);
        let chunks = split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
        prop_assert!(chunks.len() >= 2);
        prop_assert_eq!(chunks[0].x0, 0);
        prop_assert_eq!(chunks.last().unwrap().x1, width);
        for (i, chunk) in chunks.iter().enumerate() {
            prop_assert!(!chunk.space_before);
            prop_assert!(chunk.x0 < chunk.x1 && chunk.x1 - chunk.x0 <= 400);
            if i > 0 {
                prop_assert_eq!(chunks[i-1].x1, chunk.x0);
                prop_assert!((0..height).all(|y| rgb[(y*width+chunk.x0)*3] == 255));
            }
        }
    }
}

/// 既知の描画矩形は検証だけに使い、二・三を途中で切断しない。
#[test]
fn long_rendered_columns_do_not_split_glyph_interiors() {
    use ndlocr_lite_rs::pipeline::vertical_segment::rotate_vertical_column_rgb_u8;
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/long_vertical/manifest.json")).unwrap();
    for item in manifest["items"].as_array().unwrap() {
        let path = std::path::Path::new("tests/fixtures/long_vertical")
            .join(item["image"].as_str().unwrap());
        let im = ndlocr_lite_rs::io::load_rgb_u8(&path).unwrap();
        let r = rotate_vertical_column_rgb_u8(&im.data, im.width, im.height).unwrap();
        let chunks =
            split_rotated_vertical_column_for_input(&r.data, r.width, r.height, 220, 256, 24)
                .unwrap();
        assert!(chunks.len() >= 2);
        for chunk in chunks.iter().skip(1) {
            for bbox in item["glyph_boxes"].as_array().unwrap() {
                let top = bbox[1].as_u64().unwrap() as usize;
                let bottom = bbox[3].as_u64().unwrap() as usize;
                assert!(
                    chunk.x0 <= top || chunk.x0 >= bottom,
                    "{}: split {} inside [{top}, {bottom})",
                    item["id"],
                    chunk.x0
                );
            }
        }
    }
}

/// 既知の描画矩形は検証だけに使い、二・三を途中で切断しない。
#[test]
fn detected_long_columns_do_not_split_glyph_interiors() {
    use ndlocr_lite_rs::pipeline::vertical_segment::rotate_vertical_column_rgb_u8;
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/long_vertical/manifest.json")).unwrap();
    for item in manifest["items"].as_array().unwrap() {
        let path = std::path::Path::new("tests/fixtures/long_vertical")
            .join(item["image"].as_str().unwrap());
        let im = ndlocr_lite_rs::io::load_rgb_u8(&path).unwrap();
        let columns = ndlocr_lite_rs::pipeline::vertical_segment::detect_vertical_textline_columns(
            &im.data, im.width, im.height, 220,
        )
        .unwrap();
        assert_eq!(columns.len(), 1);
        let [x0, y0, x1, y1] = columns[0];
        let cropped = ndlocr_lite_rs::pipeline::crop::crop_rgb_u8(
            &im.data,
            im.width,
            im.height,
            ndlocr_lite_rs::pipeline::crop::BBox::new(x0, y0, x1, y1),
        )
        .unwrap();
        let r =
            rotate_vertical_column_rgb_u8(&cropped.data, cropped.width, cropped.height).unwrap();
        eprintln!(
            "{}: column {:?}, rotated height {}",
            item["id"], columns[0], r.height
        );
        let chunks =
            split_rotated_vertical_column_for_input(&r.data, r.width, r.height, 220, 256, 24)
                .unwrap();
        assert!(chunks.len() >= 2);
        for chunk in chunks.iter().skip(1) {
            for bbox in item["glyph_boxes"].as_array().unwrap() {
                let top = bbox[1].as_u64().unwrap() as usize;
                let bottom = bbox[3].as_u64().unwrap() as usize;
                assert!(
                    chunk.x0 + y0 <= top || chunk.x0 + y0 >= bottom,
                    "{}: split {} inside [{top}, {bottom})",
                    item["id"],
                    chunk.x0 + y0
                );
            }
        }
    }
}

/// 一定字送りの中央だけ三本の離れた画線とし、左右移動・反転も行う。
fn disconnected_center(
    count: usize,
    scale: usize,
    padding: usize,
    reflected: bool,
) -> (Vec<u8>, usize, usize, (usize, usize)) {
    let height = 34 * scale;
    let width = count * 26 * scale + padding * 2;
    let mut rgb = vec![255; width * height * 3];
    let center = count / 2;
    for n in 0..count {
        for dx in 0..20 * scale {
            if n == center
                && !(dx < 2 * scale
                    || (7 * scale..9 * scale).contains(&dx)
                    || (14 * scale..16 * scale).contains(&dx))
            {
                continue;
            }
            let x = padding + n * 26 * scale + dx;
            let x = if reflected { width - 1 - x } else { x };
            for y in 4 * scale..30 * scale {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    let start = padding + center * 26 * scale;
    let end = start + 16 * scale;
    let bounds = if reflected {
        (width - end, width - start)
    } else {
        (start, end)
    };
    (rgb, width, height, bounds)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 等間隔列の三本画線を拡大・移動・反転しても、白帯で字形を分断しない。
    #[test]
    fn disconnected_strokes_stay_together(scale in 1usize..4, padding in 0usize..15, half_count in 10usize..18, reflected in any::<bool>()) {
        let (rgb, width, height, (start, end)) = disconnected_center(half_count * 2 + 1, scale, padding, reflected);
        let chunks = split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
        prop_assert!(chunks.len() >= 2);
        prop_assert_eq!(chunks[0].x0, 0);
        prop_assert_eq!(chunks.last().unwrap().x1, width);
        for chunk in chunks.iter().skip(1) {
            prop_assert!(chunk.x0 <= start || chunk.x0 >= end, "split {} inside [{}, {})",chunk.x0,start,end);
            prop_assert!(!chunk.space_before);
        }
    }
}

/// 独立した一を並べた列は、離れた画線を一文字へ集約してモデル上限を超えない。
#[test]
fn separate_single_strokes_still_obey_the_group_limit() {
    let height = 34;
    let width = 31 * 26;
    let mut rgb = vec![255; width * height * 3];
    for n in 0..31 {
        for y in 4..30 {
            for x in n * 26..n * 26 + 2 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    let chunks =
        split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 2);
    for chunk in chunks {
        let groups = (0..31)
            .filter(|&n| chunk.x0 <= n * 26 && n * 26 < chunk.x1)
            .count();
        assert!(groups <= 20);
        assert!(!chunk.space_before);
    }
}

/// 細線の長い列は、支持字形の有無にかかわらず無制限に一群へ連鎖しない。
#[test]
fn dense_stroke_chains_do_not_bypass_chunk_limits() {
    for anchors in [0, 6] {
        let height = 34;
        let width = anchors * 26 + 61 * 8;
        let mut rgb = vec![255; width * height * 3];
        for n in 0..anchors + 61 {
            let (start, end) = if n < anchors {
                (n * 26, n * 26 + 20)
            } else {
                let start = anchors * 26 + (n - anchors) * 8;
                (start, start + 2)
            };
            for y in 4..30 {
                for x in start..end {
                    rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                }
            }
        }
        let chunks =
            split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
        assert_eq!(
            chunks.len(),
            if anchors == 0 { 4 } else { 2 },
            "anchors={anchors}"
        );
    }
}
