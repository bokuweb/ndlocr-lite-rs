//! Fixed output bounds; dynamic or incompatible outputs remain unsupported.
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

pub(super) fn selected_frame_limit<'a>(
    outputs: impl IntoIterator<Item = (&'a str, Option<&'a [i64]>)>,
    selected: &str,
    charset_len: usize,
) -> Option<usize> {
    let (_, shape) = outputs.into_iter().find(|(name, _)| *name == selected)?;
    static_frame_limit(shape?, charset_len)
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

    #[test]
    fn selected_output_name_controls_the_fixed_frame_limit() {
        let (short, long) = ([1, 31, 7], [1, 51, 7]);
        let outputs = [
            ("aux", Some(short.as_slice())),
            ("main", Some(long.as_slice())),
        ];
        assert_eq!(selected_frame_limit(outputs, "main", 6), Some(51));
        assert_eq!(selected_frame_limit(outputs, "aux", 6), Some(31));
    }

    #[test]
    fn selected_dynamic_invalid_or_missing_shape_never_falls_back_to_another_output() {
        let fixed = [1, 31, 7];
        for invalid in [
            vec![],
            vec![1, -1, 7],
            vec![1, 0, 7],
            vec![1, 31, 8],
            vec![0, 31, 7],
        ] {
            let outputs = [
                ("aux", Some(fixed.as_slice())),
                ("main", Some(invalid.as_slice())),
            ];
            assert_eq!(selected_frame_limit(outputs, "main", 6), None);
        }
        assert_eq!(
            selected_frame_limit([("main", None), ("aux", Some(fixed.as_slice()))], "main", 6),
            None
        );
        assert_eq!(
            selected_frame_limit([("aux", Some(fixed.as_slice()))], "missing", 6),
            None
        );
    }

    #[test]
    fn a_duplicate_selected_name_cannot_hide_the_first_missing_shape() {
        let fixed = [1, 31, 7];
        assert_eq!(
            selected_frame_limit(
                [("main", None), ("main", Some(fixed.as_slice()))],
                "main",
                6
            ),
            None
        );
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
