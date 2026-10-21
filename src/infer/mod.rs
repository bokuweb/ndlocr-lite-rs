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

#[cfg(any(feature = "onnx", test))]
pub(crate) mod dense_line_recovery;

#[cfg(any(feature = "onnx", test))]
pub(crate) mod band_line_recovery;

pub(crate) mod visible_repetitions;

#[cfg(test)]
mod punctuation_context;

#[cfg(any(feature = "onnx", test))]
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

#[cfg(any(feature = "onnx", test))]
mod vertical_capacity;

#[cfg(any(feature = "onnx", test))]
mod output_capacity;

#[cfg(any(feature = "onnx", test))]
mod vertical_fallback;
