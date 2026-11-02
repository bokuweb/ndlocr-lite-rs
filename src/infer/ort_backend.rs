//! `ort` (ONNX Runtime) による [`InferenceBackend`] 実装。
#![cfg(feature = "onnx")]

use anyhow::{Result, anyhow};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::{SessionInputValue, SessionOutputs};
use ort::value::TensorRef;
use std::path::Path;

use super::backend::{
    InferenceBackend, Input, InputData, ModelSession, Outputs, Session, SessionOptions, TensorInfo,
};
use super::ort_init::{OrtAnyhow, ensure_init};

/// ファイルシステム上のモデルを `ort` で読む既定のネイティブバックエンド。
pub struct OrtBackend;

impl InferenceBackend for OrtBackend {
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn load(&self, path: &Path, options: SessionOptions) -> Result<Session> {
        ensure_init();
        let session = ort::session::Session::builder()
            .anyort()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .anyort()?
            .with_intra_threads(options.intra_threads.max(1))
            .anyort()?
            .commit_from_file(path)
            .anyort()?;
        Ok(Box::new(OrtSession::new(session)))
    }
}

struct OrtSession {
    session: ort::session::Session,
    inputs: Vec<TensorInfo>,
    outputs: Vec<TensorInfo>,
}

fn tensor_infos(outlets: &[ort::value::Outlet]) -> Vec<TensorInfo> {
    outlets
        .iter()
        .map(|outlet| TensorInfo {
            name: outlet.name().to_string(),
            shape: outlet.dtype().tensor_shape().map(|shape| {
                let dims: &[i64] = shape;
                dims.to_vec()
            }),
        })
        .collect()
}

impl OrtSession {
    fn new(session: ort::session::Session) -> Self {
        Self {
            inputs: tensor_infos(session.inputs()),
            outputs: tensor_infos(session.outputs()),
            session,
        }
    }
}

impl ModelSession for OrtSession {
    fn inputs(&self) -> &[TensorInfo] {
        &self.inputs
    }

    fn outputs(&self) -> &[TensorInfo] {
        &self.outputs
    }

    fn run<'s>(&'s mut self, inputs: &[Input<'_>]) -> Result<Box<dyn Outputs + 's>> {
        let mut values: Vec<(&str, SessionInputValue<'_>)> = Vec::with_capacity(inputs.len());
        for input in inputs {
            let value: SessionInputValue<'_> = match input.data {
                InputData::F32(data) => TensorRef::from_array_view((input.shape, data))
                    .anyort()?
                    .into(),
                InputData::I64(data) => TensorRef::from_array_view((input.shape, data))
                    .anyort()?
                    .into(),
            };
            values.push((input.name, value));
        }
        let outputs = self.session.run(values).anyort()?;
        Ok(Box::new(OrtOutputs(outputs)))
    }
}

struct OrtOutputs<'s>(SessionOutputs<'s>);

impl OrtOutputs<'_> {
    fn value(&self, name: &str) -> Result<&ort::value::DynValue> {
        self.0
            .get(name)
            .ok_or_else(|| anyhow!("model has no output named {name}"))
    }
}

impl Outputs for OrtOutputs<'_> {
    fn contains(&self, name: &str) -> bool {
        self.0.get(name).is_some()
    }

    fn f32(&self, name: &str) -> Result<(Vec<i64>, &[f32])> {
        let (shape, data) = self.value(name)?.try_extract_tensor::<f32>().anyort()?;
        Ok((shape.to_vec(), data))
    }

    fn i64(&self, name: &str) -> Result<(Vec<i64>, &[i64])> {
        let (shape, data) = self.value(name)?.try_extract_tensor::<i64>().anyort()?;
        Ok((shape.to_vec(), data))
    }
}
