/// 紙面補正のopt-in APIを、入力画像から同寸法PNGへの変換で試す。
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    anyhow::ensure!(
        args.len() == 3,
        "usage: normalize_paper_shading INPUT OUTPUT.png"
    );
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&args[1]))?;
    let rgb = ndlocr_lite_rs::pipeline::paper_background::normalize_paper_shading(
        &image.data,
        image.width,
        image.height,
    )
    .map_err(anyhow::Error::msg)?;
    image::save_buffer(
        std::path::Path::new(&args[2]),
        &rgb,
        image.width as u32,
        image.height as u32,
        image::ColorType::Rgb8,
    )?;
    Ok(())
}
