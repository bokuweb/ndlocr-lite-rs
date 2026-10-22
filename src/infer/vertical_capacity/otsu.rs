//! Exact histogram thresholding for a support mask; never an NN input.
use std::cmp::Ordering;

fn compare((mut a, mut b): (u128, u128), (mut c, mut d): (u128, u128)) -> Ordering {
    debug_assert!(b > 0 && d > 0);
    let mut reversed = false;
    loop {
        let whole = (a / b).cmp(&(c / d));
        if whole != Ordering::Equal {
            return if reversed { whole.reverse() } else { whole };
        }
        let (left, right) = (a % b, c % d);
        if left == 0 || right == 0 {
            let tail = left.cmp(&right);
            return if reversed { tail.reverse() } else { tail };
        }
        (a, b, c, d) = (b, left, d, right);
        reversed = !reversed;
    }
}

pub(super) fn threshold(histogram: &[u128; 256]) -> Option<u8> {
    let total = histogram
        .iter()
        .try_fold(0u128, |n, &count| n.checked_add(count))?;
    if total == 0 {
        return None;
    }
    let sum = histogram
        .iter()
        .enumerate()
        .try_fold(0u128, |sum, (level, &count)| {
            sum.checked_add(count.checked_mul(level as u128)?)
        })?;
    let (mut weight, mut prefix) = (0u128, 0u128);
    let mut best = None;
    for (level, &count) in histogram.iter().enumerate().take(255) {
        weight = weight.checked_add(count)?;
        prefix = prefix.checked_add(count.checked_mul(level as u128)?)?;
        if weight == 0 || weight == total {
            continue;
        }
        let delta = sum
            .checked_mul(weight)?
            .checked_sub(prefix.checked_mul(total)?)?;
        let score = (
            delta.checked_mul(delta)?,
            weight.checked_mul(total - weight)?,
        );
        if best.is_none_or(|(numerator, denominator, _)| {
            compare(score, (numerator, denominator)) == Ordering::Greater
        }) {
            best = Some((score.0, score.1, level as u8));
        }
    }
    best.map(|(_, _, level)| level)
}

pub(super) fn support_mask(rgb: &[u8]) -> Option<(Vec<u8>, u8)> {
    let (pixels, remainder) = rgb.as_chunks::<3>();
    if pixels.is_empty() || !remainder.is_empty() {
        return None;
    }
    let gray = |p: &[u8; 3]| (p.iter().map(|&v| u16::from(v)).sum::<u16>() / 3) as u8;
    let mut histogram = [0u128; 256];
    for pixel in pixels {
        let bin = &mut histogram[usize::from(gray(pixel))];
        *bin = bin.checked_add(1)?;
    }
    let threshold = threshold(&histogram)?;
    let mut mask = Vec::new();
    mask.try_reserve_exact(rgb.len()).ok()?;
    for pixel in pixels {
        mask.extend_from_slice(&[if gray(pixel) <= threshold { 0 } else { 255 }; 3]);
    }
    Some((mask, threshold))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn fractions_are_ordered_without_overflowing_the_cross_product() {
        assert_eq!(compare((1, 2), (2, 3)), Ordering::Less);
        assert_eq!(compare((3, 2), (4, 3)), Ordering::Greater);
        assert_eq!(compare((7, 3), (14, 6)), Ordering::Equal);
        assert_eq!(compare((0, 7), (1, 9)), Ordering::Less);
        assert_eq!(
            compare((u128::MAX, u128::MAX - 1), (u128::MAX - 1, u128::MAX)),
            Ordering::Greater
        );
        assert_eq!(compare((u128::MAX, u128::MAX), (1, 1)), Ordering::Equal);
    }

    #[test]
    fn mask_uses_the_floored_rgb_average_and_keeps_the_threshold_bin_black() {
        let rgb = [63, 64, 65, 199, 200, 201];
        let before = rgb;
        let (mask, threshold) = support_mask(&rgb).unwrap();
        assert_eq!(threshold, 64);
        assert_eq!(mask, [0, 0, 0, 255, 255, 255]);
        assert_eq!(rgb, before);
        let (mask, threshold) = support_mask(&[0, 0, 3, 2, 2, 2]).unwrap();
        assert_eq!(threshold, 1);
        assert_eq!(mask, [0, 0, 0, 255, 255, 255]);
    }

    #[test]
    fn mask_rejects_partial_rgb_pixels_and_single_gray_levels() {
        for rgb in [
            vec![],
            vec![1],
            vec![1, 2],
            vec![1, 2, 3, 4],
            vec![0; 6],
            vec![255; 6],
            vec![240, 220, 170, 240, 220, 170],
        ] {
            assert!(support_mask(&rgb).is_none());
        }
    }
    fn histogram(points: &[(usize, u128)]) -> [u128; 256] {
        let mut result = [0; 256];
        for &(level, count) in points {
            result[level] = count;
        }
        result
    }

    #[test]
    fn two_tones_and_tied_three_tones_keep_the_lowest_maximum() {
        assert_eq!(threshold(&histogram(&[(0, 5), (255, 5)])), Some(0));
        assert_eq!(threshold(&histogram(&[(64, 7), (200, 3)])), Some(64));
        assert_eq!(
            threshold(&histogram(&[(0, 1), (127, 1), (254, 1)])),
            Some(0)
        );
        assert_eq!(
            threshold(&histogram(&[(10, 1), (20, 1), (200, 8)])),
            Some(20)
        );
    }

    #[test]
    fn uniform_gray_levels_are_split_at_the_middle() {
        assert_eq!(threshold(&[1; 256]), Some(127));
    }

    #[test]
    fn empty_single_tone_and_unrepresentable_scores_have_no_threshold() {
        assert_eq!(threshold(&[0; 256]), None);
        assert_eq!(threshold(&histogram(&[(128, 4)])), None);
        assert_eq!(threshold(&histogram(&[(0, u128::MAX), (255, 1)])), None);
        assert_eq!(
            threshold(&histogram(&[(0, 10_000_000_000), (255, 10_000_000_000)])),
            None
        );
    }
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn positive_histogram_scaling_does_not_change_the_threshold(
            weights in proptest::collection::vec(0u16..65, 192),
            scale in 1u16..65,
        ) {
            let mut original = [0; 256];
            let mut scaled = [0; 256];
            for (i, &count) in weights.iter().enumerate() {
                original[i] = u128::from(count);
                scaled[i] = u128::from(count) * u128::from(scale);
            }
            let result = threshold(&original);
            prop_assert!(result.is_some() || weights.iter().filter(|&&n| n > 0).count() < 2);
            prop_assert_eq!(threshold(&scaled), result);
        }

        #[test]
        fn non_saturating_gray_translation_moves_the_threshold_by_the_same_amount(
            weights in proptest::collection::vec(0u16..65, 192),
            shift in 0u8..65,
        ) {
            let mut original = [0; 256];
            let mut moved = [0; 256];
            for (i, &count) in weights.iter().enumerate() {
                original[i] = u128::from(count);
                moved[i + usize::from(shift)] = u128::from(count);
            }
            let result = threshold(&original);
            prop_assert!(result.is_some() || weights.iter().filter(|&&n| n > 0).count() < 2);
            prop_assert_eq!(threshold(&moved), result.map(|t| t + shift));
        }

        #[test]
        fn exact_comparison_agrees_with_safe_small_cross_products(
            a in 0u64..1_000_000_000,
            b in 1u64..1_000_000_000,
            c in 0u64..1_000_000_000,
            d in 1u64..1_000_000_000,
        ) {
            let (a,b,c,d) = (u128::from(a),u128::from(b),u128::from(c),u128::from(d));
            prop_assert_eq!(compare((a,b),(c,d)), (a*d).cmp(&(c*b)));
        }
    }
}
