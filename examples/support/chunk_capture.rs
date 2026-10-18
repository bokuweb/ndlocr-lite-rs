//! 候補解析用に認識直前のRGBを可逆保存する。
use anyhow::{Result, ensure};
use ndlocr_lite_rs::pipeline::crop::CroppedImage;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// 一回のexport専用の保存先と入力の対応。
pub struct InputCapture {
    /// 既存データを上書きしない専用ディレクトリ。
    directory: PathBuf,
    /// 実モデルの入力幅・高さ。
    model_input_dimensions: (usize, usize),
    /// 認識順に保存した区間。
    chunks: Vec<Chunk>,
}

/// PNGと元の行・区間の対応。
#[derive(Serialize)]
struct Chunk {
    /// 保存先内の連番ファイル名。
    image: String,
    /// manifestの行ID。パスとしては使わない。
    line_id: String,
    /// 行内の区間番号。
    chunk_index: usize,
    /// crop前の行内水平範囲。
    x_range: [usize; 2],
    /// 認識APIへ渡したRGBの幅・高さ。
    dimensions: [usize; 2],
}

impl InputCapture {
    /// 新しいディレクトリを作り、既存ファイルへの保存を拒否する。
    pub fn new(directory: &Path, model_input_dimensions: (usize, usize)) -> Result<Self> {
        std::fs::create_dir(directory)?;
        Ok(Self {
            directory: directory.to_owned(),
            model_input_dimensions,
            chunks: Vec::new(),
        })
    }

    /// 認識に使う同じRGBを変更せず、可逆PNGへ保存する。
    pub fn push(
        &mut self,
        line_id: &str,
        chunk_index: usize,
        x_range: [usize; 2],
        rgb: &CroppedImage,
    ) -> Result<()> {
        let expected = rgb
            .width
            .checked_mul(rgb.height)
            .and_then(|n| n.checked_mul(3));
        ensure!(
            rgb.width > 0 && rgb.height > 0 && expected == Some(rgb.data.len()),
            "invalid RGB shape"
        );
        let width = u32::try_from(rgb.width)?;
        let height = u32::try_from(rgb.height)?;
        let image = format!("{:06}.png", self.chunks.len());
        image::save_buffer(
            self.directory.join(&image),
            &rgb.data,
            width,
            height,
            image::ColorType::Rgb8,
        )?;
        self.chunks.push(Chunk {
            image,
            line_id: line_id.to_owned(),
            chunk_index,
            x_range,
            dimensions: [rgb.width, rgb.height],
        });
        Ok(())
    }

    /// 全推論の完了後にだけ、実入力の対応表を保存する。
    pub fn finish(self) -> Result<()> {
        let manifest = serde_json::json!({"model_input_dimensions":self.model_input_dimensions, "chunks":self.chunks});
        std::fs::write(
            self.directory.join("inputs.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndlocr_lite_rs::io::load_rgb_u8;
    use proptest::prelude::*;

    /// 淡い色と白余白を含む実入力とモデル寸法をそのまま保存する。
    #[test]
    fn round_trips_rgb_and_manifest_without_using_id_as_path() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("capture");
        let mut capture = InputCapture::new(&dir, (768, 24)).unwrap();
        let rgb = CroppedImage {
            width: 2,
            height: 1,
            data: vec![255, 254, 255, 255, 255, 255],
        };
        capture
            .push("../../line/日本語", 3, [10, 12], &rgb)
            .unwrap();
        assert!(!dir.join("inputs.json").exists());
        capture.finish().unwrap();
        let image = load_rgb_u8(&dir.join("000000.png")).unwrap();
        assert_eq!((image.width, image.height, image.data), (2, 1, rgb.data));
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("inputs.json")).unwrap()).unwrap();
        assert_eq!(
            manifest["model_input_dimensions"],
            serde_json::json!([768, 24])
        );
        assert_eq!(
            manifest["chunks"][0],
            serde_json::json!({"image":"000000.png", "line_id":"../../line/日本語", "chunk_index":3, "x_range":[10,12], "dimensions":[2,1]})
        );
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    /// 既存のディレクトリへ追記せず、保存済みデータを保護する。
    #[test]
    fn refuses_existing_directory() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("sentinel"), b"keep").unwrap();
        assert!(InputCapture::new(temp.path(), (256, 24)).is_err());
        assert_eq!(
            std::fs::read(temp.path().join("sentinel")).unwrap(),
            b"keep"
        );
    }

    /// 不正な寸法やRGB長を保存しない。
    #[test]
    fn rejects_invalid_shapes_before_writing() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("capture");
        let mut capture = InputCapture::new(&dir, (256, 24)).unwrap();
        for (width, height, data) in [
            (0, 1, vec![]),
            (1, 0, vec![]),
            (1, 1, vec![0; 2]),
            (1, 1, vec![0; 4]),
            (usize::MAX, 2, vec![]),
        ] {
            assert!(
                capture
                    .push(
                        "id",
                        0,
                        [0, 1],
                        &CroppedImage {
                            width,
                            height,
                            data
                        }
                    )
                    .is_err()
            );
        }
        assert_eq!(std::fs::read_dir(dir).unwrap().count(), 0);
    }

    proptest! {
        /// 任意の小さいRGBでもPNG往復が完全に一致する。
        #[test]
        fn arbitrary_pixels_survive(width in 1usize..9, height in 1usize..9, bytes in prop::collection::vec(any::<u8>(), 192)) {
            let temp = tempfile::tempdir().unwrap();
            let dir = temp.path().join("capture");
            let mut capture = InputCapture::new(&dir, (384,24)).unwrap();
            let data: Vec<_> = bytes.into_iter().cycle().take(width*height*3).collect();
            let rgb = CroppedImage { width, height, data: data.clone() };
            capture.push("id", 0, [0,width], &rgb).unwrap();
            capture.finish().unwrap();
            let actual = load_rgb_u8(&dir.join("000000.png")).unwrap();
            prop_assert_eq!((actual.width, actual.height, actual.data), (width,height,data));
        }
    }
}
