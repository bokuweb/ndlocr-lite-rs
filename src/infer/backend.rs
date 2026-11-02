//! 推論バックエンドの抽象化（docs/inference_backend.md）。
//!
//! DEIM / PARSeq のコードは `ort` を直接触らず、[`backend()`] が返す
//! [`InferenceBackend`] からモデルを読み、[`ModelSession::run`] で推論する。
//! ネイティブ (`onnx` feature) は `ort`、ブラウザ (`web` feature) は
//! onnxruntime-web への同期ブリッジが実装する。
//!
//! モデルや文字集合の場所は CLI と同じパスで表す。ファイルの有無と読み込みも
//! バックエンドに任せるので、ブラウザではパスを URL に読み替えられる。

use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;
use std::path::Path;

/// 入力テンソルの要素。
#[derive(Clone, Copy, Debug)]
pub enum InputData<'a> {
    F32(&'a [f32]),
    I64(&'a [i64]),
}

/// 名前・形状・要素を持つ入力テンソル。形状の積は要素数と一致する。
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    pub name: &'a str,
    pub shape: &'a [usize],
    pub data: InputData<'a>,
}

impl<'a> Input<'a> {
    pub fn f32(name: &'a str, shape: &'a [usize], data: &'a [f32]) -> Self {
        Self {
            name,
            shape,
            data: InputData::F32(data),
        }
    }

    pub fn i64(name: &'a str, shape: &'a [usize], data: &'a [i64]) -> Self {
        Self {
            name,
            shape,
            data: InputData::I64(data),
        }
    }
}

/// モデルの入力・出力の名前と宣言形状。負の次元は動的、テンソル以外は `None`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Option<Vec<i64>>,
}

/// 1 回の推論の出力。名前で引き、形状と要素を借りる。
pub trait Outputs {
    fn contains(&self, name: &str) -> bool;
    fn f32(&self, name: &str) -> Result<(Vec<i64>, &[f32])>;
    fn i64(&self, name: &str) -> Result<(Vec<i64>, &[i64])>;
}

/// 読み込み済みのモデル。`run` は同時に 1 スレッドからだけ呼ぶ。
pub trait ModelSession: Send {
    fn inputs(&self) -> &[TensorInfo];
    fn outputs(&self) -> &[TensorInfo];
    fn run<'s>(&'s mut self, inputs: &[Input<'_>]) -> Result<Box<dyn Outputs + 's>>;
}

/// 読み込み済みモデルの所有型。
pub type Session = Box<dyn ModelSession>;

/// セッション生成の設定。ネイティブでは ORT の intra-op スレッド数になる。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionOptions {
    pub intra_threads: usize,
}

/// モデルの読み込みとファイルアクセスを提供する。
pub trait InferenceBackend: Send + Sync {
    /// `path` にモデルや文字集合があるか。
    fn is_file(&self, path: &Path) -> bool;
    /// 文字集合 YAML などのテキストを読む。
    fn read_to_string(&self, path: &Path) -> Result<String>;
    /// `path` のモデルを読み込む。
    fn load(&self, path: &Path, options: SessionOptions) -> Result<Session>;
}

static BACKEND: OnceCell<Box<dyn InferenceBackend>> = OnceCell::new();

/// 既定のバックエンドの代わりに使うバックエンドを登録する。最初の推論より前に
/// 1 度だけ呼べる。
pub fn set_backend(backend: Box<dyn InferenceBackend>) -> Result<()> {
    BACKEND
        .set(backend)
        .map_err(|_| anyhow!("inference backend is already set"))
}

/// 登録済みのバックエンド。未登録なら feature の既定（`onnx` では `ort`）を使う。
pub fn backend() -> &'static dyn InferenceBackend {
    BACKEND.get_or_init(default_backend).as_ref()
}

#[cfg(feature = "onnx")]
fn default_backend() -> Box<dyn InferenceBackend> {
    Box::new(super::ort_backend::OrtBackend)
}

#[cfg(not(feature = "onnx"))]
fn default_backend() -> Box<dyn InferenceBackend> {
    Box::new(Unavailable)
}

/// 既定のバックエンドが無いビルド用。読み込みを `set_backend` 待ちとして失敗させる。
#[cfg(not(feature = "onnx"))]
struct Unavailable;

#[cfg(not(feature = "onnx"))]
impl InferenceBackend for Unavailable {
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn load(&self, _path: &Path, _options: SessionOptions) -> Result<Session> {
        anyhow::bail!("no inference backend is set. Call infer::backend::set_backend first.");
    }
}
