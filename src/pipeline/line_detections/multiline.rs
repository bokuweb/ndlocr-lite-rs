use super::{eligible, nearly_same_box};
use crate::infer::deim::Detection;

/// 既に保持した強い2行だけで説明できる弱い複数行候補かを確認する。
pub(super) fn covered_by_two_lines(
    candidate: &Detection,
    kept: &[&Detection],
    rgb: &[u8],
    width: usize,
    height: usize,
) -> bool {
    if !eligible(candidate, width, height)
        || !(0.0..=1.0).contains(&candidate.confidence)
        || candidate.confidence == 0.0
    {
        return false;
    }
    let b = candidate.box_xyxy.map(i64::from);
    let vertical = b[3] - b[1] > b[2] - b[0];
    let (major, minor) = if vertical { (1, 0) } else { (0, 1) };
    let size = |bounds: [i64; 4], axis: usize| bounds[axis + 2] - bounds[axis];
    let supports: Vec<_> = kept
        .iter()
        .copied()
        .filter(|d| {
            if !eligible(d, width, height)
                || d.confidence > 1.0
                || d.confidence <= candidate.confidence * 2.0
            {
                return false;
            }
            let s = d.box_xyxy.map(i64::from);
            let sh = size(s, minor);
            size(s, major) >= sh * 4
                && size(s, major) * 2 >= size(b, major)
                && sh * 4 >= size(b, minor)
                && sh * 2 <= size(b, minor)
                && mostly_inside(s, b)
        })
        .collect();
    let [first, second] = supports.as_slice() else {
        return false;
    };
    let (mut a, mut c) = (
        first.box_xyxy.map(i64::from),
        second.box_xyxy.map(i64::from),
    );
    if a[minor] > c[minor] {
        std::mem::swap(&mut a, &mut c);
    }
    let gap = c[minor] - a[minor + 2];
    if gap * 4 < size(a, minor).min(size(c, minor)) || gap > size(a, minor).max(size(c, minor)) {
        return false;
    }
    let union = [
        a[0].min(c[0]),
        a[1].min(c[1]),
        a[2].max(c[2]),
        a[3].max(c[3]),
    ];
    if !nearly_same_box(candidate.box_xyxy, union.map(|v| v as i32)) {
        return false;
    }
    let padded = [a, c].map(|s| {
        let pad = (size(s, minor) + 11) / 12;
        [s[0] - pad, s[1] - pad, s[2] + pad, s[3] + pad]
    });
    let mut has_ink = false;
    for y in b[1]..b[3] {
        for x in b[0]..b[2] {
            let i = (y as usize * width + x as usize) * 3;
            if rgb[i..i + 3] == [255; 3] {
                continue;
            }
            has_ink = true;
            if !padded
                .iter()
                .any(|s| x >= s[0] && x < s[2] && y >= s[1] && y < s[3])
            {
                return false;
            }
        }
    }
    has_ink
}

/// 支持行の95%以上が候補内にあり、隣接する別領域を根拠にしない。
fn mostly_inside(s: [i64; 4], b: [i64; 4]) -> bool {
    let iw = (s[2].min(b[2]) - s[0].max(b[0])).max(0) as u128;
    let ih = (s[3].min(b[3]) - s[1].max(b[1])).max(0) as u128;
    let area = (s[2] - s[0]) as u128 * (s[3] - s[1]) as u128;
    iw * ih * 100 >= area * 95
}
