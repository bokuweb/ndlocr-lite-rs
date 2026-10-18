use ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive;
use ndlocr_lite_rs::pipeline::paper_background::normalize_paper_shading;
use std::borrow::Cow;

/// 緩い二次元の影と、1pxの細い横線・縦線・句読点を用意する。
fn shaded_paper() -> (Vec<u8>, Vec<usize>) {
    let (width, height) = (320, 192);
    let mut rgb = vec![0; width * height * 3];
    for y in 0..height {
        for x in 0..width {
            let paper = 200 + (x * 35 / width + y * 15 / height) as u8;
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(paper);
        }
    }
    let ink: Vec<_> = (32..280)
        .map(|x| 48 * width + x)
        .chain((24..168).map(|y| y * width + 40))
        .chain([100 * width + 290, 101 * width + 291])
        .collect();
    for &i in &ink {
        let paper = rgb[i * 3];
        rgb[i * 3..i * 3 + 3].fill(paper - 40);
    }
    (rgb, ink)
}

/// 背景が閾値を越え、薄い細線・縦線・句読点は同じ座標で残る。
#[test]
fn smooth_shadow_is_removed_without_losing_thin_ink() {
    let (rgb, ink) = shaded_paper();
    let out = normalize_paper_shading(&rgb, 320, 192).unwrap();
    assert_eq!(out.len(), rgb.len());
    for i in 0..320 * 192 {
        if ink.contains(&i) {
            assert!(out[i * 3] < 220, "lost ink at {i}");
        } else {
            assert!(out[i * 3] >= 220, "shadow at {i}: {}", out[i * 3]);
        }
    }
}

/// 原画像の5行をそれぞれの縦位置で分離し、帯数だけの一致を許さない。
#[test]
fn upstream_handwriting_keeps_five_separate_body_regions() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/paper_shading/page.png",
    ))
    .unwrap();
    let out = normalize_paper_shading(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&out, image.width, image.height, 220);
    assert_eq!(bands.len(), 5, "{bands:?}");
    for (band, (top, bottom)) in
        bands
            .iter()
            .zip([(0, 80), (85, 165), (180, 265), (275, 355), (365, 451)])
    {
        assert!(band[1] >= top && band[3] <= bottom, "{bands:?}");
        assert!(band[3] - band[1] >= 30, "truncated glyphs: {bands:?}");
    }
}

/// 白い紙の文字・罫線、色面、灰色パネル、密な写真状画素は変更しない。
#[test]
fn ordinary_paper_panels_and_dense_texture_remain_borrowed() {
    let (w, h) = (320, 192);
    let mut white = vec![255; w * h * 3];
    for x in 8..312 {
        white[(40 * w + x) * 3..(40 * w + x) * 3 + 3].fill(170);
    }
    let color = [60, 150, 210].repeat(w * h);
    let mut gray_panel = white.clone();
    for y in 30..160 {
        for x in 30..290 {
            gray_panel[(y * w + x) * 3..(y * w + x) * 3 + 3].fill(200);
        }
    }
    let texture: Vec<_> = (0..w * h)
        .flat_map(|i| [if (i / w + i % w) % 2 == 0 { 90 } else { 230 }; 3])
        .collect();
    for rgb in [white, color, gray_panel, texture] {
        let out = normalize_paper_shading(&rgb, w, h).unwrap();
        assert!(matches!(out, Cow::Borrowed(_)));
        assert_eq!(&*out, &rgb);
    }
}

/// 空軸・算術overflow・RGB長の不一致をアクセス前に拒否する。
#[test]
fn invalid_dimensions_are_rejected() {
    for (rgb, w, h) in [(&[][..], 0, 1), (&[][..], usize::MAX, 2), (&[0][..], 1, 1)] {
        assert!(normalize_paper_shading(rgb, w, h).is_err());
    }
}

/// 紙とRGB平均が同じ色の注記も、色成分の差を残して消さない。
#[test]
fn equal_brightness_colored_annotation_is_not_erased() {
    let (mut rgb, _) = shaded_paper();
    let i = (100 * 320 + 160) * 3;
    let paper = rgb[i];
    rgb[i..i + 3].copy_from_slice(&[paper - 24, paper + 24, paper]);
    let out = normalize_paper_shading(&rgb, 320, 192).unwrap();
    assert!(
        out[i] < 220,
        "colored annotation erased: {:?}",
        &out[i..i + 3]
    );
}

/// 紙より一成分だけ明るい色文字も、負方向の差だけで白く消さない。
#[test]
fn brighter_colored_annotation_is_not_erased() {
    let (mut rgb, _) = shaded_paper();
    let i = (100 * 320 + 160) * 3;
    let paper = rgb[i];
    rgb[i..i + 3].copy_from_slice(&[paper, paper + 24, paper]);
    let out = normalize_paper_shading(&rgb, 320, 192).unwrap();
    assert!(
        out[i] < 220,
        "bright annotation erased: {:?}",
        &out[i..i + 3]
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 紙の明度・薄い筆画の濃さ・左右の移動が変わっても、背景と1px画線を分ける。
    #[test]
    fn thin_strokes_survive_brightness_and_translation(
        base in 184u8..=204, contrast in 20u8..=60, offset in 0usize..12,
    ) {
        let (width, height) = (320, 192);
        let mut rgb = vec![0; width * height * 3];
        for y in 0..height { for x in 0..width {
            let paper = base + (x * 35 / width + y * 15 / height) as u8;
            let value = if x == 40 + offset && (24..168).contains(&y) { paper - contrast } else { paper };
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(value);
        }}
        let out = normalize_paper_shading(&rgb, width, height).unwrap();
        for y in 0..height { for x in 0..width {
            let ink = x == 40 + offset && (24..168).contains(&y);
            proptest::prop_assert_eq!(out[(y * width + x) * 3] < 220, ink, "at {},{}", x, y);
        }}
    }
}
