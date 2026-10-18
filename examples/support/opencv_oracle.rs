//! 製品から独立した OpenCV oracle の通信境界。
use anyhow::{Context, Result, ensure};
use std::{
    io::{Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

/// 応答を必ず回収する診断用 worker。
pub struct Oracle {
    /// Python 子プロセス。
    child: Child,
    /// RGB 入力パイプ。
    input: ChildStdin,
    /// 正規化 tensor 応答パイプ。
    output: ChildStdout,
}

impl Oracle {
    /// 環境変数で指定した専用 Python から固定 script を起動する。
    pub fn start() -> Result<Self> {
        let python = std::env::var_os("NDLOCR_DIAGNOSTIC_PYTHON")
            .context("set NDLOCR_DIAGNOSTIC_PYTHON to the pinned oracle Python")?;
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/support/opencv_worker.py");
        let mut child = Command::new(python)
            .arg("-u")
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().context("missing worker stdin")?;
        let output = child.stdout.take().context("missing worker stdout")?;
        Ok(Self {
            child,
            input,
            output,
        })
    }

    /// 同一 RGB を uint8 リサイズ後の BGR/NCHW tensor にする。
    pub fn preprocess(
        &mut self,
        rgb: &[u8],
        w: usize,
        h: usize,
        iw: usize,
        ih: usize,
    ) -> Result<Vec<f32>> {
        exchange(&mut self.input, &mut self.output, rgb, w, h, iw, ih)
    }
}

impl Drop for Oracle {
    /// 推論失敗時も子プロセスを残さず回収する。
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 確保と送信の前に寸法を検証し、応答長・有限性・範囲を確認する。
fn exchange(
    input: &mut impl Write,
    output: &mut impl Read,
    rgb: &[u8],
    w: usize,
    h: usize,
    iw: usize,
    ih: usize,
) -> Result<Vec<f32>> {
    let dimensions = [w, h, iw, ih].map(u32::try_from);
    ensure!(
        dimensions.iter().all(|d| matches!(d, Ok(n) if *n > 0)),
        "invalid dimensions"
    );
    let source = w
        .checked_mul(h)
        .and_then(|n| n.checked_mul(3))
        .context("source overflow")?;
    ensure!(source == rgb.len(), "invalid RGB length");
    let len = iw
        .checked_mul(ih)
        .and_then(|n| n.checked_mul(3))
        .context("tensor overflow")?;
    let bytes = len
        .checked_mul(4)
        .filter(|n| *n <= isize::MAX as usize)
        .context("tensor byte overflow")?;
    for dimension in dimensions {
        input.write_all(&dimension?.to_le_bytes())?;
    }
    input.write_all(rgb)?;
    input.flush()?;
    let mut header = [0; 8];
    output
        .read_exact(&mut header)
        .context("worker ended before response header")?;
    ensure!(
        u64::from_le_bytes(header) == u64::try_from(bytes)?,
        "unexpected response length"
    );
    let mut data = Vec::new();
    data.try_reserve_exact(bytes)?;
    data.resize(bytes, 0);
    output
        .read_exact(&mut data)
        .context("worker ended during tensor")?;
    let tensor: Vec<f32> = data
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    ensure!(
        tensor
            .iter()
            .all(|v| v.is_finite() && (-1.0..=1.0).contains(v)),
        "invalid normalized tensor"
    );
    Ok(tensor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// 独立した整数画素 fixture と既存の浮動小数点補間との差を記録する。
    #[test]
    fn independent_pixels_expose_float_difference() {
        use ndlocr_lite_rs::infer::parseq::{
            ParseqOrientation, preprocess_rgb_u8_with_orientation,
        };
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/opencv_resize/pixels.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let dims: Vec<usize> = case["dimensions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap() as usize)
                .collect();
            let rgb: Vec<u8> = case["rgb"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap() as u8)
                .collect();
            let got = preprocess_rgb_u8_with_orientation(
                &rgb,
                dims[0],
                dims[1],
                dims[2],
                dims[3],
                ParseqOrientation::Horizontal,
            )
            .unwrap();
            let pixels = case["resized_rgb"].as_array().unwrap();
            let plane = dims[2] * dims[3];
            let expected: Vec<f32> = (0..3)
                .flat_map(|c| {
                    (0..plane)
                        .map(move |i| pixels[i * 3 + 2 - c].as_u64().unwrap() as f32 / 127.5 - 1.0)
                })
                .collect();
            let differences: Vec<f32> = got
                .iter()
                .zip(expected)
                .map(|(a, b)| (a - b).abs())
                .collect();
            let count = differences.iter().filter(|d| **d > 0.0).count();
            println!(
                "{}",
                serde_json::json!({"id":case["id"],"different_elements":count,
                "elements":got.len(),"maximum_pixel_delta":differences.iter().copied().fold(0.0f32,f32::max)*127.5})
            );
            if case["id"] == "identity" {
                assert_eq!(count, 0);
            }
            if case["id"] == "fractional" {
                assert!(count > 0);
            }
        }
    }

    /// 不正な入力は worker へ送信する前に拒否する。
    #[test]
    fn invalid_input_never_writes() {
        for (rgb, w, h, iw, ih) in [
            (&[0u8][..], 1, 1, 1, 1),
            (&[][..], 0, 1, 1, 1),
            (&[0; 3][..], 1, 1, 0, 1),
            (&[0; 3][..], 1, 1, usize::MAX, 2),
        ] {
            let mut sent = Vec::new();
            assert!(exchange(&mut sent, &mut Cursor::new([]), rgb, w, h, iw, ih).is_err());
            assert!(sent.is_empty());
        }
    }

    /// 応答の長さ・途中 EOF・NaN・範囲外を黙って通さない。
    #[test]
    fn malformed_reply_is_rejected() {
        let mut replies = vec![
            vec![],
            8u64.to_le_bytes().to_vec(),
            12u64.to_le_bytes().to_vec(),
        ];
        for bad in [f32::NAN, f32::INFINITY, 1.01, -1.01] {
            let mut reply = 12u64.to_le_bytes().to_vec();
            for value in [bad, 0.0, 0.0] {
                reply.extend(value.to_le_bytes());
            }
            replies.push(reply);
        }
        for reply in replies {
            assert!(
                exchange(
                    &mut Vec::new(),
                    &mut Cursor::new(reply),
                    &[1, 2, 3],
                    1,
                    1,
                    1,
                    1
                )
                .is_err()
            );
        }
    }

    /// little endian の RGB 入力と正規化済み BGR 応答を保持する。
    #[test]
    fn protocol_preserves_header_pixels_and_tensor() {
        let mut reply = 12u64.to_le_bytes().to_vec();
        for value in [-1.0f32, 0.0, 1.0] {
            reply.extend(value.to_le_bytes());
        }
        let mut sent = Vec::new();
        let got = exchange(
            &mut sent,
            &mut Cursor::new(reply),
            &[255, 128, 0],
            1,
            1,
            1,
            1,
        )
        .unwrap();
        assert_eq!(got, [-1.0, 0.0, 1.0]);
        assert_eq!(&sent[..16], &[1, 0, 0, 0].repeat(4));
        assert_eq!(&sent[16..], &[255, 128, 0]);
    }
}
