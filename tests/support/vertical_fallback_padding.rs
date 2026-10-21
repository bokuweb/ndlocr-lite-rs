use super::*;
use crate::infer::parseq::RecognizeResult;

fn column() -> (Vec<u8>, usize, usize) {
    let (w, h) = (20, 248);
    let mut rgb = vec![255; w * h * 3];
    for group in 0..12 {
        let top = 4 + group * 20;
        for y in top..top + 16 {
            for x in 2..18 {
                if x == 2 || x == 17 || y == top || y == top + 15 {
                    let p = (y * w + x) * 3;
                    rgb[p..p + 3].copy_from_slice(&[10, 40, 90]);
                }
            }
        }
    }
    (rgb, w, h)
}
fn recognized() -> RecognizeResult {
    RecognizeResult {
        text: "あいうえおかきくけこさし".to_string(),
        mean_confidence: 0.97,
    }
}
#[test]
fn zero_padding_preserves_existing_page_bounds_text_and_confidence_bits() {
    let (rgb, w, h) = column();
    let original = rgb.clone();
    let old = try_recognize(&rgb, w, h, 0.3, |_, _, _| Ok(recognized())).unwrap();
    let mut calls = 0;
    let actual = try_recognize_with_padding(&rgb, w, h, 0.3, 0, |pixels, cw, ch| {
        calls += 1;
        assert_eq!((cw, ch), (18, 238));
        let independent: Vec<_> = (3..241)
            .flat_map(|y| rgb[(y * w + 1) * 3..(y * w + 19) * 3].iter().copied())
            .collect();
        assert_eq!(pixels, independent);
        Ok(recognized())
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(actual.0, old.0);
    assert_eq!(actual.1.text, old.1.text);
    assert_eq!(
        actual.1.mean_confidence.to_bits(),
        old.1.mean_confidence.to_bits()
    );
    assert_eq!(rgb, original);
}
#[test]
fn requested_padding_clamps_original_rgb_and_keeps_nn_and_reported_bounds_identical() {
    let (rgb, w, h) = column();
    let original = rgb.clone();
    let mut calls = 0;
    let actual = try_recognize_with_padding(&rgb, w, h, 0.3, 3, |pixels, cw, ch| {
        calls += 1;
        assert_eq!((cw, ch), (20, 244));
        assert_eq!(pixels, &rgb[..w * 244 * 3]);
        Ok(recognized())
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(actual.0, [0, 0, 20, 244]);
    assert_eq!(actual.1.text, recognized().text);
    assert_eq!(rgb, original);
}

#[test]
fn unbounded_requested_padding_covers_the_original_image_without_integer_overflow() {
    let (rgb, w, h) = column();
    let mut calls = 0;
    let actual = try_recognize_with_padding(&rgb, w, h, 0.3, usize::MAX, |pixels, cw, ch| {
        calls += 1;
        assert_eq!((cw, ch), (w, h));
        assert_eq!(pixels, rgb);
        Ok(recognized())
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(actual.0, [0, 0, w as i32, h as i32]);
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn translated_colored_glyphs_keep_all_original_bytes_at_clamped_padding(
        groups in 12usize..40, advance in 20usize..35,
        left in 0usize..4, right in 0usize..4, top in 0usize..5, bottom in 0usize..5,
        padding in 0usize..65536,
    ) {
        let (w,h)=(20+left+right, groups*advance+8+top+bottom);
        let mut rgb=vec![255;w*h*3];
        for group in 0..groups {
            let y0=4+top+group*advance;
            for y in y0..y0+16 {
                for x in left+2..left+18 {
                    if x==left+2 || x==left+17 || y==y0 || y==y0+15 {
                        rgb[(y*w+x)*3..(y*w+x+1)*3].copy_from_slice(&[10+(group%50) as u8,40,90]);
                    }
                }
            }
        }
        let expected=[(left+1).saturating_sub(padding),(top+3).saturating_sub(padding),
                      (left+19+padding).min(w),(top+21+(groups-1)*advance+padding).min(h)];
        let before=rgb.clone();let text="あいうえおか".repeat(groups);let mut called=false;
        let actual=try_recognize_with_padding(&rgb,w,h,0.3,padding,|pixels,cw,ch| {
            called=true;
            assert_eq!((cw,ch),(expected[2]-expected[0],expected[3]-expected[1]));
            let independent:Vec<_>=(expected[1]..expected[3])
                .flat_map(|y|rgb[(y*w+expected[0])*3..(y*w+expected[2])*3].iter().copied()).collect();
            assert_eq!(pixels,independent);
            Ok(RecognizeResult{text:text.clone(),mean_confidence:0.97})
        }).expect("supported translated outline column");
        prop_assert!(called);prop_assert_eq!(actual.0,expected.map(|x|x as i32));
        prop_assert_eq!(actual.1.text,text);prop_assert_eq!(rgb,before);
    }
}
