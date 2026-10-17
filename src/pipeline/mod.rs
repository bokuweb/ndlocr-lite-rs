/// 色付き横長パネルの背景と字形を分離する。
pub mod background;
mod band_bounds;
pub mod cascade;
mod component_bands;
pub mod connect;
pub mod crop;
mod fragment_bands;
/// 横書き行を画素欠落のない短い認識区間に分割する。
pub mod line_chunks;
pub mod line_segment;
/// 明るい紙面の緩い陰影を検出と認識に共通の画素へ準備する。
pub mod paper_background;
mod punctuation_bands;
pub mod reading_order;
pub mod run_page;
