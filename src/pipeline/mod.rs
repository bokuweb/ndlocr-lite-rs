/// 色付き横長パネルの背景と字形を分離する。
pub mod background;
mod band_bounds;
pub mod cascade;
pub mod connect;
pub mod crop;
/// 横書き行を画素欠落のない短い認識区間に分割する。
pub mod line_chunks;
pub mod line_segment;
pub mod reading_order;
pub mod run_page;
