# Local Model Layout

`ndlocr-lite-rs` can use locally bundled ONNX models by default.

Place model files under `models/` with these names:

- `models/deim-s-1024x1024.onnx`
- `models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx`
- `models/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx`
- `models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx`

You can fetch the same bundled models used by ndlocr-lite v1.2 releases:

```bash
tools/fetch_models_from_ndlocr_release.sh --tag 1.2.1
```

Charset is loaded by default from:

- `ndlocr/src/config/NDLmoji.yaml`

If you want to use different files, pass explicit arguments:

```bash
cargo run --features onnx -- detect --image /path/to/image.jpg --model /path/to/deim.onnx
cargo run --features onnx -- recognize --image /path/to/image.jpg --model /path/to/parseq.onnx --charset /path/to/NDLmoji.yaml
```
