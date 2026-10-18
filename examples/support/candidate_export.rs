//! 評価用候補export。推論と本文補正から独立してJSONの対応を保持する。
use anyhow::{Result, ensure};
use ndlocr_lite_rs::infer::parseq::ParseqCandidateResult;
use serde::Serialize;

/// 原文と対応づける一行の生本文・位置候補・区間診断。
#[derive(Serialize)]
pub struct ExportLine {
    /// manifestで固定した行ID。
    id: String,
    /// 元画像の行座標。
    bounds: Option<[usize; 4]>,
    /// 連結空白だけを補った生本文。
    pub text: String,
    /// Unicode scalarごとの候補。連結空白には代替候補なし。
    positions: Vec<Position>,
    /// 停止ステップを含む区間診断。
    chunks: Vec<Chunk>,
}

/// 本文の一文字と、EOSを含む元の順位順候補。
#[derive(Serialize)]
struct Position {
    /// 実際に出力した文字。
    character: char,
    /// EOSをNoneとして順位数へ残す候補。
    candidates: Vec<Option<char>>,
}

/// 一回のforwardで認識した水平区間。
#[derive(Serialize)]
struct Chunk {
    /// 行内の未crop区間の水平範囲。
    x_range: [usize; 2],
    /// この区間の生本文。
    text: String,
    /// 区間直前に連結空白を補ったか。
    inserted_space_before: bool,
    /// 生デコーダの停止理由。
    stop_reason: String,
    /// 停止時刻を含む全判断。
    steps: Vec<Step>,
}

/// 区間内の一つの判断時刻。
#[derive(Serialize)]
struct Step {
    /// モデルの出力時刻。
    timestep: usize,
    /// 区間本文内のUTF-8バイト範囲。停止時刻はNone。
    byte_range: Option<[usize; 2]>,
    /// 全クラスsoftmax確率を保存した順位順候補。
    candidates: Vec<Candidate>,
}

/// 推論APIの順位・文字・確率を変更せず保存する候補。
#[derive(Serialize)]
struct Candidate {
    /// EOSを0とするクラスID。
    token_id: usize,
    /// EOSだけNone。
    character: Option<char>,
    /// 全クラスsoftmax確率。正解率や再推論スコアではない。
    probability: f32,
}

impl ExportLine {
    /// 空の一行を作る。
    pub fn new(id: String, bounds: [usize; 4]) -> Self {
        Self {
            id,
            bounds: Some(bounds),
            text: String::new(),
            positions: Vec::new(),
            chunks: Vec::new(),
        }
    }

    /// 行検出で欠落した原文行を、候補なしの空予測として残す。
    pub fn missing(id: String) -> Self {
        Self {
            id,
            bounds: None,
            text: String::new(),
            positions: Vec::new(),
            chunks: Vec::new(),
        }
    }

    /// 区間の文字対応を検証し、固定連結空白と生候補を追加する。
    pub fn append(
        &mut self,
        result: ParseqCandidateResult,
        space: bool,
        x_range: [usize; 2],
    ) -> Result<()> {
        let mut positions = Vec::new();
        let mut cursor = 0;
        for step in &result.steps {
            if let Some(range) = &step.text_range {
                let slice = result.recognition.text.get(range.clone());
                let winner = step.candidates.first().and_then(|c| c.character);
                ensure!(
                    range.start == cursor
                        && slice
                            .is_some_and(|s| s.chars().count() == 1 && s.chars().next() == winner),
                    "candidate/text range mismatch"
                );
                cursor = range.end;
                positions.push(Position {
                    character: winner.unwrap(),
                    candidates: step.candidates.iter().map(|c| c.character).collect(),
                });
            }
        }
        ensure!(
            cursor == result.recognition.text.len(),
            "incomplete text mapping"
        );
        if space {
            self.text.push(' ');
            self.positions.push(Position {
                character: ' ',
                candidates: vec![Some(' ')],
            });
        }
        self.text.push_str(&result.recognition.text);
        self.positions.extend(positions);
        self.chunks.push(Chunk {
            x_range,
            text: result.recognition.text,
            inserted_space_before: space,
            stop_reason: format!("{:?}", result.stop_reason),
            steps: result
                .steps
                .into_iter()
                .map(|s| Step {
                    timestep: s.timestep,
                    byte_range: s.text_range.map(|r| [r.start, r.end]),
                    candidates: s
                        .candidates
                        .into_iter()
                        .map(|c| Candidate {
                            token_id: c.token_id,
                            character: c.character,
                            probability: c.probability,
                        })
                        .collect(),
                })
                .collect(),
        });
        Ok(())
    }
}
