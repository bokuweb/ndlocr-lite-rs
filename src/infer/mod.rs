pub mod deim;
pub mod parseq;
mod preprocess;

#[cfg(feature = "onnx")]
pub mod cached;

#[cfg(feature = "onnx")]
pub mod deim_cached;

#[cfg(feature = "onnx")]
pub mod ort_init;

#[cfg(feature = "onnx")]
pub mod page_pool;

/// 横書き余白候補の明示opt-in比較。未知資料での精度改善は未証明。
pub mod margin_candidates;

#[cfg(any(feature = "onnx", test))]
pub(crate) mod short_line_order;
