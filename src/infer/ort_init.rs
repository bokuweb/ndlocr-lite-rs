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

/// `Session::with_intra_threads(N)` の N をプール並列度から自動で決める。
///
/// 旧実装は build 時に常に 1 にハードコードしていたため、Intel/AMD の多コア
/// マシンで OCR が深刻に CPU 律速していた (8C/16T で 4 コアぶん遊ぶ等)。
///
/// 戻り値は **1 セッションあたりに使う CPU スレッド数**:
///   intra ≈ available_parallelism / pool_parallelism
///
/// `pool_parallelism` 個の Session を同時に走らせる前提で、合計スレッド数が
/// だいたい論理コア数になる配分を狙う:
///
///  | logical CPUs | pool | intra | total |
///  |--------------|-----:|------:|------:|
///  | 4                         |  2 |  2 |  4 |
///  | 8 (Apple M perf cluster)  |  4 |  2 |  8 |
///  | 16 (Intel i7 8C/16T)      |  4 |  4 | 16 |
///  | 32 (Threadripper)         |  4 |  8 | 32 (intra clamp) |
///
/// 環境変数 `NDLOCR_INTRA_THREADS` で上書きできる (CI / 計測時の固定用)。
/// `pool_parallelism = 0` を渡されたら 1 にフォールバック。intra は [1, 8] に
/// clamp して、超大コアでも 1 session に thrashing 級の thread を持たせない。
pub fn auto_intra_threads(pool_parallelism: usize) -> usize {
    let raw = std::env::var("NDLOCR_INTRA_THREADS").ok();
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    resolve_intra_threads(raw.as_deref(), cpus, pool_parallelism)
}

/// 環境変数とCPU情報の取得から独立して、セッションのスレッド数を決める。
fn resolve_intra_threads(raw: Option<&str>, cpus: usize, pool_parallelism: usize) -> usize {
    if let Some(n) = raw.and_then(|value| value.parse::<usize>().ok()) {
        return n.clamp(1, 32);
    }
    (cpus / pool_parallelism.max(1)).clamp(1, 8)
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

#[cfg(test)]
mod tests {
    use super::resolve_intra_threads;

    /// 明示したスレッド数はプール並列度より優先する。
    #[test]
    fn env_override_takes_precedence() {
        assert_eq!(resolve_intra_threads(Some("3"), 8, 4), 3);
        assert_eq!(resolve_intra_threads(Some("3"), 8, 1), 3);
    }

    /// 明示値の上下限を、プロセス環境を変更せず固定する。
    #[test]
    fn env_override_clamps_to_sane_range() {
        assert_eq!(resolve_intra_threads(Some("0"), 8, 4), 1);
        assert_eq!(resolve_intra_threads(Some("999"), 8, 4), 32);
    }

    /// 不正な明示値ではCPU数とプール数から計算する。
    #[test]
    fn env_override_invalid_falls_back_to_auto() {
        for raw in ["not-a-number", "", "-1"] {
            assert_eq!(resolve_intra_threads(Some(raw), 8, 4), 2);
        }
    }

    /// ゼロのプール数は1として扱う。
    #[test]
    fn pool_zero_is_treated_as_one() {
        assert_eq!(resolve_intra_threads(None, 8, 0), 8);
        assert_eq!(resolve_intra_threads(None, 8, 1), 8);
    }

    /// 多数のCPUでも自動設定の上限を8に保つ。
    #[test]
    fn intra_clamps_to_8_on_huge_machines() {
        assert_eq!(resolve_intra_threads(None, 64, 1), 8);
    }

    /// 巨大なプールやCPU情報の欠落でも1以上を保つ。
    #[test]
    fn intra_at_least_1_for_huge_pool() {
        assert_eq!(resolve_intra_threads(None, 8, usize::MAX), 1);
        assert_eq!(resolve_intra_threads(None, 8, 1024), 1);
        assert_eq!(resolve_intra_threads(None, 0, 1), 1);
    }
}
