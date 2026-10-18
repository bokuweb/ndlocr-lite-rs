//! 文脈補正で参照できる、後処理前のトークン候補。
use super::{RecognizeResult, decode_flat_logits_with_observer};
use anyhow::{Result, bail};
use std::ops::Range;

/// 認識の打ち切り理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseqStopReason {
    /// 終端トークンを選択した。
    EndOfSequence,
    /// 既存デコーダの低信頼度条件で打ち切った。
    LowConfidence,
    /// モデルが返した全ステップを処理した。
    TimestepsExhausted,
}

/// 全クラスに対するsoftmax確率を持つ候補。
#[derive(Debug, Clone, PartialEq)]
pub struct ParseqCandidate {
    /// モデル出力のクラス番号。0はEOS。
    pub token_id: usize,
    /// EOSの場合だけNone。それ以外は文字集合の対応文字。
    pub character: Option<char>,
    /// 全クラスで正規化した確率。OCR正解率として校正した値ではない。
    pub probability: f32,
}

/// 一回の推論に含まれる一ステップの候補。
#[derive(Debug, Clone, PartialEq)]
pub struct ParseqStepCandidates {
    /// モデル出力のステップ番号。
    pub timestep: usize,
    /// 本文に出力した文字のUTF-8バイト範囲。停止ステップはNone。
    pub text_range: Option<Range<usize>>,
    /// logit降順、logit同率ならクラス番号昇順の上位候補。
    pub candidates: Vec<ParseqCandidate>,
}

/// 従来と同じ認識結果と、その判断に使用した候補。
#[derive(Debug, Clone)]
pub struct ParseqCandidateResult {
    /// 反復や空白を補正していない本文と既存の平均信頼度。
    pub recognition: RecognizeResult,
    /// 停止ステップまでの候補。停止後の推測は含まない。
    pub steps: Vec<ParseqStepCandidates>,
    /// 本文の出力を停止した理由。
    pub stop_reason: ParseqStopReason,
}

/// デコーダ内部で確定した一ステップ。通常経路では保存しない。
pub(super) struct DecodeStep<'a> {
    /// 出力ステップ番号。
    pub timestep: usize,
    /// 全クラスのlogit。
    pub row: &'a [f32],
    /// 既存デコーダが選択したクラス番号。
    pub winner: usize,
    /// 選択したクラスの全クラスsoftmax確率。
    pub probability: f32,
    /// 実際に出力した文字のバイト範囲。
    pub text_range: Option<Range<usize>>,
    /// 停止した場合の理由。
    pub stop_reason: Option<ParseqStopReason>,
}

/// 従来の本文を保ったまま、各ステップの上位候補を収集する。
///
/// `top_k`は1以上で、クラス数を超えた分は切り詰める。EOSと低信頼度の
/// 停止ステップも返す。確率は上位候補だけで再正規化しない。
/// 代替文字列をモデルに再入力した場合の系列確率を表すものではない。
///
/// # Errors
/// 形状不正、文字集合とクラス数の不一致、形状内の非有限logit、k=0は拒否する。
/// 形状外の末尾データは従来どおり無視する。
pub fn predict_text_from_flat_logits_with_candidates(
    logits: &[f32],
    timesteps: usize,
    classes: usize,
    charset: &[char],
    top_k: usize,
) -> Result<ParseqCandidateResult> {
    if top_k == 0 {
        bail!("top_k must be > 0");
    }
    let k = top_k.min(classes);
    // 共通デコーダによる形状・文字集合・有限値の検証後にのみ確保する。
    let mut indices: Option<Vec<usize>> = None;
    let mut steps = Vec::new();
    let mut stop_reason = ParseqStopReason::TimestepsExhausted;
    let recognition =
        decode_flat_logits_with_observer(logits, timesteps, classes, charset, |step| {
            let indices = indices.get_or_insert_with(|| (0..classes).collect());
            let compare = |a: &usize, b: &usize| {
                step.row[*b]
                    .partial_cmp(&step.row[*a])
                    .expect("validated finite logits")
                    .then_with(|| a.cmp(b))
            };
            if k < classes {
                indices.select_nth_unstable_by(k, compare);
            }
            indices[..k].sort_unstable_by(compare);
            let candidates = indices[..k]
                .iter()
                .map(|&token_id| ParseqCandidate {
                    token_id,
                    character: token_id.checked_sub(1).map(|id| charset[id]),
                    probability: (step.row[token_id] - step.row[step.winner]).exp()
                        * step.probability,
                })
                .collect();
            if let Some(reason) = step.stop_reason {
                stop_reason = reason;
            }
            steps.push(ParseqStepCandidates {
                timestep: step.timestep,
                text_range: step.text_range,
                candidates,
            });
        })?;
    Ok(ParseqCandidateResult {
        recognition,
        steps,
        stop_reason,
    })
}
