#!/usr/bin/env python3
"""Restore the parent evidence and reproduce the detached-body comparisons without ONNX."""
from pathlib import Path
import argparse
import csv
import hashlib
import importlib.util
import json
import struct
import sys
import unicodedata
import zipfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def restore(packet, output, names):
    hashes = {}
    for name in names:
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


def json_file(root, path):
    return json.loads((root / path).read_text())


def load_parent(packet, root):
    parents = json_file(packet, "parents.json")
    directory = packet.parent / parents["packet"]
    script = directory / "reproduce.py"
    assert digest(script.read_bytes()) == parents["reproducer_sha256"]
    for name, h in parents["archives"].items():
        assert json_file(directory, name + ".json")["sha256"] == h
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("verified_parent_reproducer", script)
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    hashes = helper.restore(directory, root / "parent")
    assert hashes == parents["archives"]
    return helper, hashes


def verify_pixels(root):
    from PIL import Image, ImageChops
    captures = 0
    noise_pairs = 0
    native_commands = 0
    for prefix, folder, count in [("initial", "fresh", 10944), ("positive", "fresh", 10944),
                                  ("positive", "horizontal", 64)]:
        data = root / prefix / folder
        m = json_file(data, "evaluation-manifest.json")
        rows = m["inputs"]
        assert len(rows) == count

        def resolve(path):
            for old, new in [("tmp/detached-body", "initial"), ("tmp/detached-positive", "positive")]:
                if Path(path).is_relative_to(old):
                    return root / new / Path(path).relative_to(old)
            raise AssertionError(path)

        atlas = Image.open(data / "atlas.png").convert("RGB")
        assert digest((data / "atlas.png").read_bytes()) == m["atlas_sha256"]
        assert digest((data / "lines.json").read_bytes()) == m["lines_sha256"]
        for x in rows:
            for key in ["rgb", "effective_rgb", "truth", "image", "original"]:
                assert digest(resolve(x[key]).read_bytes()) == x[key + "_sha256"], x["id"]
            original = Image.open(resolve(x["image"])).convert("RGB")
            assert original.tobytes() == resolve(x["rgb"]).read_bytes()
            effective = resolve(x["effective_rgb"]).read_bytes()
            assert atlas.crop(x["bounds"]).tobytes() == effective
            if folder == "fresh":
                rotated = 5 * x["height"] > 4 * x["width"]
                assert rotated == x["rotated_for_auto"]
                transformed = original.transpose(Image.Transpose.ROTATE_90) if rotated else original
                assert transformed.tobytes() == effective
                if x["kind"] == "noise" and x["format"] == "png":
                    stem = f'd_t{x["body_id"]}_bs{x["body_spaces"]}_{x["font"]}_{x["size"]}_p{x["palt"]}_bare'
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
                    noise_pairs += 1
            else:
                assert x["requested_orientation"] == "horizontal" and x["original_auto_would_rotate"]
                assert effective == original.tobytes()
        cap = json_file(data, "whole-rgb-capture/inputs.json")
        assert cap["model_input_dimensions"] == [256, 24] and len(cap["chunks"]) == count
        by_id = {x["id"]: x for x in rows}
        assert {x["line_id"] for x in cap["chunks"]} == set(by_id)
        for c in cap["chunks"]:
            x = by_id[c["line_id"]]
            im = Image.open(data / "whole-rgb-capture" / c["image"]).convert("RGB")
            assert list(im.size) == c["dimensions"] == [x["forward_width"], x["forward_height"]]
            assert c["chunk_index"] == 0 and c["x_range"] == [0, x["forward_width"]]
            assert digest(im.tobytes()) == x["effective_rgb_sha256"]
            captures += 1
        if folder == "fresh":
            assert len(m["native_commands"]) == 2232
            for command in m["native_commands"]:
                native = data / "native" / command["id"]
                trace_bytes = native.with_suffix(".json").read_bytes()
                assert digest(trace_bytes) == command["trace_sha256"]
                assert digest(native.with_suffix(".png").read_bytes()) == command["png_sha256"]
                trace = json.loads(trace_bytes)
                normalize = lambda p: unicodedata.normalize("NFC", p)
                assert normalize(trace["font_file"]) == normalize(command["argv"][1])
                for run in trace["runs"]:
                    assert normalize(run["font_file"]) == normalize(trace["font_file"])
                    assert run["font"] == trace["font"] and 0 not in run["glyphs"]
                assert sorted(i for run in trace["runs"] for i in run["string_indices"]) == list(range(len(command["argv"][4])))
                native_commands += 1
            tensor = json_file(root, "diagnostics/" + prefix + "/fresh/tensor-proof.json")
            assert tensor == {"images": 10944, "model_shapes": 3,
                              "all_original_auto_and_effective_horizontal_tensors_bit_equal": True}
            portable = {**m, "inputs": [{**x, **{k: str(resolve(x[k])) for k in ["rgb", "effective_rgb"]}} for x in rows]}
            harness = root / ("tensor-" + prefix) / "fresh"
            harness.mkdir(parents=True)
            (harness / "evaluation-manifest.json").write_text(json.dumps(portable, ensure_ascii=False, indent=2) + "\n")
    return captures, noise_pairs, native_commands


def reproduce_audit(helper, root, cases, corpora, expected_dir, expected_audit, prior=False):
    audit = {"summary": [], "changes": [], "char_regressions": [], "word_regressions": []}
    scorer = root / "diagnostics/source/eval_ocr_metrics.py"
    count_by_corpus = {c: len({x["id"] for x in cases if x["corpus"] == c}) for c in corpora}
    for corpus in corpora:
        for model in ["30", "50", "100"]:
            base = root / "generated" / ("prior" if prior else "core") / corpus / model
            for norm in ["none", "width", "strict"]:
                name = model + "-" + norm + ".csv"
                expected = root / expected_dir / corpus / name
                target = root / "reproduced" / expected_dir / corpus / name
                helper.score(scorer, root / "truth" / corpus, base / "baseline", base / "candidate", norm,
                             target, expected, ["baseline", "candidate"])
                rows = list(csv.DictReader(target.open()))
                b = {x["file"]: x for x in rows if x["system"] == "baseline"}
                a = {x["file"]: x for x in rows if x["system"] == "candidate"}
                assert len(a) == len(b) == count_by_corpus[corpus]
                stat = {"corpus": corpus, "model": model, "normalization": norm,
                        "images": len(a), "changes": 0, "char_improvements": 0,
                        "word_improvements": 0, "exact_improvements": 0}
                for id in b:
                    bc, ac = int(b[id]["char_errors"]), int(a[id]["char_errors"])
                    bw, aw = int(b[id]["word_errors"]), int(a[id]["word_errors"])
                    changed = (base / "baseline" / id).read_bytes() != (base / "candidate" / id).read_bytes()
                    stat["changes"] += changed
                    stat["char_improvements"] += ac < bc
                    stat["word_improvements"] += aw < bw
                    stat["exact_improvements"] += bc > 0 and ac == 0
                    if changed:
                        audit["changes"].append([corpus, model, norm, id, bc, ac, bw, aw])
                audit["summary"].append(stat)
    assert audit == json_file(root, expected_audit), expected_audit
    return len(corpora) * 18


def main():
    from PIL import Image
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True)
    packet = Path(__file__).resolve().parent
    helper, parents = load_parent(packet, root)
    names = ["initial-images", "initial-inputs", "positive-images", "positive-inputs", "diagnostics", "text"]
    hashes = restore(packet, root, names)
    cases = json_file(root, "diagnostics/portable-cases.json")["cases"]
    assert len(cases) == 99591
    refs = {(x["corpus"], x["model"], x["id"]): x for x in cases}
    assert len(refs) == len(cases) and sum(x["selected"] for x in cases) == 394
    parent_cases = json_file(root, "parent/diagnostics/portable-cases.json")["cases"]
    for x in parent_cases:
        y = refs[x["corpus"], x["model"], x["id"]]
        assert (x["raw"], x["truth"], x["candidate"], x["rgb_sha256"]) == (y["raw"], y["truth"], y["expected_before"], y["rgb_sha256"])
    verified_rgb = set()
    for x in cases:
        if x["rgb"] not in verified_rgb:
            assert digest((root / x["rgb"]).read_bytes()) == x["rgb_sha256"]
            verified_rgb.add(x["rgb"])
        truth = root / "truth" / x["corpus"] / (x["id"] + ".txt")
        truth.parent.mkdir(parents=True, exist_ok=True)
        if truth.exists():
            assert truth.read_bytes() == x["truth"].encode()
        truth.write_bytes(x["truth"].encode())
        for phase, field in [("baseline", "expected_before"), ("candidate", "expected_after")]:
            path = root / "generated/core" / x["corpus"] / x["model"] / phase / (x["id"] + ".txt")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(x[field].encode())
    tables = 0
    configurations = [(["old", "spaces", "fresh", "horizontal"], "text/initial/scores", "diagnostics/initial/audit.json"),
                      (["new"], "text/initial/fresh-policy/scores", "diagnostics/initial/fresh-policy/audit.json"),
                      (["positive"], "text/positive/fresh-policy/scores", "diagnostics/positive/fresh-policy/audit.json"),
                      (["positive-horizontal"], "text/positive/horizontal-policy/scores", "diagnostics/positive/horizontal-policy/audit.json")]
    for corpora, directory, audit in configurations:
        tables += reproduce_audit(helper, root, cases, corpora, directory, audit)
    for prefix, folder, corpus in [("initial", "previous-bridge-audit", "new"),
                                   ("positive", "previous-bridge-audit", "positive"),
                                   ("positive", "horizontal-previous-bridge-audit", "positive-horizontal")]:
        records = json_file(root, f"diagnostics/{prefix}/{folder}/existing-replay/records.json")
        for x in records:
            pin = refs[x["corpus"], x["model"], x["id"]]
            assert x["raw"] == pin["raw"] and x["candidate"] == pin["expected_before"]
            for phase in ["baseline", "candidate"]:
                path = root / "generated/prior" / corpus / x["model"] / phase / (x["id"] + ".txt")
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(x[phase].encode())
        tables += reproduce_audit(helper, root, cases, [corpus], f"text/{prefix}/{folder}/scores", f"diagnostics/{prefix}/{folder}/audit.json", True)
    assert tables == 180
    means = {}
    old = json_file(root, "parent/diagnostics/parents/old-probe.json")
    for model in old["models"]:
        for x in model["rows"]:
            means["old", model["label"].removeprefix("single"), x["id"]] = struct.unpack("<I", struct.pack("<f", x["mean_confidence"]))[0]
    forwards = 0
    for prefix, folder, corpus in [("initial", "fresh", "new"), ("positive", "fresh", "positive"),
                                   ("positive", "horizontal", "positive-horizontal")]:
        m = json_file(root, f"{prefix}/{folder}/evaluation-manifest.json")
        inputs = {x["id"]: x for x in m["inputs"]}
        for model in ["30", "50", "100"]:
            rows = json_file(root, f"diagnostics/{prefix}/{folder}/{model}-export.json")
            assert len(rows) == len(inputs) and {x["id"] for x in rows} == set(inputs)
            for x in rows:
                key = corpus, model, x["id"]
                assert x["text"] == refs[key]["raw"] and x["bounds"] == inputs[x["id"]]["bounds"]
                assert len(x["chunks"]) == 1 and x["chunks"][0]["x_range"] == [0, inputs[x["id"]]["forward_width"]]
                assert not x["chunks"][0]["inserted_space_before"]
                means[key] = helper.emitted_mean_bits(x)
                forwards += 1
    assert forwards == 65856
    native = json_file(root, "diagnostics/initial/native-manifest.json")["cases"]
    assert len(native) == 1882
    pins = {(x["corpus"], x["model"], x["id"]): x for x in native}
    seen = set()
    for model in ["30", "50", "100"]:
        rows = json_file(root, f"diagnostics/initial/native{model}-results.json")
        for x in rows:
            key = x["corpus"], model, x["id"]
            assert key not in seen and x["raw"] == refs[key]["raw"] and x["text"] == refs[key]["expected_after"]
            assert x["mean_bits"] == pins[key]["mean_bits"] == means[key]
            seen.add(key)
    assert seen == set(pins)
    cli = json_file(root, "diagnostics/initial/line-cli-results.json")
    cli_inputs = json_file(root, "diagnostics/initial/line-cli-command.json")["cases"]
    cli_by_id = {(x["corpus"], x["model"], x["id"]): x for x in cli_inputs}
    assert len(cli) == 682 and sum(x["selected"] for x in cli) == 394
    for x in cli:
        key = x["corpus"], x["model"], x["id"]
        assert x["exit_code"] == 0 and x["text"] == refs[key]["expected_after"]
        name = x["model"] + "-" + x["corpus"] + "-" + x["id"]
        stdout = (root / "text/initial/line-cli" / (name + ".stdout")).read_bytes()
        assert stdout[-1:] == b"\n" and stdout[:-1].decode() == x["text"]
        image = root / "text/initial/line-cli/images" / (x["corpus"] + "-" + x["id"] + ".png")
        assert digest(image.read_bytes()) == cli_by_id[key]["image_sha256"]
        assert Image.open(image).convert("RGB").tobytes() == (root / refs[key]["rgb"]).read_bytes()
    captures, noise_pairs, native_commands = verify_pixels(root)
    assert (captures, noise_pairs, native_commands) == (21952, 2304, 4464)
    portable = [x | {"rgb": str(root / x["rgb"])} for x in cases]
    (root / "product-manifest.json").write_text(json.dumps({"cases": portable}, ensure_ascii=False, indent=2) + "\n")
    result = {"parent_archives": parents, "archive_sha256": hashes, "table_files_byte_equal": tables,
              "audits_recomputed_equal": 7, "product_pairs": len(cases), "new_whole_roi_nn_forwards": forwards,
              "native_pairs": len(native), "cli_processes": len(cli), "actual_rgb_captures_verified": captures,
              "pre_jpeg_noise_pairs_verified": noise_pairs, "native_renderer_commands_verified": native_commands,
              "char_and_word_regressions": 0}
    (root / "verification.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
