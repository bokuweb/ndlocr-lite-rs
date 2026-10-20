//! Metadata-only bounds for diagnostics; never guesses a dynamic time axis.
pub(super) fn static_frame_limit(shape: &[i64], charset_len: usize) -> Option<usize> {
    let expected_classes = i64::try_from(charset_len.checked_add(1)?).ok()?;
    let (batch, time, classes) = match shape {
        [time, classes] => (1, *time, *classes),
        [batch, time, classes] => (*batch, *time, *classes),
        _ => return None,
    };
    if batch == 0 || time <= 0 || classes != expected_classes {
        return None;
    }
    let time = usize::try_from(time).ok()?;
    time.checked_mul(charset_len.checked_add(1)?)?;
    Some(time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn fixed_time_is_an_upper_bound_for_batched_and_legacy_logits() {
        assert_eq!(static_frame_limit(&[1, 31, 7], 6), Some(31));
        assert_eq!(static_frame_limit(&[31, 7], 6), Some(31));
        assert_eq!(static_frame_limit(&[-1, 31, 7], 6), Some(31));
        assert_eq!(static_frame_limit(&[4, 101, 7], 6), Some(101));
    }

    #[test]
    fn unknown_time_classes_and_invalid_shapes_do_not_get_a_guessed_bound() {
        for shape in [
            vec![],
            vec![31],
            vec![1, 31, 7, 1],
            vec![1, -1, 7],
            vec![1, 0, 7],
            vec![1, 31, -1],
            vec![1, 31, 0],
            vec![1, 31, 8],
            vec![0, 31, 7],
        ] {
            assert_eq!(static_frame_limit(&shape, 6), None, "{shape:?}");
        }
        assert_eq!(static_frame_limit(&[1, i64::MAX, 7], 6), None);
        assert_eq!(static_frame_limit(&[1, 31, 7], usize::MAX), None);
        assert_eq!(static_frame_limit(&[1, 31, 7], i64::MAX as usize), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn dynamic_batch_does_not_hide_a_fixed_time_axis(time in 1i64..8193, chars in 1usize..100000, batch in prop_oneof![Just(-1i64),1i64..65]) {
            prop_assert_eq!(static_frame_limit(&[batch,time,chars as i64+1],chars),Some(time as usize));
            prop_assert_eq!(static_frame_limit(&[batch,-1,chars as i64+1],chars),None);
        }
    }
}
