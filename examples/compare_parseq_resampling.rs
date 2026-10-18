//! 横書きの同一RGBに対する縮小方式の比較診断。
#[path = "support/area_resampling.rs"]
mod area_resampling;
#[path = "support/resampling_comparison.rs"]
mod resampling_comparison;

/// 正解を読み込まず、固定された行の全chunkを比較する。
fn main() -> anyhow::Result<()> {
    resampling_comparison::run()
}
