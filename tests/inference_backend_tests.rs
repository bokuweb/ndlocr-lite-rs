#![cfg(feature = "inference")]
use anyhow::{Result, bail};
use ndlocr_lite_rs::infer::backend::*;
use ndlocr_lite_rs::infer::{cached::ParseqSession, deim_cached::DeimSession};
use std::path::Path;

static LOADS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
struct VirtualBackend;
struct VirtualSession {
    detector: bool,
    inputs: Vec<TensorInfo>,
    outputs: Vec<TensorInfo>,
}
struct Values;
impl Outputs for Values {
    fn contains(&self, name: &str) -> bool {
        ["labels", "boxes", "scores", "char_count", "logits"].contains(&name)
    }
    fn f32(&self, name: &str) -> Result<(Vec<i64>, &[f32])> {
        match name {
            "boxes" => Ok((vec![1, 1, 4], &[0., 0., 8., 8.])),
            "scores" => Ok((vec![1, 1], &[0.95])),
            "logits" => Ok((vec![1, 2, 3], &[0., 10., 0., 10., 0., 0.])),
            _ => bail!("wrong type or output: {name}"),
        }
    }
    fn i64(&self, name: &str) -> Result<(Vec<i64>, &[i64])> {
        match name {
            "labels" => Ok((vec![1, 1], &[1])),
            "char_count" => Ok((vec![1, 1], &[3])),
            _ => bail!("wrong type or output: {name}"),
        }
    }
}
impl ModelSession for VirtualSession {
    fn inputs(&self) -> &[TensorInfo] {
        &self.inputs
    }
    fn outputs(&self) -> &[TensorInfo] {
        &self.outputs
    }
    fn run<'s>(&'s mut self, inputs: &[Input<'_>]) -> Result<Box<dyn Outputs + 's>> {
        let InputData::F32(pixels) = inputs[0].data else {
            panic!("image must be f32")
        };
        assert_eq!(pixels.len(), inputs[0].shape.iter().product::<usize>());
        if self.detector {
            assert_eq!(inputs[0].name, "images");
            assert_eq!(inputs[1].name, "orig_target_sizes");
            assert!(matches!(inputs[1].data, InputData::I64([8, 8])));
        } else {
            assert_eq!(inputs[0].name, "image");
        }
        Ok(Box::new(Values))
    }
}
impl InferenceBackend for VirtualBackend {
    fn is_file(&self, path: &Path) -> bool {
        !path.ends_with("missing.onnx")
    }
    fn read_to_string(&self, _: &Path) -> Result<String> {
        Ok("model:\n  charset_train: 'AB'\n".into())
    }
    fn load(&self, path: &Path, options: SessionOptions) -> Result<Session> {
        assert!(options.intra_threads > 0);
        LOADS
            .lock()
            .unwrap()
            .push(path.to_string_lossy().into_owned());
        let detector = path.ends_with("deim.onnx");
        Ok(Box::new(VirtualSession {
            detector,
            inputs: vec![TensorInfo {
                name: if detector { "images" } else { "image" }.into(),
                shape: Some(vec![-1, 3, 8, 8]),
            }],
            outputs: vec![TensorInfo {
                name: "logits".into(),
                shape: Some(vec![-1, 2, 3]),
            }],
        }))
    }
}
#[test]
fn cached_models_use_virtual_files_and_typed_backend_tensors() {
    set_backend(Box::new(VirtualBackend)).unwrap();
    assert!(set_backend(Box::new(VirtualBackend)).is_err());
    let recognizer = ParseqSession::load(
        Path::new("virtual/parseq.onnx"),
        Path::new("virtual/charset.yaml"),
    )
    .unwrap();
    assert_eq!(recognizer.input_dimensions(), (8, 8));
    assert!(recognizer.supports_batch());
    let recognized = recognizer
        .recognize_rgb_u8_unprocessed(&[255; 8 * 8 * 3], 8, 8)
        .unwrap();
    assert_eq!(recognized.text, "A");
    let detector = DeimSession::load(Path::new("virtual/deim.onnx")).unwrap();
    let lines = detector
        .detect_rgb_u8(&[255; 8 * 8 * 3], 8, 8, 0.25)
        .unwrap();
    assert_eq!(lines.len(), 1);
    assert!(
        ParseqSession::load(Path::new("missing.onnx"), Path::new("charset.yaml"))
            .err()
            .unwrap()
            .to_string()
            .contains("parseq model not found")
    );
    let options = ndlocr_lite_rs::PageOptions {
        model: "virtual/100.onnx".into(),
        model30: "virtual/30.onnx".into(),
        model50: "virtual/50.onnx".into(),
        charset: "virtual/charset.yaml".into(),
        use_deim_detection: false,
        ..Default::default()
    };
    ndlocr_lite_rs::recognize_page_rgb_u8(&[255; 8 * 8 * 3], 8, 8, &options).unwrap();
    let paths = LOADS.lock().unwrap().clone();
    assert!(
        paths.contains(&"virtual/30.onnx".into()),
        "virtual model30 must load"
    );
    assert!(
        paths.contains(&"virtual/50.onnx".into()),
        "virtual model50 must load"
    );
    assert_eq!(
        ndlocr_lite_rs::infer::parseq::recognize_rgb_u8_with_score_unprocessed(
            Path::new("virtual/parseq.onnx"),
            &[255; 8 * 8 * 3],
            8,
            8,
            Path::new("virtual/charset.yaml")
        )
        .unwrap()
        .text,
        "A"
    );
}
