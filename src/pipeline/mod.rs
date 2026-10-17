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
/// 行クラスの競合検出を、認識前に共通の条件で選ぶ。
pub(crate) mod line_detections;
/// 原領域を保持し、予算内の隣接帯結合を採点候補として列挙する。
pub mod line_hypotheses;
pub mod line_segment;
mod noisy_line_gaps;
/// 明るい紙面の緩い陰影を検出と認識に共通の画素へ準備する。
pub mod paper_background;
mod punctuation_bands;
pub mod reading_order;
pub mod run_page;
/// 既知の縦書きページを元座標の右から左の列へ分ける。
pub mod vertical_segment;
