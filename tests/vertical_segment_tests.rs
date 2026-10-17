use ndlocr_lite_rs::pipeline::{
    paper_background::normalize_paper_shading, vertical_segment::detect_vertical_textline_columns,
};
use proptest::prelude::*;

/// 初回推論前に固定した6列を元座標の右から左へ保持する。
#[test]
fn prepared_handwriting_separates_six_vertical_columns() {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/vertical_handwriting/page.png",
    ))
    .unwrap();
    let rgb = normalize_paper_shading(&im.data, im.width, im.height).unwrap();
    let columns = detect_vertical_textline_columns(&rgb, im.width, im.height, 220).unwrap();
    eprintln!("VERTICAL COLUMNS {columns:?}");
    assert_eq!(columns.len(), 6);
    // 目視固定した列中心を全て異なる列に含め、元画像外へ出ない。
    for (column, x) in columns.iter().zip([515, 425, 325, 225, 125, 40]) {
        assert!(column[0] <= x && x < column[2], "{column:?} misses {x}");
        assert!(column[0] < column[2] && column[2] <= im.width);
        assert!(column[1] < column[3] && column[3] <= im.height);
    }
    assert!(columns.windows(2).all(|c| c[0][0] >= c[1][2]));
    let truth = include_str!("fixtures/vertical_handwriting/truth.txt");
    assert_eq!(
        (
            truth.lines().count(),
            truth.trim_end_matches('\n').chars().count()
        ),
        (6, 83)
    );
}

/// 半開矩形を黒で埋める。
fn rect(rgb: &mut [u8], width: usize, bounds: [usize; 4]) {
    for y in bounds[1]..bounds[3] {
        for x in bounds[0]..bounds[2] {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(0);
        }
    }
}

/// 長さの違う二列を右→左に返す。既存検出の行端2px・行幅1pxの余白も元座標へ戻す。
#[test]
fn synthetic_columns_use_original_coordinates_and_reading_order() {
    let (w, h) = (160, 200);
    let mut rgb = vec![255; w * h * 3];
    rect(&mut rgb, w, [110, 20, 130, 170]);
    rect(&mut rgb, w, [30, 40, 50, 140]);
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        [[109, 18, 131, 172], [29, 38, 51, 142]]
    );
}

/// 空・長さ不一致・積のoverflowを推論や画像確保の前に拒否する。
#[test]
fn invalid_rgb_dimensions_are_rejected() {
    for (rgb, w, h) in [
        (&[][..], 0, 0),
        (&[][..], 0, 1),
        (&[][..], 1, 0),
        (&[][..], usize::MAX, 1),
        (&[][..], 1, usize::MAX),
        (&[255, 255][..], 1, 1),
        (&[255, 255, 255, 255][..], 1, 1),
    ] {
        assert!(detect_vertical_textline_columns(rgb, w, h, 220).is_err());
    }
}

/// 最初の列の右下に離れた句点も、同じ元座標の列へ含める。
#[test]
fn vertical_handwriting_keeps_detached_first_period() {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/vertical_handwriting/page.png",
    ))
    .unwrap();
    let rgb = normalize_paper_shading(&im.data, im.width, im.height).unwrap();
    let columns = detect_vertical_textline_columns(&rgb, im.width, im.height, 220).unwrap();
    assert_eq!(columns.len(), 6);
    // 推論文字列ではなく、元画像を目視した句点の画素範囲。
    let [x0, y0, x1, y1] = columns[0];
    assert!(
        x0 <= 537 && y0 <= 349 && x1 >= 545 && y1 >= 358,
        "{:?}",
        columns[0]
    );
}

/// 縦書き本文の右下の輪郭状句点を、横書き用の上側ノイズとして捨てない。
#[test]
fn includes_sparse_period_to_the_right_of_column_tail() {
    let (w, h) = (150, 260);
    let mut rgb = vec![255; w * h * 3];
    for y in [20, 50, 80, 110, 140] {
        rect(&mut rgb, w, [20, y, 52, y + 3]);
        rect(&mut rgb, w, [46, y, 52, y + 20]);
    }
    for mark in [
        [55, 177, 63, 178],
        [55, 182, 63, 183],
        [55, 177, 56, 183],
        [62, 177, 63, 183],
    ] {
        rect(&mut rgb, w, mark);
    }
    let columns = detect_vertical_textline_columns(&rgb, w, h, 220).unwrap();
    assert_eq!(columns.len(), 1);
    assert!(columns[0][2] >= 63 && columns[0][3] >= 183, "{columns:?}");
}

/// 白紙から本文列を捏造しない。
#[test]
fn white_page_has_no_columns() {
    assert!(
        detect_vertical_textline_columns(&vec![255; 100 * 150 * 3], 100, 150, 220)
            .unwrap()
            .is_empty()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// 拡大・移動しても右→左順と元画像上の半開矩形・余白を保つ。
    #[test]
    fn scaling_and_translation_preserve_original_bounds(scale in 1usize..4, dx in 0usize..13, dy in 0usize..13) {
        let (w, h) = (160 * scale + dx, 200 * scale + dy);
        let mut rgb = vec![255; w * h * 3];
        let bodies = [[110, 20, 130, 170], [30, 40, 50, 140]].map(|[x0,y0,x1,y1]| [x0*scale+dx,y0*scale+dy,x1*scale+dx,y1*scale+dy]);
        for body in bodies { rect(&mut rgb, w, body); }
        let columns = detect_vertical_textline_columns(&rgb, w, h, 220).unwrap();
        let expected = bodies.map(|[x0,y0,x1,y1]| [x0-1,y0-2,x1+1,y1+2]);
        prop_assert_eq!(columns, expected);
    }

    /// 有効寸法より1byte短い入力も安全に拒否する。
    #[test]
    fn truncated_rgb_is_rejected(w in 1usize..40, h in 1usize..40) {
        prop_assert!(detect_vertical_textline_columns(&vec![255; w*h*3-1], w, h, 220).is_err());
    }

    /// 小画像・端の画素・任意閾値でも元画像内の有効矩形だけを返す。
    #[test]
    fn small_valid_pages_keep_bounds_in_image(w in 1usize..35, h in 1usize..35, seed in any::<u8>(), threshold in any::<u8>()) {
        let rgb: Vec<_> = (0..w*h*3).map(|i| seed.wrapping_add(i as u8)).collect();
        let columns = detect_vertical_textline_columns(&rgb, w, h, threshold).unwrap();
        for &[x0,y0,x1,y1] in &columns {
            prop_assert!(x0 < x1 && x1 <= w && y0 < y1 && y1 <= h);
        }
        prop_assert!(columns.windows(2).all(|pair| pair[0][0] >= pair[1][0]));
    }
}
