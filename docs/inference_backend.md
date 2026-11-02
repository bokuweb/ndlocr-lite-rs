# Inference backends

DEIM and PARSeq use `infer::backend::{InferenceBackend, ModelSession, Outputs}`.
The backend owns filesystem access and model sessions; callers keep the existing
preprocessing, decoding, detection, ordering and postprocessing.

`onnx` enables `inference` and the native `OrtBackend`. ORT stays at rc.13, with
Level3 graph optimization and the existing per-call intra-thread settings.
Inputs borrow f32 or i64 buffers; outputs borrow native ORT tensors without copying.
Metadata preserves model names and dynamic shapes. Both single-call and cached
sessions use this interface. Missing-model diagnostics remain unchanged.

An application may call `set_backend` once before inference. An `inference`-only
build has no default engine; `load` reports that a backend must be registered.
Without `inference`, the existing feature-disabled CLI behavior remains unchanged.

Acceptance: no-feature build/tests, inference-only mock model tests, native ONNX
regressions and the preserved 34-run/165-output CLI comparison must pass.
The mock backend must exercise virtual charset/model loading, f32 image inputs,
i64 DEIM sizes/labels/counts, dynamic metadata and borrowed outputs through the
actual cached sessions. This is the interface the browser backend will implement.
