use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 描画元で独立した4行のRGBと、各行の画素範囲を生成する。
fn separate_stroke_rows(
    scale: usize,
    dx: usize,
    dy: usize,
    cells: usize,
    lower_y: usize,
    thickness: usize,
) -> (Vec<u8>, usize, usize, [[usize; 4]; 4]) {
    let (width, height) = ((cells * 100 + 80) * scale + dx, 300 * scale + dy);
    let mut rgb = vec![255; width * height * 3];
    let mut paint = |[x0, y0, x1, y1]: [usize; 4]| {
        for y in y0 * scale + dy..y1 * scale + dy {
            rgb[(y * width + x0 * scale + dx) * 3..(y * width + x1 * scale + dx) * 3].fill(0);
        }
    };
    paint([40, 10, 300, 40]);
    paint([40, 60, 300, 90]);
    for i in 0..cells {
        let x = 40 + i * 100;
        paint([x + 5, 130, x + 75, 130 + thickness]);
        paint([x, lower_y, x + 80, lower_y + thickness]);
    }
    let expected = [
        [40, 10, 300, 40],
        [40, 60, 300, 90],
        [45, 130, 15 + cells * 100, 130 + thickness],
        [40, lower_y, 20 + cells * 100, lower_y + thickness],
    ]
    .map(|[x0, y0, x1, y1]| {
        [
            x0 * scale + dx,
            y0 * scale + dy,
            x1 * scale + dx,
            y1 * scale + dy,
        ]
    });
    (rgb, width, height, expected)
}

/// 左右に揃った横線2行でも、大字形のように結合せず元の4帯を保持する。
#[test]
fn aligned_stroke_rows_keep_their_separate_regions() {
    let mut reports = Vec::new();
    for cells in 3..=5 {
        for lower_y in [200, 210, 220] {
            for thickness in [10, 15] {
                let (rgb, w, h, expected) =
                    separate_stroke_rows(1, 0, 0, cells, lower_y, thickness);
                let bands = detect_textline_bands_naive(&rgb, w, h, 220);
                let fast = detect_textline_bands_fast(&rgb, w, h, 220);
                let accepted = bands.len() == 4
                    && bands.iter().zip(expected).all(|(bounds, ink)| {
                        bounds[0] <= ink[0]
                            && bounds[1] <= ink[1]
                            && bounds[2] >= ink[2]
                            && bounds[3] >= ink[3]
                    })
                    && bands.windows(2).all(|pair| pair[0][3] <= pair[1][1])
                    && bands == fast;
                reports.push(serde_json::json!({
                    "cells":cells,"lower_y":lower_y,"thickness":thickness,
                    "expected_ink":expected,"naive":bands,"fast":fast,"accepted":accepted,
                }));
            }
        }
    }
    println!("STROKE_ROWS_REPORT={}", serde_json::json!(reports));
    assert!(reports.iter().all(|r| r["accepted"] == true));
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大・移動・線数・厚み・行間が変わっても、罫線の行と元座標を保持する。
    #[test]
    fn stroke_rows_preserve_scale_translation_and_coverage(
        scale in 1usize..4, dx in 0usize..20, dy in 0usize..20,
        cells in 3usize..6, lower_y in 200usize..221, thickness in 10usize..16,
    ) {
        let (rgb,w,h,expected)=separate_stroke_rows(scale,dx,dy,cells,lower_y,thickness);
        let bands=detect_textline_bands_naive(&rgb,w,h,220);
        proptest::prop_assert_eq!(bands.len(),4,"{:?}",bands);
        for (bounds, ink) in bands.iter().zip(expected) {
            proptest::prop_assert!(bounds[0]<=ink[0] && bounds[1]<=ink[1]);
            proptest::prop_assert!(bounds[2]>=ink[2] && bounds[3]>=ink[3]);
        }
        proptest::prop_assert!(bands.windows(2).all(|pair| pair[0][3] <= pair[1][1]));
        proptest::prop_assert_eq!(&bands,&detect_textline_bands_fast(&rgb,w,h,220));
    }
}
