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

/// 上下に分離した段は上→下、各段内は右→左で返し、同じxの列を結合しない。
#[test]
fn stacked_blocks_preserve_reading_order_and_original_bounds() {
    let (w, h) = (180, 340);
    let bodies = [
        [120, 20, 140, 120],
        [40, 20, 60, 120],
        [120, 200, 140, 300],
        [40, 200, 60, 300],
    ];
    let mut rgb = vec![255; w * h * 3];
    for body in bodies {
        rect(&mut rgb, w, body);
    }
    let actual = detect_vertical_textline_columns(&rgb, w, h, 220).unwrap();
    assert_eq!(
        actual,
        bodies.map(|[x0, y0, x1, y1]| [x0 - 1, y0 - 2, x1 + 1, y1 + 2])
    );
}

/// 字間が全列で揃っていても、字幅より狭い白帯では段を作らない。
#[test]
fn aligned_character_spaces_remain_one_block() {
    let (w, h) = (180, 340);
    let mut rgb = vec![255; w * h * 3];
    for x in [40, 120] {
        for y in [20, 60, 100, 140, 180, 220] {
            rect(&mut rgb, w, [x, y, x + 20, y + 20]);
        }
    }
    let actual = detect_vertical_textline_columns(&rgb, w, h, 220).unwrap();
    assert_eq!(actual, [[119, 18, 141, 242], [39, 18, 61, 242]]);
}

/// 一列が段間を横断するときは、局所的な空白をページ全体の境界にしない。
#[test]
fn local_column_gaps_do_not_create_page_blocks() {
    let (w, h) = (180, 340);
    let mut rgb = vec![255; w * h * 3];
    for body in [[120, 20, 140, 300], [40, 20, 60, 120], [40, 200, 60, 300]] {
        rect(&mut rgb, w, body);
    }
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        [[119, 18, 141, 302], [39, 18, 61, 302]]
    );
}

/// 手動固定した実資料の上23列・下24列を、本文ROIの元座標で保持する。
#[test]
fn printed_body_has_two_blocks_and_forty_seven_columns() {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/stacked_vertical/body.png",
    ))
    .unwrap();
    let columns = detect_vertical_textline_columns(&im.data, im.width, im.height, 220).unwrap();
    eprintln!("STACKED BOUNDS {columns:?}");
    assert_eq!(columns.len(), 47);
    assert!(columns[..23].iter().all(|b| b[3] < 310));
    assert!(columns[23..].iter().all(|b| b[1] > 310));
    for block in [&columns[..23], &columns[23..]] {
        assert!(block.windows(2).all(|pair| pair[0][0] > pair[1][0]));
    }
    // 写真や枠を除いた本文ROIなので、閾値内の全画素に検出領域を要求する。
    let mut missing = Vec::new();
    for y in 0..im.height {
        for x in 0..im.width {
            let i = (y * im.width + x) * 3;
            let ink = (im.data[i] as u16 + im.data[i + 1] as u16 + im.data[i + 2] as u16) / 3 < 220;
            if ink
                && !columns
                    .iter()
                    .any(|b| b[0] <= x && x < b[2] && b[1] <= y && y < b[3])
            {
                missing.push((x, y));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "uncovered body ink: {} pixels, first {:?}",
        missing.len(),
        &missing[..missing.len().min(12)]
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 拡大・移動しても段順・列順と全インクの元座標を保つ。
    #[test]
    fn stacked_blocks_scale_and_translate(scale in 1usize..4, dx in 0usize..11, dy in 0usize..11) {
        let (w,h)=(180*scale+dx,340*scale+dy);
        let mut rgb=vec![255;w*h*3];
        let bodies=[[120,20,140,120],[40,20,60,120],
            [120,200,140,300],[40,200,60,300]]
            .map(|[x0,y0,x1,y1]| [x0*scale+dx,y0*scale+dy,x1*scale+dx,y1*scale+dy]);
        for body in bodies { rect(&mut rgb,w,body); }
        let actual=detect_vertical_textline_columns(&rgb,w,h,220).unwrap();
        prop_assert_eq!(actual,bodies.map(|[x0,y0,x1,y1]| [x0-1,y0-2,x1+1,y1+2]));
    }
}

/// 三段でも段順を保ち、前後の大きなページ余白から空の段を作らない。
#[test]
fn three_blocks_ignore_outer_page_margins() {
    let (w, h) = (180, 680);
    let mut rgb = vec![255; w * h * 3];
    for y in [80, 280, 480] {
        for x in [40, 120] {
            rect(&mut rgb, w, [x, y, x + 20, y + 100]);
        }
    }
    let expected: Vec<_> = [80, 280, 480]
        .into_iter()
        .flat_map(|y| [[119, y - 2, 141, y + 102], [39, y - 2, 61, y + 102]])
        .collect();
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        expected
    );
}

/// 単列だけの広い段落空白は複数列による裏付けがないため分割しない。
#[test]
fn single_column_keeps_large_internal_paragraph_gap() {
    let (w, h) = (180, 340);
    let mut rgb = vec![255; w * h * 3];
    for body in [[120, 20, 140, 120], [120, 200, 140, 300]] {
        rect(&mut rgb, w, body);
    }
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        [[119, 18, 141, 302]]
    );
}

/// 分割後の片側が単列の場合も、段としての裏付けが足りないので元の検出を保つ。
#[test]
fn unsupported_single_column_block_keeps_original_detection() {
    let (w, h) = (180, 340);
    let mut rgb = vec![255; w * h * 3];
    for body in [[120, 20, 140, 120], [40, 20, 60, 120], [120, 200, 140, 300]] {
        rect(&mut rgb, w, body);
    }
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        [[119, 18, 141, 302], [39, 18, 61, 122]]
    );
}

/// 本文の外側余白に段を貫く細い罫線があっても、上下の本文を結合しない。
#[test]
fn outer_margin_rule_does_not_join_stacked_body() {
    for rule_x in [10, 165] {
        let (w, h) = (180, 340);
        let mut rgb = vec![255; w * h * 3];
        for body in [
            [120, 20, 140, 120],
            [40, 20, 60, 120],
            [120, 200, 140, 300],
            [40, 200, 60, 300],
        ] {
            rect(&mut rgb, w, body);
        }
        rect(&mut rgb, w, [rule_x, 10, rule_x + 2, 320]);
        assert_eq!(
            detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
            [
                [119, 18, 141, 122],
                [39, 18, 61, 122],
                [119, 198, 141, 302],
                [39, 198, 61, 302]
            ],
            "rule x={rule_x}"
        );
    }
}

/// 検出列の間を貫く細線は本文内なので、外側余白の罫線として無視しない。
#[test]
fn internal_rule_keeps_original_detection() {
    let (w, h) = (180, 340);
    let mut rgb = vec![255; w * h * 3];
    for body in [
        [120, 20, 140, 120],
        [40, 20, 60, 120],
        [120, 200, 140, 300],
        [40, 200, 60, 300],
        [90, 10, 92, 320],
    ] {
        rect(&mut rgb, w, body);
    }
    assert_eq!(
        detect_vertical_textline_columns(&rgb, w, h, 220).unwrap(),
        [[119, 18, 141, 302], [39, 18, 61, 302]]
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 移動・拡大・左右を変えても、外側の細罫線が本文の段を結合しない。
    #[test]
    fn margin_rule_blocks_scale_and_translate(
        scale in 1usize..4, dx in 0usize..11, dy in 0usize..11, left in any::<bool>()
    ) {
        let (w, h) = (180 * scale + dx, 340 * scale + dy);
        let mut rgb = vec![255; w * h * 3];
        let bodies = [[120,20,140,120], [40,20,60,120],
            [120,200,140,300], [40,200,60,300]]
            .map(|[x0,y0,x1,y1]| [x0*scale+dx,y0*scale+dy,x1*scale+dx,y1*scale+dy]);
        for body in bodies { rect(&mut rgb,w,body); }
        let x = (if left { 10 } else { 165 }) * scale + dx;
        rect(&mut rgb, w, [x, 10*scale+dy, x+2*scale, 320*scale+dy]);
        let actual = detect_vertical_textline_columns(&rgb,w,h,220).unwrap();
        prop_assert_eq!(actual, bodies.map(|[x0,y0,x1,y1]| [x0-1,y0-2,x1+1,y1+2]));
    }
}
