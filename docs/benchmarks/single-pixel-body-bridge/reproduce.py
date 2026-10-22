#!/usr/bin/env python3
"""Verify the evidence and reproduce every official score table without ONNX."""
from pathlib import Path
import argparse
import csv
import hashlib
import json
import subprocess
import struct
import sys
import zipfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def emitted_mean_bits(row):
    f32 = lambda x: struct.unpack("<f", struct.pack("<f", x))[0]
    assert len(row["chunks"]) == 1
    total = f32(0)
    count = 0
    cursor = 0
    encoded = row["text"].encode()
    for step in row["chunks"][0]["steps"]:
        if step["byte_range"] is None:
            continue
        start, end = step["byte_range"]
        token = step["candidates"][0]
        assert start == cursor and token["character"].encode() == encoded[start:end]
        cursor = end
        total = f32(total + f32(token["probability"]))
        count += 1
    assert cursor == len(encoded)
    mean = f32(total / f32(count)) if count else f32(0)
    return struct.unpack("<I", struct.pack("<f", mean))[0]


def restore(packet, output):
    hashes = {}
    for name in ["data", "diagnostics", "text"]:
        index = json.loads((packet / (name + ".json")).read_text())
        archive = packet / (name + ".zip")
        assert digest(archive.read_bytes()) == index["sha256"]
        hashes[name] = index["sha256"]
        with zipfile.ZipFile(archive) as z:
            assert set(z.namelist()) == set(index["members"])
            for key, expected in index["members"].items():
                p = Path(key)
                assert not p.is_absolute() and ".." not in p.parts
                data = z.read(key)
                assert digest(data) == expected, key
                target = output / p
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
        aliases = json.loads((output / (name + "-aliases.json")).read_text())
        for key, alias in aliases.items():
            for p in [Path(key), Path(alias["source"])]:
                assert not p.is_absolute() and ".." not in p.parts
            data = (output / alias["source"]).read_bytes()
            assert digest(data) == alias["sha256"]
            target = output / key
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    return hashes


def score(scorer, truth, before, after, norm, target, expected, labels):
    target.parent.mkdir(parents=True, exist_ok=True)
    report = target.with_suffix(".md")
    subprocess.run(
        [sys.executable, str(scorer), "--truth-dir", str(truth),
         "--system", labels[0] + "=" + str(before),
         "--system", labels[1] + "=" + str(after), "--normalize", norm,
         "--require-all", "--output-csv", str(target), "--output-md", str(report)],
        check=True, capture_output=True,
    )
    target.write_bytes(target.read_bytes().replace(b"\r\n", b"\n"))
    report.write_bytes((report.read_text().rstrip() + "\n").encode())
    assert target.read_bytes() == expected.read_bytes(), expected
    assert report.read_bytes() == expected.with_suffix(".md").read_bytes(), expected
    rows = list(csv.DictReader(target.open()))
    b = {x["file"]: x for x in rows if x["system"] == labels[0]}
    a = {x["file"]: x for x in rows if x["system"] == labels[1]}
    assert set(a) == set(b)
    for key in b:
        assert int(a[key]["char_errors"]) <= int(b[key]["char_errors"]), key
        assert int(a[key]["word_errors"]) <= int(b[key]["word_errors"]), key


def verify_pixels(root):
    from PIL import Image, ImageChops
    verified = 0
    for folder, count in [("fresh", 3648), ("horizontal", 128)]:
        data = root / "data" / folder
        manifest = json.loads((data / "evaluation-manifest.json").read_text())
        rows = manifest["inputs"]
        assert len(rows) == count
        resolve = lambda p: root / "data" / Path(p).relative_to("tmp/closer-bridge")
        atlas = Image.open(data / "atlas.png").convert("RGB")
        assert digest((data / "atlas.png").read_bytes()) == manifest["atlas_sha256"]
        for x in rows:
            for key in ["rgb", "effective_rgb", "truth", "image", "original"]:
                assert digest(resolve(x[key]).read_bytes()) == x[key + "_sha256"], x["id"]
            rgb = resolve(x["rgb"]).read_bytes()
            original = Image.open(resolve(x["image"])).convert("RGB")
            assert original.tobytes() == rgb
            effective = resolve(x["effective_rgb"]).read_bytes()
            assert atlas.crop(x["bounds"]).tobytes() == effective
            if folder == "fresh":
                rotated = 5 * x["height"] > 4 * x["width"]
                assert rotated == x["rotated_for_auto"]
                transformed = original.transpose(Image.Transpose.ROTATE_90) if rotated else original
                assert transformed.tobytes() == effective
                if x["kind"] == "noise" and x["format"] == "png":
                    stem = f'b_t{x["body_id"]}_{x["font"]}_{x["size"]}_p{x["palt"]}_bare'
                    full = Image.open(data / "native" / (stem + ".png")).convert("RGB")
                    box = ImageChops.difference(full, Image.new("RGB", full.size, "white")).getbbox()
                    body = full.crop((box[0] - 2, box[1] - 2, box[2] + 2, box[3] + 2))
                    sx, sy, ex, ey = x["shape_bounds"]
                    mw = ex - sx
                    shift = mw + x["gap"]
                    offset = shift if x["leading"] else 0
                    assert not any(body.getpixel((xx, yy)) != (255, 255, 255)
                                   and sx <= xx + offset < ex and sy <= yy < ey
                                   for yy in range(body.height) for xx in range(body.width))
                    expected = Image.new("RGB", (body.width + shift, body.height), "white")
                    expected.paste(body, (offset, 0))
                    if mw:
                        expected.paste((0, 0, 0), (sx, sy, ex, ey))
                    assert expected.tobytes() == original.tobytes()
        capture = json.loads((data / "whole-rgb-capture/inputs.json").read_text())["chunks"]
        assert len(capture) == count
        by_id = {x["id"]: x for x in rows}
        assert {c["line_id"] for c in capture} == set(by_id)
        for c in capture:
            x = by_id[c["line_id"]]
            im = Image.open(data / "whole-rgb-capture" / c["image"]).convert("RGB")
            assert list(im.size) == c["dimensions"] == [x["forward_width"], x["forward_height"]]
            assert c["chunk_index"] == 0 and c["x_range"] == [0, x["forward_width"]]
            assert digest(im.tobytes()) == x["effective_rgb_sha256"]
            verified += 1
    return verified


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True)
    packet = Path(__file__).resolve().parent
    archives = restore(packet, root)
    cases = json.loads((root / "diagnostics/portable-cases.json").read_text())["cases"]
    assert len(cases) == 33735
    refs = {(x["corpus"], x["model"], x["id"]): x for x in cases}
    for x in cases:
        assert digest((root / x["rgb"]).read_bytes()) == x["rgb_sha256"]
        assert (root / "data/truth" / x["corpus"] / (x["id"] + ".txt")).read_text() == x["truth"]
        for phase in ["baseline", "candidate"]:
            target = root / "replayed" / x["corpus"] / x["model"] / phase / (x["id"] + ".txt")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(x[phase])
    scorer = root / "diagnostics/source/eval_ocr_metrics.py"
    tables = 0
    for corpus, folder in [("old", "scores"), ("spaces", "scores"),
                           ("fresh", "fresh-policy/scores"), ("horizontal", "horizontal-policy/scores")]:
        for model in ["30", "50", "100"]:
            for norm in ["none", "width", "strict"]:
                base = root / "replayed" / corpus / model
                expected = root / "text" / folder / corpus / (model + "-" + norm + ".csv")
                score(scorer, root / "data/truth" / corpus, base / "baseline", base / "candidate", norm,
                      root / "reproduced" / folder / corpus / expected.name, expected, ["baseline", "candidate"])
                tables += 2
    for mode in ["deim", "bands", "runtime30_eval", "runtime30_default", "runtime50_eval",
                 "runtime50_default", "runtime100_eval", "runtime100_default"]:
        before = root / "text/page-before" / mode
        after = root / "text/cli-after" / mode if mode in ["deim", "bands"] else root / "text/page-after" / ("primary50" if mode.startswith("runtime50") else "after") / mode
        if (before / "results.json").exists():
            assert json.loads((before / "results.json").read_text()) == json.loads((after / "results.json").read_text()), mode
        for norm in ["none", "width", "strict"]:
            expected = root / "text/page-scores" / (mode + "-" + norm + ".csv")
            score(scorer, root / "data/page/truth", before, after, norm,
                  root / "reproduced/page-scores" / expected.name, expected, ["before", "after"])
            tables += 2
    native_pins = json.loads((root / "diagnostics/native-manifest.json").read_text())["cases"]
    native_by_id = {(x["corpus"], x["model"], x["id"]): x for x in native_pins}
    source_means = {}
    old = json.loads((root / "diagnostics/parents/old-probe.json").read_text())
    old_api = json.loads((root / "diagnostics/parents/old-after-api.json").read_text())
    original_texts = {m["label"]: {x["id"]: x["text"] for x in m["actual"]} for m in old_api["models"]}
    for model in old["models"]:
        label = model["label"].removeprefix("single")
        for x in model["rows"]:
            key = "old", label, x["id"]
            assert refs[key]["raw"] == x["raw"]
            assert refs[key]["baseline"] == original_texts[model["label"]][x["id"]]
            source_means[key] = struct.unpack("<I", struct.pack("<f", x["mean_confidence"]))[0]
    spaces = json.loads((root / "diagnostics/parents/space-native-rows.json").read_text())
    assert len(spaces) == 13794
    for x in spaces:
        assert refs["spaces", x["model"], x["id"]]["raw"] == x["base"]["text"]
    for folder in ["fresh", "horizontal"]:
        for label in ["30", "50", "100"]:
            rows = json.loads((root / "diagnostics" / folder / (label + "-export.json")).read_text())
            for x in rows:
                key = folder, label, x["id"]
                assert refs[key]["raw"] == x["text"]
                source_means[key] = emitted_mean_bits(x)
    for key, pin in native_by_id.items():
        assert source_means[key] == pin["mean_bits"], key
    for model in ["30", "50", "100"]:
        rows = json.loads((root / "diagnostics" / ("native" + model + "-results.json")).read_text())
        assert len(rows) == 915
        for x in rows:
            key = x["corpus"], model, x["id"]
            assert x["text"] == refs[key]["candidate"] and x["raw"] == refs[key]["raw"]
            assert x["mean_bits"] == native_by_id[key]["mean_bits"]
    cli = json.loads((root / "diagnostics/line-cli-results.json").read_text())
    assert len(cli) == 2361
    for x in cli:
        key = x["corpus"], x["model"], x["id"]
        assert x["exit_code"] == 0 and x["text"] == refs[key]["candidate"]
        path = root / "text/line-cli" / (x["model"] + "-" + x["corpus"] + "-" + x["id"] + ".stdout")
        stdout = path.read_bytes()
        assert stdout[-1:] == b"\n" and stdout[:-1].decode() == x["text"]
    rgb_captures = verify_pixels(root)
    portable = [x | {"rgb": str(root / x["rgb"]), "expected": x["candidate"]} for x in cases]
    (root / "product-manifest.json").write_text(json.dumps({"cases": portable}, ensure_ascii=False, indent=2) + "\n")
    result = {"archive_sha256": archives, "table_files_byte_equal": tables,
              "product_pairs": len(cases), "native_pairs": len(native_pins),
              "cli_processes": len(cli), "actual_rgb_captures_verified": rgb_captures,
              "page_metadata_unchanged": True, "char_and_word_regressions": 0}
    (root / "verification.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
