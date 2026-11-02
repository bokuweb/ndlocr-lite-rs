//! Synchronous worker bridge to onnxruntime-web. No JS objects cross Rust threads.
use super::backend::*;
use anyhow::{Result, anyhow, bail};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __ndlocrRequest)]
    fn request_js(request: &str, data: &[u8]) -> std::result::Result<js_sys::Uint8Array, JsValue>;
}

pub struct WebBackend;
#[derive(Deserialize)]
struct Info {
    name: String,
    shape: Option<Vec<i64>>,
}
fn infos(value: Value) -> Result<Vec<TensorInfo>> {
    Ok(serde_json::from_value::<Vec<Info>>(value)?
        .into_iter()
        .map(|i| TensorInfo {
            name: i.name,
            shape: i.shape,
        })
        .collect())
}
fn request(meta: Value, data: &[u8]) -> Result<(Value, Vec<u8>)> {
    let bytes = request_js(&meta.to_string(), data)
        .map_err(|error| {
            anyhow!(
                "browser inference: {}",
                error.as_string().unwrap_or_else(|| format!("{error:?}"))
            )
        })?
        .to_vec();
    let prefix: [u8; 4] = bytes
        .get(..4)
        .ok_or_else(|| anyhow!("truncated bridge header"))?
        .try_into()?;
    let end = 4usize
        .checked_add(u32::from_le_bytes(prefix) as usize)
        .ok_or_else(|| anyhow!("bridge header overflow"))?;
    let meta: Value = serde_json::from_slice(
        bytes
            .get(4..end)
            .ok_or_else(|| anyhow!("truncated bridge metadata"))?,
    )?;
    if let Some(error) = meta.get("error").and_then(Value::as_str) {
        bail!("{error}");
    }
    Ok((meta, bytes[end..].to_vec()))
}
impl InferenceBackend for WebBackend {
    fn is_file(&self, _path: &Path) -> bool {
        true
    }
    fn read_to_string(&self, path: &Path) -> Result<String> {
        let (meta, _) = request(json!({"op":"read", "path":path}), &[])?;
        Ok(meta["text"]
            .as_str()
            .ok_or_else(|| anyhow!("missing charset text"))?
            .into())
    }
    fn load(&self, path: &Path, _options: SessionOptions) -> Result<Session> {
        let (meta, _) = request(json!({"op":"load", "path":path}), &[])?;
        Ok(Box::new(WebSession {
            path: path.to_string_lossy().into_owned(),
            inputs: infos(meta["inputs"].clone())?,
            outputs: infos(meta["outputs"].clone())?,
        }))
    }
}
struct WebSession {
    path: String,
    inputs: Vec<TensorInfo>,
    outputs: Vec<TensorInfo>,
}
enum Data {
    F32(Vec<f32>),
    I64(Vec<i64>),
}
struct WebOutputs(HashMap<String, (Vec<i64>, Data)>);
impl ModelSession for WebSession {
    fn inputs(&self) -> &[TensorInfo] {
        &self.inputs
    }
    fn outputs(&self) -> &[TensorInfo] {
        &self.outputs
    }
    fn run<'s>(&'s mut self, inputs: &[Input<'_>]) -> Result<Box<dyn Outputs + 's>> {
        let mut data = Vec::new();
        let mut descriptors = Vec::new();
        for input in inputs {
            let start = data.len();
            let kind = match input.data {
                InputData::F32(values) => {
                    for value in values {
                        data.extend(value.to_le_bytes());
                    }
                    "float32"
                }
                InputData::I64(values) => {
                    for value in values {
                        data.extend(value.to_le_bytes());
                    }
                    "int64"
                }
            };
            descriptors.push(json!({"name":input.name,"dims":input.shape,"type":kind,"offset":start,"length":data.len()-start}));
        }
        let (meta, bytes) = request(
            json!({"op":"run","path":self.path,"inputs":descriptors}),
            &data,
        )?;
        let mut outputs = HashMap::new();
        for descriptor in meta["tensors"]
            .as_array()
            .ok_or_else(|| anyhow!("missing bridge tensors"))?
        {
            let name = descriptor["name"]
                .as_str()
                .ok_or_else(|| anyhow!("missing tensor name"))?
                .to_owned();
            let shape: Vec<i64> = serde_json::from_value(descriptor["dims"].clone())?;
            let offset = descriptor["offset"]
                .as_u64()
                .ok_or_else(|| anyhow!("invalid tensor offset"))? as usize;
            let length = descriptor["length"]
                .as_u64()
                .ok_or_else(|| anyhow!("invalid tensor length"))? as usize;
            let end = offset
                .checked_add(length)
                .ok_or_else(|| anyhow!("tensor size overflow"))?;
            let slice = bytes
                .get(offset..end)
                .ok_or_else(|| anyhow!("truncated bridge tensor"))?;
            let data = match descriptor["type"].as_str() {
                Some("float32") if length % 4 == 0 => Data::F32(
                    slice
                        .chunks_exact(4)
                        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                        .collect(),
                ),
                Some("int64") if length % 8 == 0 => Data::I64(
                    slice
                        .chunks_exact(8)
                        .map(|c| i64::from_le_bytes(c.try_into().unwrap()))
                        .collect(),
                ),
                _ => bail!("unsupported browser output tensor type"),
            };
            outputs.insert(name, (shape, data));
        }
        Ok(Box::new(WebOutputs(outputs)))
    }
}
impl Outputs for WebOutputs {
    fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
    fn f32(&self, name: &str) -> Result<(Vec<i64>, &[f32])> {
        match self.0.get(name) {
            Some((shape, Data::F32(data))) => Ok((shape.clone(), data)),
            _ => bail!("missing f32 output {name}"),
        }
    }
    fn i64(&self, name: &str) -> Result<(Vec<i64>, &[i64])> {
        match self.0.get(name) {
            Some((shape, Data::I64(data))) => Ok((shape.clone(), data)),
            _ => bail!("missing i64 output {name}"),
        }
    }
}
