use ndlocr_lite_rs::pipeline::line_hypotheses::{MergeBudget, adjacent_line_merge_hypotheses};

/// 小さなテストで十分な予算を用意する。
fn budget() -> MergeBudget {
    MergeBudget {
        max_source_bands: 100,
        max_candidates: 100,
        max_total_pixels: 1_000_000,
    }
}

/// 元の帯を失わず、重複する候補も元の添字へ対応付ける。
#[test]
fn alternatives_preserve_sources_and_do_not_chain() {
    let bands = [[10, 10, 90, 20], [20, 30, 80, 40], [5, 60, 95, 80]];
    let original = bands;
    let result = adjacent_line_merge_hypotheses(100, 100, &bands, budget()).unwrap();
    assert_eq!(bands, original);
    assert_eq!(result.candidates.len(), 2);
    assert_eq!(result.candidates[0].source_indices, [0, 1]);
    assert_eq!(result.candidates[0].bounds, [10, 10, 90, 40]);
    assert_eq!(result.candidates[1].source_indices, [1, 2]);
    assert_eq!(result.candidates[1].bounds, [5, 30, 95, 80]);
    assert_eq!(result.total_pixels, 80 * 30 + 90 * 50);
    assert_eq!(result.omitted_pairs, 0);
}

/// 件数境界と画素量境界を厳密に守り、未列挙の数を隠さない。
#[test]
fn exact_and_zero_budgets_report_omissions() {
    let bands = [[0, 0, 10, 10], [0, 10, 10, 20], [0, 20, 10, 30]];
    for limit in [
        MergeBudget {
            max_candidates: 1,
            ..budget()
        },
        MergeBudget {
            max_total_pixels: 200,
            ..budget()
        },
    ] {
        let result = adjacent_line_merge_hypotheses(10, 30, &bands, limit).unwrap();
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.total_pixels, 200);
        assert_eq!(result.omitted_pairs, 1);
    }
    for limit in [
        MergeBudget {
            max_candidates: 0,
            ..budget()
        },
        MergeBudget {
            max_total_pixels: 199,
            ..budget()
        },
        MergeBudget {
            max_total_pixels: 0,
            ..budget()
        },
    ] {
        let result = adjacent_line_merge_hypotheses(10, 30, &bands, limit).unwrap();
        assert!(result.candidates.is_empty());
        assert_eq!(result.total_pixels, 0);
        assert_eq!(result.omitted_pairs, 2);
    }
}

/// 大きな候補を飛ばしても、後続の小さな候補へ残予算を使える。
#[test]
fn expensive_pair_does_not_block_later_small_pair() {
    let bands = [[0, 0, 100, 5], [0, 10, 10, 15], [0, 20, 10, 25]];
    let result = adjacent_line_merge_hypotheses(
        100,
        30,
        &bands,
        MergeBudget {
            max_total_pixels: 150,
            ..budget()
        },
    )
    .unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].source_indices, [1, 2]);
    assert_eq!(result.total_pixels, 150);
    assert_eq!(result.omitted_pairs, 1);
}

/// 無効矩形と順序を、無効化された候補予算でも黙って受理しない。
#[test]
fn rejects_invalid_bounds_order_and_dimensions() {
    let disabled = MergeBudget {
        max_candidates: 0,
        ..budget()
    };
    for bands in [
        vec![[10, 0, 10, 5]],
        vec![[20, 0, 10, 5]],
        vec![[0, 5, 10, 5]],
        vec![[0, 10, 10, 5]],
        vec![[0, 0, 101, 5]],
        vec![[0, 0, 10, 101]],
        vec![[0, 10, 10, 20], [0, 0, 10, 5]],
        vec![[0, 0, 10, 20], [0, 19, 10, 30]],
    ] {
        assert!(
            adjacent_line_merge_hypotheses(100, 100, &bands, disabled).is_err(),
            "{bands:?}"
        );
    }
    for (w, h) in [(0, 10), (10, 0), (usize::MAX, 2)] {
        assert!(adjacent_line_merge_hypotheses(w, h, &[], budget()).is_err());
    }
}

/// 入力帯数の上限は原領域を切り詰めずエラーにする。
#[test]
fn source_limit_is_not_silent_truncation() {
    let bands = [[0, 0, 10, 10], [0, 10, 10, 20]];
    assert!(
        adjacent_line_merge_hypotheses(
            10,
            20,
            &bands,
            MergeBudget {
                max_source_bands: 1,
                ..budget()
            }
        )
        .is_err()
    );
    assert!(
        adjacent_line_merge_hypotheses(
            10,
            20,
            &bands,
            MergeBudget {
                max_source_bands: 2,
                ..budget()
            }
        )
        .is_ok()
    );
}

/// 各矩形は有効でも面積合計がusizeを超え得る場合、減算した残予算で制限する。
#[test]
fn cumulative_pixels_do_not_overflow() {
    let width = usize::MAX / 3;
    let bands = [[0, 0, width, 1], [0, 1, width, 2], [0, 2, width, 3]];
    let result = adjacent_line_merge_hypotheses(
        width,
        3,
        &bands,
        MergeBudget {
            max_total_pixels: usize::MAX,
            ..budget()
        },
    )
    .unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.total_pixels, width * 2);
    assert_eq!(result.omitted_pairs, 1);
}

/// 空集合と1帯は巨大な予算でも確保を要求せず候補なしで返る。
#[test]
fn empty_and_single_sources_need_no_candidates() {
    let unlimited = MergeBudget {
        max_source_bands: usize::MAX,
        max_candidates: usize::MAX,
        max_total_pixels: usize::MAX,
    };
    for bands in [vec![], vec![[0, 0, 1, 1]]] {
        let result = adjacent_line_merge_hypotheses(1, 1, &bands, unlimited).unwrap();
        assert!(result.candidates.is_empty());
        assert_eq!((result.total_pixels, result.omitted_pairs), (0, 0));
    }
}

/// 既存開発画像の4帯を保持し、見出しの和集合だけを任意採点用に取得できる。
#[test]
fn disconnected_heading_is_an_alternative_not_a_replacement() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/noisy_line_gaps/mixed_height.png",
    ))
    .unwrap();
    let bands = ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive(
        &image.data,
        image.width,
        image.height,
        220,
    );
    assert_eq!(bands.len(), 4);
    let original = bands.clone();
    let result =
        adjacent_line_merge_hypotheses(image.width, image.height, &bands, budget()).unwrap();
    assert_eq!(bands, original);
    assert_eq!(result.candidates.len(), 3);
    let heading = &result.candidates[2];
    assert_eq!(heading.source_indices, [2, 3]);
    assert_eq!(heading.bounds, [56, 229, 926, 372]);
    println!(
        "LINE_MERGE_REPORT={}",
        serde_json::json!({
            "original_bands": bands,
            "candidates": result.candidates.iter().map(|c| serde_json::json!({
                "source_indices": c.source_indices, "bounds": c.bounds
            })).collect::<Vec<_>>(),
            "total_pixels": result.total_pixels,
            "omitted_pairs": result.omitted_pairs,
            "recognizer_calls": 0,
        })
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    /// 有効な任意帯と予算で、元領域・添字・矩形被覆・順序・総量を保持する。
    #[test]
    fn candidates_obey_resource_and_source_invariants(
        sizes in proptest::collection::vec((1usize..200, 1usize..30, 0usize..30), 0..40),
        max_candidates in 0usize..45, max_total_pixels in 0usize..50000,
    ) {
        let mut y = 0;
        let bands: Vec<_> = sizes.into_iter().map(|(w,h,gap)| {
            y += gap;
            let b = [0, y, w, y+h]; y += h; b
        }).collect();
        let original = bands.clone();
        let limit = MergeBudget { max_candidates, max_total_pixels, ..budget() };
        let result = adjacent_line_merge_hypotheses(200, y.max(1), &bands, limit).unwrap();
        proptest::prop_assert_eq!(&bands, &original);
        proptest::prop_assert!(result.candidates.len() <= max_candidates);
        proptest::prop_assert!(result.total_pixels <= max_total_pixels);
        proptest::prop_assert_eq!(result.candidates.len() + result.omitted_pairs, bands.len().saturating_sub(1));
        let mut pixels = 0;
        for c in &result.candidates {
            let [a,b] = c.source_indices;
            proptest::prop_assert_eq!(b, a+1);
            let area = (c.bounds[2]-c.bounds[0])*(c.bounds[3]-c.bounds[1]);
            pixels += area;
            for band in [bands[a], bands[b]] {
                proptest::prop_assert!(c.bounds[0]<=band[0] && c.bounds[1]<=band[1]);
                proptest::prop_assert!(c.bounds[2]>=band[2] && c.bounds[3]>=band[3]);
            }
        }
        proptest::prop_assert_eq!(pixels, result.total_pixels);
        proptest::prop_assert!(result.candidates.windows(2).all(|p| p[0].source_indices[0]<p[1].source_indices[0]));
        proptest::prop_assert_eq!(&result, &adjacent_line_merge_hypotheses(200, y.max(1), &bands, limit).unwrap());
    }
}
