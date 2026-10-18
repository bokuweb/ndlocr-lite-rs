//! 縮小軸の被覆面積を積算する診断用前処理。

/// 縮小軸を面積平均し、拡大軸だけ中心座標の線形補間でBGR/NCHWを作る。
pub fn preprocess(
    rgb: &[u8],
    width: usize,
    height: usize,
    out_w: usize,
    out_h: usize,
) -> anyhow::Result<Vec<f32>> {
    use anyhow::{Context, ensure};
    ensure!(
        width > 0 && height > 0 && out_w > 0 && out_h > 0,
        "zero dimension"
    );
    let source = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(3))
        .context("source overflow")?;
    ensure!(rgb.len() == source, "RGB length mismatch");
    let len = out_w
        .checked_mul(out_h)
        .and_then(|n| n.checked_mul(3))
        .context("output overflow")?;
    let bytes = len
        .checked_mul(size_of::<f32>())
        .context("output byte overflow")?;
    ensure!(bytes <= isize::MAX as usize, "output exceeds capacity");
    let mut out = Vec::new();
    out.try_reserve_exact(len)?;
    out.resize(len, 0.0);
    let xs = weights(width, out_w);
    let ys = weights(height, out_h);
    let plane = out_w * out_h;
    for (y, wy) in ys.iter().enumerate() {
        for (x, wx) in xs.iter().enumerate() {
            for channel in 0..3 {
                let mut sum = 0.0;
                for &(sy, dy) in wy {
                    for &(sx, dx) in wx {
                        sum += f64::from(rgb[(sy * width + sx) * 3 + 2 - channel]) * dx * dy;
                    }
                }
                out[channel * plane + y * out_w + x] = (sum / 127.5 - 1.0) as f32;
            }
        }
    }
    Ok(out)
}

/// 出力画素の被覆率、または既存方式と同じ拡大座標から重みを作る。
fn weights(source: usize, destination: usize) -> Vec<Vec<(usize, f64)>> {
    (0..destination)
        .map(|i| {
            if source > destination {
                let start = i as f64 * source as f64 / destination as f64;
                let end = (i + 1) as f64 * source as f64 / destination as f64;
                (start.floor() as usize..(end.ceil() as usize).min(source))
                    .map(|s| {
                        (
                            s,
                            (end.min((s + 1) as f64) - start.max(s as f64)) / (end - start),
                        )
                    })
                    .collect()
            } else {
                let coord = if destination <= 1 {
                    0.0
                } else {
                    (((i as f32 + 0.5) * source as f32 / destination as f32) - 0.5)
                        .clamp(0.0, (source - 1) as f32)
                };
                let lo = coord.floor() as usize;
                let hi = (lo + 1).min(source - 1);
                let fraction = f64::from(coord - lo as f32);
                vec![(lo, 1.0 - fraction), (hi, fraction)]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    /// 非整数の縮小率でも境界画素の寄与を分け、BGR各planeを保つ。
    #[test]
    fn fractional_boundaries_and_bgr_planes() {
        let rgb = [0, 10, 20, 255, 10, 20, 0, 10, 20, 255, 10, 20];
        let out = super::preprocess(&rgb, 4, 1, 3, 1).unwrap();
        let expected = [20.0, 20.0, 20.0, 10.0, 10.0, 10.0, 63.75, 127.5, 191.25];
        for (actual, value) in out.iter().zip(expected) {
            assert!((actual - (value / 127.5 - 1.0)).abs() < 1e-6);
        }
    }

    /// 拡大軸の座標と正規化は既存のHorizontal前処理と揃える。
    #[test]
    fn identity_and_upsampling_match_existing() {
        use ndlocr_lite_rs::infer::parseq::{
            ParseqOrientation, preprocess_rgb_u8_with_orientation,
        };
        let rgb: Vec<u8> = (0..18).map(|v| v * 13).collect();
        for (w, h) in [(3, 2), (7, 5)] {
            let expected =
                preprocess_rgb_u8_with_orientation(&rgb, 3, 2, w, h, ParseqOrientation::Horizontal)
                    .unwrap();
            let actual = super::preprocess(&rgb, 3, 2, w, h).unwrap();
            assert!(
                actual
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| (a - b).abs() < 1e-6)
            );
        }
    }

    /// 不正な長さとゼロ・overflowを巨大確保より先に拒否する。
    #[test]
    fn invalid_shapes_are_errors() {
        for (rgb, w, h, ow, oh) in [
            (&[][..], 0, 1, 1, 1),
            (&[255; 3][..], 1, 1, 0, 1),
            (&[255; 2][..], 1, 1, 1, 1),
            (&[255; 4][..], 1, 1, 1, 1),
            (&[][..], usize::MAX, 2, 1, 1),
            (&[255; 3][..], 1, 1, usize::MAX, 2),
        ] {
            assert!(super::preprocess(rgb, w, h, ow, oh).is_err());
        }
    }

    proptest! {
        /// 両軸縮小時に画像全体の各チャンネル平均濃度を保存する。
        #[test]
        fn downsampling_preserves_global_mean(w in 1usize..40, h in 1usize..30,
            dx in 1usize..40, dy in 1usize..30, seed in any::<u64>()) {
            let (ow, oh) = (1 + dx % w, 1 + dy % h);
            let rgb: Vec<u8> = (0..w*h*3).map(|i| (seed.wrapping_add(i as u64 * 73) % 256) as u8).collect();
            let out = super::preprocess(&rgb, w, h, ow, oh).unwrap();
            for c in 0..3 {
                let source = rgb.chunks_exact(3).map(|p| f64::from(p[2-c])).sum::<f64>() / (w*h) as f64;
                let mean = out[c*ow*oh..(c+1)*ow*oh].iter().map(|p| (f64::from(*p)+1.0)*127.5).sum::<f64>() / (ow*oh) as f64;
                prop_assert!((source-mean).abs() < 1e-4);
            }
        }
    }

    /// 出力中心から外れた1画素の黒線も平均濃度へ寄与する。
    #[test]
    fn thin_stroke_survives_downsampling() {
        let mut rgb = vec![255; 7 * 3];
        rgb[3..6].fill(0);
        let out = super::preprocess(&rgb, 7, 1, 1, 1).unwrap();
        for channel in out {
            assert!((channel - (6.0 / 7.0 * 2.0 - 1.0)).abs() < 1e-6);
        }
        use ndlocr_lite_rs::infer::parseq::{
            ParseqOrientation, preprocess_rgb_u8_with_orientation,
        };
        assert_eq!(
            preprocess_rgb_u8_with_orientation(&rgb, 7, 1, 1, 1, ParseqOrientation::Horizontal)
                .unwrap(),
            vec![1.0; 3]
        );
    }
}
