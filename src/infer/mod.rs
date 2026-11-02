/// 推論バックエンドの抽象化（docs/inference_backend.md）。
pub mod backend;
pub mod deim;
pub mod parseq;
mod preprocess;
/// タイル分割の行検出の統合（opt-in、`docs/tiled_detection.md`）。
pub mod tiled_detect;

#[cfg(feature = "inference")]
pub mod cached;

#[cfg(feature = "inference")]
pub mod deim_cached;

#[cfg(feature = "inference")]
pub mod ort_init;

#[cfg(feature = "onnx")]
pub mod ort_backend;

#[cfg(feature = "inference")]
pub mod page_pool;

/// 横書き余白候補の明示opt-in比較。未知資料での精度改善は未証明。
pub mod margin_candidates;

#[cfg(any(feature = "inference", test))]
pub(crate) mod short_line_order;

#[cfg(any(feature = "inference", test))]
pub(crate) mod dense_line_recovery;

#[cfg(any(feature = "inference", test))]
pub(crate) mod band_line_recovery;

pub(crate) mod visible_repetitions;

#[cfg(test)]
mod punctuation_context;

#[cfg(any(feature = "inference", test))]
pub(crate) mod missing_line_context;

#[cfg(test)]
#[path = "../../tests/support/ink_height_space.rs"]
mod ink_height_space_tests;

#[cfg(test)]
#[path = "../../tests/support/punctuation_space_components.rs"]
mod punctuation_space_component_tests;

#[cfg(test)]
#[path = "../../tests/support/punctuation_space_width.rs"]
mod punctuation_space_width_tests;

#[cfg(test)]
#[path = "../../tests/support/punctuation_space_advance.rs"]
mod punctuation_space_advance_tests;

#[cfg(any(feature = "inference", test))]
mod vertical_capacity;

#[cfg(any(feature = "inference", test))]
mod output_capacity;

#[cfg(any(feature = "inference", test))]
pub(crate) mod vertical_fallback;

#[cfg(feature = "inference")]
pub(crate) mod execution;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web_backend;
