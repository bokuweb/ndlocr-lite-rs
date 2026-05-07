use clap::Parser;
use ndlocr_lite_rs::app::run_cli;
use ndlocr_lite_rs::cli::Cli;
use tempfile::tempdir;

#[test]
fn mock_page_command_writes_json_xml_and_txt() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 0, 255, 0]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
    ]);
    run_cli(cli).unwrap();

    let json = outdir.join("input.json");
    let xml = outdir.join("input.xml");
    let txt = outdir.join("input.txt");
    assert!(json.is_file());
    assert!(xml.is_file());
    assert!(txt.is_file());
}

#[test]
fn mock_page_command_uses_output_stem_when_specified() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 0, 255, 0]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--output-stem",
        "custom_name",
    ]);
    run_cli(cli).unwrap();

    assert!(outdir.join("custom_name.json").is_file());
    assert!(outdir.join("custom_name.xml").is_file());
    assert!(outdir.join("custom_name.txt").is_file());
}

#[test]
fn mock_page_command_uses_line_text_when_specified() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 0, 255, 0]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--line-text",
        "hello_custom_text",
    ]);
    run_cli(cli).unwrap();

    let txt_body = std::fs::read_to_string(outdir.join("input.txt")).unwrap();
    assert_eq!(txt_body, "hello_custom_text");
}

#[test]
fn mock_page_command_uses_line_count_for_multiple_outputs() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 6, vec![255; 2 * 6 * 3]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--line-count",
        "3",
        "--line-text",
        "multi",
    ]);
    run_cli(cli).unwrap();

    let txt_body = std::fs::read_to_string(outdir.join("input.txt")).unwrap();
    let lines: Vec<&str> = txt_body.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines, vec!["multi", "multi", "multi"]);

    let json_body = std::fs::read_to_string(outdir.join("input.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_body).unwrap();
    assert_eq!(v["contents"][0].as_array().unwrap().len(), 3);

    let xml_body = std::fs::read_to_string(outdir.join("input.xml")).unwrap();
    let ys: Vec<i32> = xml_body
        .lines()
        .filter(|line| line.contains("<LINE "))
        .filter_map(|line| {
            let (_, rest) = line.split_once(" Y=\"")?;
            let (y, _) = rest.split_once('"')?;
            y.parse::<i32>().ok()
        })
        .collect();
    assert_eq!(ys.len(), 3);
    assert!(ys.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn mock_page_command_uses_line_confidence_when_specified() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 2, vec![255; 2 * 2 * 3]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--line-confidence",
        "0.42",
    ]);
    run_cli(cli).unwrap();

    let json_body = std::fs::read_to_string(outdir.join("input.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_body).unwrap();
    let conf = v["contents"][0][0]["confidence"].as_f64().unwrap();
    assert!((conf - 0.42).abs() < 1e-6);
}

#[test]
fn mock_page_command_uses_line_orientation_when_specified() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(4, 2, vec![255; 4 * 2 * 3]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--line-orientation",
        "horizontal",
    ]);
    run_cli(cli).unwrap();

    let json_body = std::fs::read_to_string(outdir.join("input.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_body).unwrap();
    assert_eq!(v["contents"][0][0]["isVertical"].as_str().unwrap(), "false");
}

#[test]
fn mock_page_command_rejects_empty_output_stem() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let outdir = dir.path().join("out");
    std::fs::create_dir_all(&outdir).unwrap();
    let img = image::RgbImage::from_raw(2, 2, vec![255; 2 * 2 * 3]).unwrap();
    img.save(&image).unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "mock-page",
        "--image",
        &image.to_string_lossy(),
        "--output-dir",
        &outdir.to_string_lossy(),
        "--output-stem",
        "",
    ]);
    let err = run_cli(cli).expect_err("must reject empty output stem");
    assert!(err.to_string().contains("output_stem"));
}

#[cfg(not(feature = "onnx"))]
#[test]
fn recognize_command_reports_feature_disabled_when_onnx_is_off() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let model = dir.path().join("model.onnx");
    let charset = dir.path().join("charset.yaml");
    let img = image::RgbImage::from_raw(1, 1, vec![255, 255, 255]).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&model, b"dummy").unwrap();
    std::fs::write(&charset, "model:\n  charset_train: \"abc\"\n").unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize",
        "--image",
        &image.to_string_lossy(),
        "--model",
        &model.to_string_lossy(),
        "--charset",
        &charset.to_string_lossy(),
    ]);
    let err = run_cli(cli).expect_err("onnx disabled should fail");
    assert!(err.to_string().contains("onnx feature is disabled"));
}

#[cfg(not(feature = "onnx"))]
#[test]
fn recognize_page_command_reports_feature_disabled_when_onnx_is_off() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let model = dir.path().join("model.onnx");
    let charset = dir.path().join("charset.yaml");
    let mut data = vec![255u8; 200 * 100 * 3];
    for y in 30..45 {
        for x in 20..180 {
            let i = (y * 200 + x) * 3;
            data[i] = 0;
            data[i + 1] = 0;
            data[i + 2] = 0;
        }
    }
    let img = image::RgbImage::from_raw(200, 100, data).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&model, b"dummy").unwrap();
    std::fs::write(&charset, "model:\n  charset_train: \"abc\"\n").unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        &image.to_string_lossy(),
        "--model",
        &model.to_string_lossy(),
        "--charset",
        &charset.to_string_lossy(),
        "--output-docx",
        &dir.path().join("out.docx").to_string_lossy(),
    ]);
    let err = run_cli(cli).expect_err("onnx disabled should fail");
    assert!(err.to_string().contains("onnx feature is disabled"));
}

#[test]
fn recognize_page_command_rejects_invalid_post_dict_yaml() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let out_dict = dir.path().join("bad.yaml");
    let img = image::RgbImage::from_raw(10, 10, vec![255; 10 * 10 * 3]).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&out_dict, "replacements: [\n").unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        &image.to_string_lossy(),
        "--post-dict",
        &out_dict.to_string_lossy(),
    ]);
    let err = run_cli(cli).expect_err("invalid dict yaml should fail");
    assert!(err.to_string().contains("invalid postprocess dict yaml"));
}

#[test]
fn recognize_page_command_rejects_invalid_rule_pack_yaml() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let rule_pack = dir.path().join("bad-rule-pack.yaml");
    let img = image::RgbImage::from_raw(10, 10, vec![255; 10 * 10 * 3]).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&rule_pack, "merge_rules: [\n").unwrap();

    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        &image.to_string_lossy(),
        "--rule-pack",
        &rule_pack.to_string_lossy(),
    ]);
    let err = run_cli(cli).expect_err("invalid rule-pack yaml should fail");
    assert!(err.to_string().contains("invalid rule pack yaml"));
}

#[cfg(not(feature = "morph-correct"))]
#[test]
fn recognize_page_command_reports_morph_correct_feature_disabled() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "input.png",
        "--morph-correct-dict",
        "dict/system.dic.zst",
    ]);

    let err = run_cli(cli).expect_err("morph-correct disabled should fail");
    assert!(
        err.to_string()
            .contains("--morph-correct-dict requires building with --features morph-correct")
    );
}
