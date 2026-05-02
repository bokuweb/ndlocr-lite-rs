use ndlocr_lite_rs::pipeline::connect::RecognizedLine;
use ndlocr_lite_rs::pipeline::reading_order::sort_lines_in_reading_order;

#[test]
fn sort_lines_in_reading_order_sorts_horizontal_by_y_then_x() {
    let mut lines = vec![
        RecognizedLine {
            bbox_xyxy: [20, 30, 30, 40],
            text: "c".into(),
            confidence: 1.0,
            is_vertical: false,
        },
        RecognizedLine {
            bbox_xyxy: [10, 10, 20, 20],
            text: "a".into(),
            confidence: 1.0,
            is_vertical: false,
        },
        RecognizedLine {
            bbox_xyxy: [30, 10, 40, 20],
            text: "b".into(),
            confidence: 1.0,
            is_vertical: false,
        },
    ];
    sort_lines_in_reading_order(&mut lines);
    let texts: Vec<_> = lines.into_iter().map(|l| l.text).collect();
    assert_eq!(texts, vec!["a", "b", "c"]);
}

#[test]
fn sort_lines_in_reading_order_sorts_vertical_by_x_desc_then_y() {
    let mut lines = vec![
        RecognizedLine {
            bbox_xyxy: [10, 30, 20, 40],
            text: "b".into(),
            confidence: 1.0,
            is_vertical: true,
        },
        RecognizedLine {
            bbox_xyxy: [30, 10, 40, 20],
            text: "a".into(),
            confidence: 1.0,
            is_vertical: true,
        },
        RecognizedLine {
            bbox_xyxy: [10, 10, 20, 20],
            text: "c".into(),
            confidence: 1.0,
            is_vertical: true,
        },
    ];
    sort_lines_in_reading_order(&mut lines);
    let texts: Vec<_> = lines.into_iter().map(|l| l.text).collect();
    assert_eq!(texts, vec!["a", "c", "b"]);
}

#[test]
fn sort_lines_in_reading_order_prioritizes_vertical_over_horizontal_when_mixed() {
    let mut lines = vec![
        RecognizedLine {
            bbox_xyxy: [10, 10, 60, 20],
            text: "h1".into(),
            confidence: 1.0,
            is_vertical: false,
        },
        RecognizedLine {
            bbox_xyxy: [40, 5, 50, 40],
            text: "v1".into(),
            confidence: 1.0,
            is_vertical: true,
        },
        RecognizedLine {
            bbox_xyxy: [20, 20, 70, 30],
            text: "h2".into(),
            confidence: 1.0,
            is_vertical: false,
        },
        RecognizedLine {
            bbox_xyxy: [30, 10, 40, 45],
            text: "v2".into(),
            confidence: 1.0,
            is_vertical: true,
        },
    ];
    sort_lines_in_reading_order(&mut lines);
    let texts: Vec<_> = lines.into_iter().map(|l| l.text).collect();
    assert_eq!(texts, vec!["v1", "v2", "h1", "h2"]);
}
