//! 認識済みの同一行の包含断片を、信頼できる親文脈へ整理する。
pub(crate) struct ScoredRegion<'a> {
    pub bounds: [i32; 4],
    pub text: &'a str,
    pub mean_confidence: f32,
}

/// Return original-order indices without changing the recognized results.
pub(crate) fn select_contexts(rows: &[ScoredRegion<'_>], native: (usize, usize)) -> Vec<usize> {
    let mut removed = vec![false; rows.len()];
    if native.0 == 0 || native.1 == 0 {
        return (0..rows.len()).collect();
    }
    let valid = |row: &ScoredRegion<'_>| {
        let [x0, y0, x1, y1] = row.bounds;
        x0 >= 0
            && y0 >= 0
            && x1 > x0
            && y1 > y0
            && row.mean_confidence.is_finite()
            && row.mean_confidence > 0.0
            && row.mean_confidence <= 1.0
            && row.text.chars().any(|c| !c.is_whitespace())
    };
    let area = |i: usize| {
        let [x0, y0, x1, y1] = rows[i].bounds.map(i64::from);
        ((x1 - x0) as u128) * ((y1 - y0) as u128)
    };
    let mut parents: Vec<_> = (0..rows.len()).filter(|&i| valid(&rows[i])).collect();
    parents.sort_by_key(|&i| std::cmp::Reverse(area(i)));
    for parent in parents {
        if removed[parent] {
            continue;
        }
        let [x0, y0, x1, y1] = rows[parent].bounds.map(i64::from);
        let (width, height) = (x1 - x0, y1 - y0);
        let vertical = height > width;
        let axes = |bounds: [i32; 4]| {
            let [a, b, c, d] = bounds.map(i64::from);
            if vertical { (b, d, a, c) } else { (a, c, b, d) }
        };
        let (start, end, minor_start, minor_end) = axes(rows[parent].bounds);
        let (major, minor) = (end - start, minor_end - minor_start);
        if major * 4 < minor * 5
            || major as u128 * native.1 as u128 > minor as u128 * native.0 as u128
        {
            continue;
        }
        let pad = (minor + 9) / 10;
        let mut children: Vec<_> = (0..rows.len())
            .filter(|&i| {
                if i == parent || removed[i] || !valid(&rows[i]) {
                    return false;
                }
                let (a, b, c, d) = axes(rows[i].bounds);
                b - a < major
                    && a >= start - pad
                    && b <= end + pad
                    && (c - minor_start).abs() <= pad
                    && (d - minor_end).abs() <= pad
            })
            .collect();
        children.sort_by(|&a, &b| rows[b].mean_confidence.total_cmp(&rows[a].mean_confidence));
        let mut support: Vec<usize> = Vec::new();
        for &child in &children {
            let (a, b, _, _) = axes(rows[child].bounds);
            if support.iter().all(|&i| {
                let (c, d, _, _) = axes(rows[i].bounds);
                b <= c || a >= d
            }) {
                support.push(child);
            }
        }
        if support.len() < 2 {
            continue;
        }
        support.sort_by_key(|&i| axes(rows[i].bounds).0);
        let first = axes(rows[support[0]].bounds).0;
        let last = axes(rows[*support.last().unwrap()].bounds).1;
        if (last - first) * 4 < major * 3
            || support
                .windows(2)
                .any(|pair| axes(rows[pair[1]].bounds).0 - axes(rows[pair[0]].bounds).1 > minor)
        {
            continue;
        }
        let mut total_width = 0i64;
        let mut weighted_confidence = 0f64;
        let mut explained_chars = 0f64;
        for &i in &support {
            let (a, b, _, _) = axes(rows[i].bounds);
            let score = f64::from(rows[i].mean_confidence);
            total_width += b - a;
            weighted_confidence += (b - a) as f64 * score;
            explained_chars +=
                rows[i].text.chars().filter(|c| !c.is_whitespace()).count() as f64 * score;
        }
        let parent_chars = rows[parent]
            .text
            .chars()
            .filter(|c| !c.is_whitespace())
            .count();
        if f64::from(rows[parent].mean_confidence) < weighted_confidence / total_width as f64
            || (parent_chars as f64) * 2.0 < explained_chars
        {
            continue;
        }
        for child in children {
            removed[child] = true;
        }
    }
    (0..rows.len()).filter(|&i| !removed[i]).collect()
}
#[cfg(test)]
#[path = "../../tests/support/contained_line_context_policy.rs"]
mod tests;
