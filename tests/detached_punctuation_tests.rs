use ndlocr_lite_rs::pipeline::{
    line_segment::{detect_textline_bands_fast, detect_textline_bands_naive},
    paper_background::normalize_paper_shading,
};
use proptest::prelude::*;

/// 半開矩形を黒で埋める。
fn rect(rgb: &mut [u8], width: usize, bounds: [usize; 4]) {
    for y in bounds[1]..bounds[3] {
        for x in bounds[0]..bounds[2] {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(0);
        }
    }
}

/// 原画像と独立したL字5文字と、離れた輪郭状の句点を描く。
fn page(scale: usize, dx: usize, dy: usize, mark: [usize; 4]) -> (Vec<u8>, usize, usize) {
    let (width, height) = (260 * scale + dx, 150 * scale + dy);
    let mut rgb = vec![255; width * height * 3];
    let transform = |b: [usize; 4]| {
        [
            b[0] * scale + dx,
            b[1] * scale + dy,
            b[2] * scale + dx,
            b[3] * scale + dy,
        ]
    };
    for x in [20, 50, 80, 110, 140] {
        rect(&mut rgb, width, transform([x, 20, x + 3, 52]));
        rect(&mut rgb, width, transform([x, 46, x + 20, 52]));
    }
    let [x0, y0, x1, y1] = mark;
    for b in [
        [x0, y0, x1, y0 + 1],
        [x0, y1 - 1, x1, y1],
        [x0, y0, x0 + 1, y1],
        [x1 - 1, y0, x1, y1],
    ] {
        rect(&mut rgb, width, transform(b));
    }
    (rgb, width, height)
}

/// 目視固定した原画素の5句読点を、それぞれの本文帯が含む。
#[test]
fn prepared_handwriting_keeps_all_five_trailing_marks() {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/paper_shading/page.png",
    ))
    .unwrap();
    let rgb = normalize_paper_shading(&im.data, im.width, im.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, im.width, im.height, 220);
    eprintln!("DETACHED BANDS {bands:?}");
    assert_eq!(bands.len(), 5);
    assert_eq!(
        bands[0][2], 582,
        "must not chase background ink beyond the first row"
    );
    for (band, mark) in bands.iter().zip([
        [434, 57, 439, 64],
        [547, 142, 552, 150],
        [555, 245, 560, 254],
        [570, 335, 575, 342],
        [615, 434, 624, 445],
    ]) {
        assert!(
            band[0] <= mark[0] && band[1] <= mark[1] && band[2] >= mark[2] && band[3] >= mark[3],
            "band {band:?} excludes {mark:?}"
        );
    }
    assert_eq!(
        bands,
        detect_textline_bands_fast(&rgb, im.width, im.height, 220)
    );
}

/// 長い行の右下に離れた句点を、その行へ一度だけ含める。
#[test]
fn includes_sparse_period_below_the_trailing_body() {
    let (rgb, w, h) = page(1, 0, 0, [177, 55, 183, 63]);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands.len(), 1);
    assert!(bands[0][2] >= 183 && bands[0][3] >= 63, "{bands:?}");
}

/// 上側、遠方、次行位置、塗り潰し、小点、長い線は行末の句読点にしない。
#[test]
fn rejects_unrelated_marks_and_solid_noise() {
    for (mark, fill) in [
        ([177, 25, 183, 33], false),
        ([215, 55, 221, 63], false),
        ([177, 90, 183, 98], false),
        ([177, 55, 185, 63], true),
        ([177, 55, 180, 58], true),
        ([177, 55, 181, 80], false),
    ] {
        let (mut rgb, w, h) = page(1, 0, 0, mark);
        if fill {
            rect(&mut rgb, w, mark);
        }
        let bands = detect_textline_bands_naive(&rgb, w, h, 220);
        let right = if mark[1] == 25 { 185 } else { 162 };
        assert_eq!(bands[0], [18, 19, right, 53], "mark {mark:?}");
    }
}

/// 疎な余白ノイズで左右境界が広がっても、それを足場に点を回収しない。
#[test]
fn sparse_margin_ink_is_not_a_body_anchor() {
    let (mut rgb, w, h) = page(1, 0, 0, [236, 55, 242, 63]);
    rect(&mut rgb, w, [218, 47, 222, 52]);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands[0], [18, 19, 224, 53]);
}

/// 単一字形の短い帯を句読点の文脈として利用しない。
#[test]
fn short_glyph_is_not_a_sentence_anchor() {
    let (mut rgb, w, h) = page(1, 0, 0, [177, 55, 183, 63]);
    for y in 20..52 {
        rgb[(y * w + 20) * 3..(y * w + 130) * 3].fill(255);
    }
    rect(&mut rgb, w, [140, 20, 148, 52]);
    assert_eq!(
        detect_textline_bands_naive(&rgb, w, h, 220),
        vec![[138, 19, 162, 53]]
    );
}

/// 次行の上端へかかる小成分を前行の句読点として取り込まない。
#[test]
fn does_not_cross_the_next_body() {
    let (mut rgb, w, h) = page(1, 0, 0, [177, 61, 183, 69]);
    for x in [20, 50, 80, 110, 140] {
        rect(&mut rgb, w, [x, 66, x + 3, 98]);
        rect(&mut rgb, w, [x, 92, x + 20, 98]);
    }
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands.len(), 2);
    assert_eq!(bands[0], [18, 19, 162, 53]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 拡大と移動でも句点を保持し、両検出器は同じ画像内の境界を返す。
    #[test]
    fn translated_scaled_periods_remain_in_their_body(scale in 1usize..4, dx in 0usize..20, dy in 0usize..20) {
        let (rgb,w,h) = page(scale,dx,dy,[177,55,183,63]);
        let bands = detect_textline_bands_naive(&rgb,w,h,220);
        prop_assert_eq!(bands.len(),1);
        let b = bands[0];
        prop_assert!(b[0] <= 20*scale+dx && b[1] <= 20*scale+dy);
        prop_assert!(b[2] >= 183*scale+dx && b[3] >= 63*scale+dy);
        prop_assert!(b[2] <= w && b[3] <= h);
        prop_assert_eq!(bands,detect_textline_bands_fast(&rgb,w,h,220));
    }
}
