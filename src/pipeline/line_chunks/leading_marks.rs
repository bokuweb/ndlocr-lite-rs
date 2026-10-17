use crate::pipeline::crop::CroppedImage;

/// 行頭の孤立した小さい上部画線へ、元画素を保って上下の文脈を加える。
pub(super) fn preserve_leading_mark(image: CroppedImage) -> CroppedImage {
    let (width, height) = (image.width, image.height);
    if height < 16 || width as u128 * 4 < height as u128 * 5 {
        return image;
    }
    let limit = (height / 2).min(width);
    let ink = |x: usize, y: usize| {
        let pixel = &image.data[(y * width + x) * 3..(y * width + x + 1) * 3];
        pixel.iter().map(|&v| u16::from(v)).sum::<u16>() < 220 * 3
    };
    let seed = (0..limit).find_map(|x| (0..height).find(|&y| ink(x, y)).map(|y| (x, y)));
    let Some((sx, sy)) = seed else {
        return image;
    };
    if sx > height / 4 {
        return image;
    }
    let mut visited = vec![false; limit * height];
    let mut pending = vec![(sx, sy)];
    visited[sy * limit + sx] = true;
    let (mut x1, mut y0, mut y1, mut area) = (sx + 1, sy, sy + 1, 0);
    while let Some((x, y)) = pending.pop() {
        area += 1;
        x1 = x1.max(x + 1);
        y0 = y0.min(y);
        y1 = y1.max(y + 1);
        if x1 - sx >= height.div_ceil(4) || y1 >= height.div_ceil(2) {
            return image;
        }
        for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
            for nx in x.saturating_sub(1)..=(x + 1).min(limit - 1) {
                let index = ny * limit + nx;
                if !visited[index] && ink(nx, ny) {
                    visited[index] = true;
                    pending.push((nx, ny));
                }
            }
        }
    }
    if area < 3
        || y0 > height / 8
        || y1 - y0 >= height / 2
        || (sx..x1).any(|x| (0..height).any(|y| ink(x, y) && !visited[y * limit + x]))
        || !(x1..width).any(|x| (height / 2..height).any(|y| ink(x, y)))
    {
        return image;
    }
    let padding = height.div_ceil(32);
    let new_height = height + 2 * padding;
    if width as u128 * 4 < new_height as u128 * 5 {
        return image;
    }
    let mut data = vec![255; width * new_height * 3];
    data[padding * width * 3..(padding + height) * width * 3].copy_from_slice(&image.data);
    CroppedImage {
        data,
        width,
        height: new_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 上部の小画線と、それに重ならない本文を用意する。
    fn sample() -> CroppedImage {
        let (width, height) = (160, 32);
        let mut data = vec![255; width * height * 3];
        for (xs, ys) in [(2..5, 1..8), (10..150, 4..30)] {
            for y in ys {
                for x in xs.clone() {
                    data[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                }
            }
        }
        CroppedImage {
            data,
            width,
            height,
        }
    }

    /// 小画線を失わず、上下へ一列ずつの白余白を加える。
    #[test]
    fn small_leading_mark_keeps_pixels_with_vertical_context() {
        let original = sample();
        let actual = preserve_leading_mark(original.clone());
        assert_eq!((actual.width, actual.height), (160, 34));
        assert_eq!(&actual.data[160 * 3..160 * 33 * 3], &original.data);
        assert!(actual.data[..160 * 3].iter().all(|&v| v == 255));
        assert!(actual.data[160 * 33 * 3..].iter().all(|&v| v == 255));
    }

    /// 斜めの小画線は左端画素の高さではなく、成分全体の上端で判定する。
    #[test]
    fn slanted_mark_uses_component_top() {
        let mut image = sample();
        for y in 1..8 {
            image.data[(y * 160 + 2) * 3..(y * 160 + 3) * 3].fill(255);
        }
        image.data[(7 * 160 + 2) * 3..(7 * 160 + 3) * 3].fill(0);
        assert_eq!(preserve_leading_mark(image).height, 34);
    }

    /// 点の下の軸・本文へ繋がる画線・単独記号は変更しない。
    #[test]
    fn unsupported_mark_preserves_image() {
        for case in 0..3 {
            let mut image = sample();
            match case {
                0 => {
                    for y in 16..30 {
                        image.data[(y * image.width + 3) * 3] = 0;
                    }
                }
                1 => {
                    for x in 4..11 {
                        image.data[(4 * image.width + x) * 3] = 0;
                    }
                }
                2 => {
                    for y in 0..32 {
                        image.data[(y * 160 + 10) * 3..(y * 160 + 160) * 3].fill(255);
                    }
                }
                _ => unreachable!(),
            }
            assert_eq!(preserve_leading_mark(image.clone()), image, "case={case}");
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(24))]
        /// 整数拡大と左余白の移動でも、全画素と回転分類を維持する。
        #[test]
        fn leading_mark_context_preserves_scaled_pixels(scale in 1usize..4, margin in 0usize..5) {
            let source = sample();
            let (width,height) = (source.width*scale+margin,source.height*scale);
            let mut data=vec![255;width*height*3];
            for y in 0..height { for x in 0..source.width*scale {
                let start=(y*width+x+margin)*3;
                let original=((y/scale)*source.width+x/scale)*3;
                data[start..start+3].copy_from_slice(&source.data[original..original+3]);
            }}
            let image=CroppedImage { data, width, height };
            let actual=preserve_leading_mark(image.clone());
            proptest::prop_assert_eq!(actual.width,width);
            proptest::prop_assert_eq!(actual.height,height+2*scale);
            proptest::prop_assert_eq!(&actual.data[scale*width*3..(height+scale)*width*3],&image.data);
            proptest::prop_assert_eq!(height*5>width*4,actual.height*5>actual.width*4);
        }
    }
}
