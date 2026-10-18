//! 明示的に要求された横書きchunkだけで原画像と縮小余白を比較する。
use super::parseq::RecognizeResult;
use crate::pipeline::horizontal_margins::compact_horizontal_margins;
use anyhow::{Result, ensure};

/// 両方の認識結果を保持する、実験用opt-inの候補比較。
#[derive(Debug)]
pub struct MarginCandidates {
    /// 必ず先に認識する原画像の出力。
    original: RecognizeResult,
    /// 画像が変化した場合だけ認識する縮小画像の出力。
    compact: Option<RecognizeResult>,
}
impl MarginCandidates {
    /// 元画像の出力を変更せず返す。
    pub fn original(&self) -> &RecognizeResult {
        &self.original
    }
    /// 追加推論した場合の出力を返す。
    pub fn compact(&self) -> Option<&RecognizeResult> {
        self.compact.as_ref()
    }
    /// 平均信頼度が厳密に高い場合だけ縮小候補を返す。同点は元画像。
    ///
    /// 平均信頼度は校正済みの正答確率ではなく、この選択は精度改善を保証しない。
    /// 実画像では信頼度上昇を伴う `AI` → `Al` の退行が確認されている。
    /// 採用判断には [`Self::original`] と [`Self::compact`] の両結果を保持して評価する。
    pub fn selected(&self) -> &RecognizeResult {
        self.compact
            .as_ref()
            .filter(|candidate| candidate.mean_confidence > self.original.mean_confidence)
            .unwrap_or(&self.original)
    }
}
/// 同じ推論callbackで原画像と縮小限定余白を比較する。既定pipelineへは適用しない。
pub fn recognize_horizontal_margin_candidates<F>(
    rgb: &[u8],
    width: usize,
    height: usize,
    mut recognize: F,
) -> Result<MarginCandidates>
where
    F: FnMut(&[u8], usize, usize) -> Result<RecognizeResult>,
{
    ensure!(width > 0 && height > 0, "image dimensions must be nonzero");
    let expected = width.checked_mul(height).and_then(|v| v.checked_mul(3));
    ensure!(
        expected == Some(rgb.len()),
        "RGB length does not match dimensions"
    );
    let original = recognize(rgb, width, height)?;
    validate_confidence(&original)?;
    let mut compact = None;
    if height
        .checked_mul(5)
        .is_some_and(|minimum| width >= minimum)
    {
        let prepared = compact_horizontal_margins(rgb, width, height, height / 4)?;
        let image = prepared.image;
        if image.width != width || image.height != height || image.data != rgb {
            let candidate = recognize(&image.data, image.width, image.height)?;
            validate_confidence(&candidate)?;
            compact = Some(candidate);
        }
    }
    Ok(MarginCandidates { original, compact })
}

/// 比較不能な信頼度を認識成功として扱わない。
fn validate_confidence(result: &RecognizeResult) -> Result<()> {
    ensure!(
        result.mean_confidence.is_finite() && (0.0..=1.0).contains(&result.mean_confidence),
        "mean confidence must be finite and in [0, 1]"
    );
    Ok(())
}
