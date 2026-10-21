# Additional real-six Page primary trace

`trace-v1.index.json` authenticates `trace-v1.zip` (3,664,852 bytes), 412 logical files in 120 content-addressed objects.
All original runtime JSON/logs, before-NN bindings, observer source/build proof, 174 captured RGB members and official metrics are retained.
The input/GT and untraced FVB baseline are authenticated from the adjacent `real-vertical-additional-controls` packets.

Output ZIP containers are normalized to exact member bytes to deduplicate repeated crops. Their original SHA/size and member mapping are retained;
the packet does not claim byte reconstruction of the original containers. Original output ZIP files remain in the private experiment directory.
Model and executable bytes are not bundled; their hashes and fresh compilation/command receipts are retained.

From the repository root with the pinned Pillow dependency:

```sh
python tools/verify_additional_real_primary_trace.py
```

No recognition runs in this replay. It checks all 36 FVB body/bbox/f32-confidence alignments, 174 RGBs, 396 official metric rows and 66 successful PARSeq forwards.
The before-NN protocol and result interpretation are in `docs/real_vertical_additional_primary_trace.md` and `docs/real_vertical_additional_primary_trace_results.md`.
