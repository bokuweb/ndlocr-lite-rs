//! Cascade-bucket helpers exposed for downstream consumers (e.g. apps that
//! drive [`crate::infer::cached::ParseqCascadePool`] directly without going
//! through the bundled CLI in `app.rs`).
//!
//! `ParseqCascadePool::recognize_batch_with_buckets_rgb_u8` takes a
//! `pred_char_count: Option<f32>` per line, where:
//!   * `Some(3.0)` → use the 30-char model (smallest, fastest)
//!   * `Some(2.0)` → use the 50-char model (medium)
//!   * any other value (incl. `None`) → use the 100-char model (largest)
//!
//! Choosing the right bucket per line is the whole point of the cascade — a
//! short Japanese line never needs the 100-char decoder, and using the 30-char
//! model makes parseq inference roughly 2-3x faster end-to-end.
//!
//! These helpers were previously private to `app.rs` (the CLI). Lifting them
//! to a public module so library consumers can reuse the same heuristic and
//! defaults without re-implementing them ad-hoc.

/// Default char-count threshold that bumps a line out of the 30-char bucket
/// into the 50-char bucket (matches the bundled CLI default).
pub const DEFAULT_CASCADE_THRESHOLD_30_TO_50: usize = 25;

/// Default char-count threshold that bumps a line out of the 50-char bucket
/// into the 100-char bucket (matches the bundled CLI default).
pub const DEFAULT_CASCADE_THRESHOLD_50_TO_100: usize = 45;

/// Empirical estimator of how many characters a line crop is likely to hold,
/// based purely on its aspect ratio. Tuned for Japanese page-line crops where
/// glyphs are roughly square. Returns `>= 1`.
pub fn estimate_line_char_count(width: usize, height: usize) -> usize {
    let h = height.max(1);
    let ratio = width as f32 / h as f32;
    (ratio * 2.5).round().max(1.0) as usize
}

/// Choose the cascade bucket for a line of the given pixel size, using the
/// supplied char-count thresholds. The return value is the encoding expected
/// by [`crate::infer::cached::ParseqCascadePool::recognize_batch_with_buckets_rgb_u8`]:
/// `3.0` for 30-char model, `2.0` for 50-char model, `100.0` for 100-char model.
pub fn estimate_pred_char_bucket(
    width: usize,
    height: usize,
    th_30_to_50: usize,
    th_50_to_100: usize,
) -> f32 {
    let estimated_chars = estimate_line_char_count(width, height);
    if estimated_chars <= th_30_to_50 {
        3.0
    } else if estimated_chars <= th_50_to_100 {
        2.0
    } else {
        100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_char_count_scales_with_aspect_ratio() {
        // 24x60 (very narrow): ratio 2.5, estimate ~6 chars.
        assert_eq!(estimate_line_char_count(60, 24), 6);
        // 24x240 (wide): ratio 10, estimate ~25 chars.
        assert_eq!(estimate_line_char_count(240, 24), 25);
        // 24x768 (very wide): ratio 32, estimate ~80 chars.
        assert_eq!(estimate_line_char_count(768, 24), 80);
    }

    #[test]
    fn line_char_count_handles_zero_height() {
        // Avoids division by zero (rare crop edge case).
        assert!(estimate_line_char_count(100, 0) >= 1);
    }

    #[test]
    fn line_char_count_returns_at_least_one() {
        // Very short crop should still bucket somewhere, not 0.
        assert!(estimate_line_char_count(1, 10) >= 1);
    }

    #[test]
    fn bucket_routes_short_lines_to_30_model() {
        let b = estimate_pred_char_bucket(60, 24, 25, 45);
        assert_eq!(b, 3.0, "narrow line should pick 30-char model");
    }

    #[test]
    fn bucket_routes_medium_lines_to_50_model() {
        // ~30 chars line → above 30→50 threshold (25), below 50→100 (45).
        let b = estimate_pred_char_bucket(288, 24, 25, 45);
        assert_eq!(b, 2.0);
    }

    #[test]
    fn bucket_routes_long_lines_to_100_model() {
        // Wide crop, > 45 estimated chars.
        let b = estimate_pred_char_bucket(600, 24, 25, 45);
        assert_eq!(b, 100.0);
    }

    #[test]
    fn bucket_uses_default_thresholds() {
        // Sanity: the published defaults match the CLI behaviour.
        assert_eq!(DEFAULT_CASCADE_THRESHOLD_30_TO_50, 25);
        assert_eq!(DEFAULT_CASCADE_THRESHOLD_50_TO_100, 45);
    }
}
