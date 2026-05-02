//! `ort` v2 のグローバル初期化を 1 度だけ実行するためのヘルパ。
//!
//! `ort::init()` は execution provider をプロセスに登録する関数で、最初の
//! [`ort::session::Session`] 構築より前に 1 度だけ呼ぶ。
//!
//! `coreml` feature を有効化すると CoreML EP を登録するが、
//! 2026-05 時点で parseq / DEIM の組み合わせは CoreML 経由で動かない:
//!   - parseq: 動的バッチで MLProgram を作ると MatMul の shape mismatch で
//!     runtime エラー (decoder の swish_ffn/w3/MatMul)。
//!   - DEIM: MLProgram の execution plan ビルドに失敗 (`error code: -7`)。
//!   - cold compile が 1 セッション 数分 〜 10 分掛かるので静的シェイプで
//!     動かしても実用にならない。
//! Apple ANE/GPU を使うには `with_static_input_shapes(true)` 化と batch=1 に
//! 固定する変更がモデル側にも必要。CPU EP は ort 2.0 に同梱されている既定
//! 実装で、こちらは旧 `onnxruntime` 0.0.14 比 ~30-40% 速い。

#![cfg(feature = "onnx")]

use once_cell::sync::OnceCell;

static INITIALIZED: OnceCell<()> = OnceCell::new();

/// `ort::Error` は内部に `NonNull` を持っており `Send + Sync` ではない。
/// `anyhow::Result` への `?` 変換は `Send + Sync` を要求するので素直には通らない。
/// このトレイトは ort 系結果型を `Display` 経由で `anyhow::Error` に潰す。
/// 使い方: `Session::builder().anyort()?`
pub trait OrtAnyhow<T> {
    fn anyort(self) -> anyhow::Result<T>;
}
impl<T, E: std::fmt::Display> OrtAnyhow<T> for std::result::Result<T, E> {
    #[inline]
    fn anyort(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!("ort: {e}"))
    }
}

/// 最初の [`ort::session::Session`] を作る前に呼ぶ。多重呼び出しは no-op。
pub fn ensure_init() {
    INITIALIZED.get_or_init(|| {
        #[allow(unused_mut)]
        let builder = ort::init();
        #[cfg(feature = "coreml")]
        let builder = {
            // CoreML MLProgram は初回ロード時に ONNX graph を `.mlmodelc` に
            // コンパイルする。parseq tiny でも 1 モデルあたり数分掛かるので、
            // ディスクキャッシュ必須。`NDLOCR_COREML_CACHE_DIR` で上書き可能。
            let cache_dir = std::env::var("NDLOCR_COREML_CACHE_DIR").unwrap_or_else(|_| {
                let mut p = std::env::temp_dir();
                p.push("ndlocr-coreml-cache");
                p.to_string_lossy().into_owned()
            });
            // None の場合は空文字を渡しても無視される実装になっている。
            let _ = std::fs::create_dir_all(&cache_dir);
            builder.with_execution_providers([ort::ep::CoreML::default()
                .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
                .with_compute_units(ort::ep::coreml::ComputeUnits::All)
                .with_model_cache_dir(cache_dir)
                .build()])
        };
        let _ = builder.commit();
    });
}
