use clap::Parser;
use ndlocr_lite_rs::cli::{Cli, Command};
use ndlocr_lite_rs::{PageOptions, recognize_page_gray_u8, recognize_page_rgb_u8};

fn cli_options(extra: &[&str]) -> PageOptions {
    let mut argv = vec!["ndlocr-lite-rs", "recognize-page", "--image", "page.png"];
    argv.extend_from_slice(extra);
    let Command::RecognizePage(args) = Cli::parse_from(argv).command else {
        unreachable!()
    };
    args.page_options(None, None)
}

#[test]
fn default_options_match_recognize_page_cli_defaults() {
    assert_eq!(
        format!("{:?}", PageOptions::default()),
        format!("{:?}", cli_options(&[]))
    );
}

#[test]
fn trace_option_follows_trace_json_argument() {
    assert!(!cli_options(&[]).trace);
    assert!(cli_options(&["--trace-json", "t.json"]).trace);
}

#[test]
fn rejects_buffers_that_do_not_match_dimensions() {
    let options = PageOptions::default();
    let err = recognize_page_rgb_u8(&[0; 11], 2, 2, &options).unwrap_err();
    assert!(err.to_string().contains("does not match 2x2x3"), "{err}");
    let err = recognize_page_gray_u8(&[0; 5], 2, 2, &options).unwrap_err();
    assert!(err.to_string().contains("does not match 2x2"), "{err}");
}

/// Without detection or models the page falls back to bands and fails at recognition,
/// the same way `recognize-page` does in a build without inference.
#[cfg(not(feature = "onnx"))]
#[test]
fn gray_and_rgb_pages_fail_alike_without_inference() {
    let options = PageOptions {
        use_deim_detection: false,
        ..PageOptions::default()
    };
    let gray = vec![255u8; 64 * 32];
    let rgb = vec![255u8; 64 * 32 * 3];
    let a = recognize_page_gray_u8(&gray, 64, 32, &options).map(|p| p.body);
    let b = recognize_page_rgb_u8(&rgb, 64, 32, &options).map(|p| p.body);
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}
