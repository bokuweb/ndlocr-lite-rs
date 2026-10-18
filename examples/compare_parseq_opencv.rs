//! 本流と同じ uint8 OpenCV 補間を独立プロセスで比較する診断。
#[path = "support/opencv_oracle.rs"]
mod opencv_oracle;
#[path = "support/resampling_comparison.rs"]
mod resampling_comparison;

/// 固定された Python oracle を起動して同一 RGB を比較する。
fn main() -> anyhow::Result<()> {
    let mut oracle = opencv_oracle::Oracle::start()?;
    resampling_comparison::run_with("opencv", |rgb, w, h, iw, ih| {
        oracle.preprocess(rgb, w, h, iw, ih)
    })
}
