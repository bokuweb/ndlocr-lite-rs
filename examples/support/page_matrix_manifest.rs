use anyhow::{Result, ensure};
use serde::Deserialize;
use std::{collections::HashSet, path::PathBuf};

#[derive(Deserialize)]
pub struct Input {
    pub id: String,
    pub image: PathBuf,
}

#[derive(Deserialize)]
struct Manifest {
    inputs: Vec<Input>,
}

pub fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}

/// 画像だけを読む。重複/不正なIDは、既存予測の上書きにつながるため拒否する。
pub fn parse_manifest(bytes: &[u8]) -> Result<Vec<Input>> {
    let manifest: Manifest = serde_json::from_slice(bytes)?;
    let mut ids = HashSet::new();
    ensure!(
        !manifest.inputs.is_empty(),
        "manifest inputs must not be empty"
    );
    for row in &manifest.inputs {
        ensure!(safe_id(&row.id), "unsafe input id: {:?}", row.id);
        ensure!(ids.insert(&row.id), "duplicate input id: {:?}", row.id);
        ensure!(
            !row.image.as_os_str().is_empty(),
            "empty image path: {}",
            row.id
        );
    }
    Ok(manifest.inputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 診断データの欠落・上書き・出力先からの逸脱を推論前に拒否する。
    #[test]
    fn rejects_empty_duplicate_and_unsafe_manifest_ids() {
        for input in [
            r#"{"inputs":[]}"#,
            r#"{"inputs":[{"id":"same","image":"a.png"},{"id":"same","image":"b.png"}]}"#,
            r#"{"inputs":[{"id":"","image":"a.png"}]}"#,
            r#"{"inputs":[{"id":"../escape","image":"a.png"}]}"#,
            r#"{"inputs":[{"id":"a/b","image":"a.png"}]}"#,
            r#"{"inputs":[{"id":"a\\b","image":"a.png"}]}"#,
            r#"{"inputs":[{"id":"missing_image"}]}"#,
        ] {
            assert!(parse_manifest(input.as_bytes()).is_err(), "{input}");
        }
    }

    /// 認識に渡す画像パスを保持し、真値や過去の予測を入力へ取り込まない。
    #[test]
    fn preserves_images_and_ignores_truth_and_prior_predictions() {
        let rows = parse_manifest(r#"{"inputs":[{"id":"page_01-a","image":"資料/頁.png","truth":"do not use","after":"do not use"}]}"#.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "page_01-a");
        assert_eq!(rows[0].image, std::path::PathBuf::from("資料/頁.png"));
    }
}
