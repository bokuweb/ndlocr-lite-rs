#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

import torch
import yaml
from strhub.models.utils import load_from_checkpoint


def main() -> int:
    parser = argparse.ArgumentParser(description="Export PARSeq checkpoint to ONNX.")
    parser.add_argument("--checkpoint", required=True, help="Path to PARSeq .ckpt")
    parser.add_argument("--charset-yaml", required=True, help="Path to charset yaml (NDLmoji.yaml)")
    parser.add_argument("--output", required=True, help="Output ONNX path")
    parser.add_argument("--height", type=int, required=True, help="Input image height")
    parser.add_argument("--width", type=int, required=True, help="Input image width")
    parser.add_argument("--opset", type=int, default=17, help="ONNX opset version")
    args = parser.parse_args()

    checkpoint = Path(args.checkpoint)
    charset_yaml = Path(args.charset_yaml)
    output = Path(args.output)

    if not checkpoint.is_file():
        raise FileNotFoundError(f"checkpoint not found: {checkpoint}")
    if not charset_yaml.is_file():
        raise FileNotFoundError(f"charset yaml not found: {charset_yaml}")

    with charset_yaml.open("r", encoding="utf-8") as f:
        config = yaml.safe_load(f)
    charset_test = config["model"]["charset_test"]

    model = load_from_checkpoint(str(checkpoint), charset_test=charset_test).eval().to("cpu")
    dummy = torch.randn([1, 3, args.height, args.width], dtype=torch.float32)

    output.parent.mkdir(parents=True, exist_ok=True)
    model.to_onnx(str(output), dummy, do_constant_folding=True, opset_version=args.opset)
    print(f"exported: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
