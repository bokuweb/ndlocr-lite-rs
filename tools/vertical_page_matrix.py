"""Frozen-source Page/CLI accuracy matrix helpers and runner."""
import hashlib
import json
import math
from pathlib import Path, PurePosixPath
import re
import zipfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check_job(meta, bundle_digest, kind, physical_count):
    if (meta.get("complete") is not True or meta.get("kind") != kind
            or meta.get("bundle_sha256") != bundle_digest or meta.get("physical_inputs") != physical_count
            or not meta.get("commands") or any(type(x.get("exit_code")) is not int or x["exit_code"] != 0 for x in meta["commands"])):
        raise ValueError("partial job or mismatched source/data bindings")


def verify_command(path, expected):
    receipt = json.loads(Path(path).read_text())
    if receipt.get("exit_code") != 0 or receipt.get("argv") != [str(x) for x in expected]:
        raise ValueError("actual command differs from declared executable/model/input")
    return receipt


def cli_command(binary, image, models, assets, mode, body):
    return [str(binary), "recognize-page", "--image", str(image),
            "--det-model", str(models / assets["deim"]["name"]),
            "--model", str(models / assets["100"]["name"]),
            "--model30", str(models / assets["30"]["name"]),
            "--model50", str(models / assets["50"]["name"]),
            "--charset", str(models / assets["charset"]["name"]),
            "--use-deim-detection", str(mode == "deim").lower(), "--output-txt", str(body)]


def safe_member(name):
    path = PurePosixPath(name)
    if (not name or path.is_absolute() or "\\" in name or ":" in name
            or ".." in path.parts or path.as_posix() != name or name.endswith("/")):
        raise ValueError(f"unsafe member: {name}")
    return path


class Bundle:
    def __init__(self, folder):
        self.folder = Path(folder)
        self.index = json.loads((self.folder / "bundle.json").read_text())
        self._payload = {}
        if self.index["schema"] != 1:
            raise ValueError("unsupported bundle schema")
        for name, meta in self.index["archives"].items():
            safe_member(name)
            data = (self.folder / name).read_bytes()
            if len(data) != meta["bytes"] or sha(data) != meta["sha256"]:
                raise ValueError(f"archive mismatch: {name}")
            with zipfile.ZipFile(self.folder / name) as archive:
                members = self.index["members"][name]
                if len(archive.namelist()) != len(set(archive.namelist())) or set(archive.namelist()) != set(members):
                    raise ValueError("missing, extra or duplicate members")
                for member, expected in members.items():
                    safe_member(member)
                    payload = archive.read(member)
                    if len(payload) != expected["bytes"] or sha(payload) != expected["sha256"]:
                        raise ValueError(f"member mismatch: {member}")
                    self._payload[name, member] = payload
        physical = self.index["physical_inputs"]
        scoring = self.index["scoring_inputs"]
        ids = [x["id"] for x in physical]
        score_ids = [x["id"] for x in scoring]
        if (not ids or len(ids) != len(set(ids)) or len(score_ids) != len(set(score_ids))
                or any(not re.fullmatch(r"[A-Za-z0-9_-]+", x) for x in ids + score_ids)):
            raise ValueError("missing, duplicate or unsafe input IDs")
        expected = self.index["expected"]
        if (len(ids) != expected["physical_inputs"] or len(score_ids) != expected["scoring_inputs"]
                or len({(x["width"], x["height"], x["rgb_sha256"]) for x in physical}) != expected["unique_RGB"]):
            raise ValueError("input counts differ from frozen specification")
        for row in physical:
            if sha(self.read("data.zip", row["image"])) != row["image_sha256"]:
                raise ValueError("image binding mismatch")
        for row in scoring:
            if row["physical_id"] not in ids or sha(self.read("data.zip", row["truth"])) != row["truth_sha256"]:
                raise ValueError("GT or physical prediction binding mismatch")
        if {x["physical_id"] for x in scoring} != set(ids):
            raise ValueError("unscored physical input")
        for variant, files in self.index["source_files"].items():
            for name, digest in files.items():
                if sha(self.read("sources.zip", variant + "/" + name)) != digest:
                    raise ValueError("source binding mismatch")

    def read(self, archive, member):
        try:
            return self._payload[archive, member]
        except KeyError as error:
            raise ValueError(f"unbound member: {archive}/{member}") from error

    def materialize(self, destination):
        destination = Path(destination)
        destination.mkdir(parents=True, exist_ok=False)
        for (_, member), data in self._payload.items():
            target = destination / safe_member(member)
            if target.exists():
                raise ValueError("overlapping archive members")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)


def check_binary_pair(baseline, candidate):
    for value in [*baseline.values(), *candidate.values()]:
        if not re.fullmatch(r"[0-9a-f]{64}", value):
            raise ValueError("invalid executable hash")
    if baseline["page"] == candidate["page"]:
        raise ValueError("baseline and candidate Page executable are identical")


def check_build_artifacts(events, manifest):
    found = {}
    wanted = {("lib", "ndlocr_lite_rs"): "lib", ("bin", "ndlocr-lite-rs"): "cli",
              ("example", "capture_page_matrix"): "page"}
    for event in events:
        if event.get("reason") != "compiler-artifact" or Path(event["manifest_path"]).resolve() != Path(manifest).resolve():
            continue
        target = event["target"]
        key = wanted.get((target["kind"][0], target["name"]))
        if key is None:
            continue
        if (key in found or event.get("fresh") is not False or event.get("profile", {}).get("test") is not False
                or "onnx" not in event["features"]):
            raise ValueError("stale, duplicate or test executable in primary build")
        found[key] = event
    if set(found) != {"lib", "cli", "page"}:
        raise ValueError("incomplete primary build")
    return found


def map_predictions(inputs, predictions):
    if {x["physical_id"] for x in inputs} != set(predictions):
        raise ValueError("missing or extra physical predictions")
    return {x["id"]: predictions[x["physical_id"]] for x in inputs}


def read_cli_result(stdout, body):
    text = Path(body).read_bytes()
    if Path(stdout).read_bytes() != text + b"\n":
        raise ValueError("CLI text differs from exact transport framing")
    return text.decode("utf-8")


def read_page_results(folder, model, inputs):
    expected = {x["id"]: x for x in inputs}
    results = {}
    try:
        for mode in ["eval", "default"]:
            directory = Path(folder) / ("runtime" + model + "_" + mode)
            rows = json.loads((directory / "results.json").read_text())
            if [x["id"] for x in rows] != list(expected):
                raise ValueError("partial, duplicate or wrongly ordered Page results")
            for row in rows:
                text = (directory / (row["id"] + ".txt")).read_bytes().decode("utf-8")
                if text != row["text"] or text != "\n".join(x["text"] for x in row["lines"]):
                    raise ValueError("literal Page text mismatch")
                if type(row["page_index"]) is not int or row["page_index"] != 0:
                    raise ValueError("unexpected page index")
                for line in row["lines"]:
                    confidence = line["confidence"]
                    if not isinstance(confidence, (int, float)) or isinstance(confidence, bool) or not math.isfinite(confidence) or not 0 <= confidence <= 1:
                        raise ValueError("invalid confidence")
                    box = line["bbox"]
                    source = expected[row["id"]]
                    if (len(box) != 4 or any(type(x) is not int for x in box)
                            or not (0 <= box[0] < box[2] <= source["width"] and 0 <= box[1] < box[3] <= source["height"])):
                        raise ValueError("invalid Page bbox")
            results[model + "_" + mode] = {x["id"]: x for x in rows}
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        raise ValueError("incomplete Page results") from error
    return results


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".partial")
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")
    temporary.replace(path)


def run_command(argv, label, evidence, meta, env=None):
    import os
    import subprocess
    import time
    directory = Path(evidence) / "commands"
    directory.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    with (directory / (label + ".stdout")).open("xb") as stdout, (directory / (label + ".stderr")).open("xb") as stderr:
        result = subprocess.run([str(x) for x in argv], stdout=stdout, stderr=stderr, env=os.environ | (env or {}))
    receipt = {"argv": [str(x) for x in argv], "exit_code": result.returncode, "seconds": time.monotonic() - started}
    write_json(directory / (label + ".json"), receipt)
    meta["commands"].append(receipt)
    if result.returncode:
        raise RuntimeError(f"{label}: exit {result.returncode}; raw output retained")


def build_variant(bundle, variant, work, evidence, target, models, repository, meta):
    import os
    import shutil
    import subprocess
    source = work / variant
    expected = bundle.index["source_files"][variant]
    for name, digest in expected.items():
        data = (source / name).read_bytes()
        if sha(data) != digest:
            raise ValueError("source changed before compilation")
        if variant == "baseline" and name != "Cargo.lock":
            original = subprocess.check_output(["git", "-C", str(repository), "show", bundle.index["baseline_ref"] + ":" + name])
            if original != data:
                raise ValueError("baseline differs from declared Git reference")
    if sha((source / "Cargo.lock").read_bytes()) != bundle.index["lock_sha256"]:
        raise ValueError("dependency lock mismatch")
    (source / "models").symlink_to(models, target_is_directory=True)
    (source / "docs").symlink_to(repository / "docs", target_is_directory=True)
    # Refresh after the preceding build, without changing any source bytes.
    for path in source.rglob("*.rs"):
        path.touch()
    env = {"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_JOBS": "2", "CARGO_PROFILE_RELEASE_DEBUG": "0"}
    run_command(["cargo", "build", "--locked", "--release", "--manifest-path", source / "Cargo.toml",
                 "--features", "onnx", "--bin", "ndlocr-lite-rs", "--example", "capture_page_matrix",
                 "--message-format=json"], variant + "-build", evidence, meta, env)
    log = evidence / "commands" / (variant + "-build.stdout")
    events = [json.loads(line) for line in log.read_text().splitlines() if line.strip()]
    primary = check_build_artifacts(events, source / "Cargo.toml")
    binaries = {}
    for key in ["cli", "page"]:
        original = Path(primary[key]["executable"])
        destination = work / "binaries" / (variant + "-" + key)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, destination)
        destination.chmod(0o755)
        data = destination.read_bytes()
        if data != original.read_bytes():
            raise ValueError("frozen executable copy mismatch")
        binaries[key] = {"path": str(destination), "sha256": sha(data), "bytes": len(data)}
    if any(sha((source / name).read_bytes()) != digest for name, digest in expected.items()):
        raise ValueError("source changed during compilation")
    meta["builds"][variant] = {"source_manifest": str(source / "Cargo.toml"), "artifacts": primary,
                                "build_stdout_sha256": sha(log.read_bytes()), "binaries": binaries,
                                "source_files": expected}
    write_json(evidence / "job.json", meta)
    return binaries


def run_kind(bundle, kind, output, models):
    import os
    import platform
    import subprocess
    import sys
    repository = Path(__file__).resolve().parents[1]
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    evidence = output / "evidence"
    evidence.mkdir()
    work = output / "work"
    bundle.materialize(work)
    models = Path(models).resolve()
    for asset in bundle.index["assets"].values():
        if sha((models / asset["name"]).read_bytes()) != asset["sha256"]:
            raise ValueError("pinned model/charset mismatch")
    meta = {"kind": kind, "bundle_sha256": sha((bundle.folder / "bundle.json").read_bytes()),
            "physical_inputs": len(bundle.index["physical_inputs"]), "complete": False,
            "profile": "release", "platform": platform.platform(),
            "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
            "builds": {}, "commands": [], "assets": bundle.index["assets"], "models_dir": str(models),
            "internal_session_run_calls": None, "internal_caught_vertical_errors": None}
    write_json(evidence / "job.json", meta)
    try:
        target = output / "cargo-target"
        executables = {}
        for variant in ["baseline", "candidate"]:
            executables[variant] = build_variant(bundle, variant, work, evidence, target, models, repository, meta)
        check_binary_pair({k: v["sha256"] for k, v in executables["baseline"].items()},
                          {k: v["sha256"] for k, v in executables["candidate"].items()})
        directories = [target / "release", target / "release/deps"]
        directories += list((target / "release/build").glob("ort-sys-*/out/onnxruntime/lib"))
        libraries = {}
        for directory in directories:
            for path in directory.glob("*onnxruntime*"):
                if path.is_file():
                    libraries[str(path)] = sha(path.read_bytes())
        variable = "DYLD_LIBRARY_PATH" if sys.platform == "darwin" else "LD_LIBRARY_PATH"
        env = {"NDLOCR_INTRA_THREADS": "1", variable: os.pathsep.join(str(p) for p in directories)}
        meta["runtime_environment"] = env
        meta["runtime_libraries"] = libraries
        inputs = bundle.index["physical_inputs"]
        manifest = work / "NN-images-only.json"
        write_json(manifest, {"inputs": [{"id": x["id"], "image": str(work / x["image"])} for x in inputs]})
        meta["NN_manifest_sha256"] = sha(manifest.read_bytes())
        (evidence / "NN-images-only.json").write_bytes(manifest.read_bytes())
        if kind.startswith("page"):
            model = kind[4:]
            for variant in ["baseline", "candidate"]:
                directory = evidence / "page" / variant
                run_command([executables[variant]["page"]["path"], "--manifest", manifest,
                             "--det-model", models / bundle.index["assets"]["deim"]["name"],
                             "--parseq-model", models / bundle.index["assets"][model]["name"],
                             "--charset", models / bundle.index["assets"]["charset"]["name"],
                             "--label", "runtime" + model, "--output-dir", directory],
                            variant + "-page" + model, evidence, meta, env)
                read_page_results(directory, model, inputs)
            meta["Page_API_results"] = len(inputs) * 4
            meta["PageRecognizer_loads"] = 2
            meta["model_session_loads"] = 4
        elif kind == "cli":
            completed = 0
            for variant in ["baseline", "candidate"]:
                for mode in ["deim", "bands"]:
                    directory = evidence / "cli" / variant / mode
                    directory.mkdir(parents=True)
                    for row in inputs:
                        name = variant + "-cli-" + mode + "-" + row["id"]
                        body = directory / (row["id"] + ".txt")
                        run_command(cli_command(executables[variant]["cli"]["path"], work / row["image"], models,
                                                bundle.index["assets"], mode, body),
                                    name, evidence, meta, env)
                        read_cli_result(evidence / "commands" / (name + ".stdout"), body)
                        completed += 1
                        if completed % 100 == 0:
                            print(f"{completed}/{len(inputs) * 4} CLI processes complete", flush=True)
            meta["CLI_processes"] = completed
            meta["model_session_loads"] = None
        else:
            raise ValueError("unknown matrix kind")
        for variant, files in bundle.index["source_files"].items():
            if any(sha((work / variant / name).read_bytes()) != digest for name, digest in files.items()):
                raise ValueError("source changed during inference")
        for asset in bundle.index["assets"].values():
            if sha((models / asset["name"]).read_bytes()) != asset["sha256"]:
                raise ValueError("model changed during inference")
        for path, digest in libraries.items():
            if sha(Path(path).read_bytes()) != digest:
                raise ValueError("inference library changed")
        meta["all_frozen_materials_unchanged"] = True
        meta["complete"] = True
    except Exception as error:
        meta["error"] = repr(error)
        raise
    finally:
        write_json(evidence / "job.json", meta)


def verify_matrix(bundle, artifacts, output):
    import csv
    import importlib.util
    import string
    import subprocess
    import sys
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    work = output / "work"
    bundle.materialize(work)
    inputs = bundle.index["physical_inputs"]
    scoring = bundle.index["scoring_inputs"]
    digest = sha((bundle.folder / "bundle.json").read_bytes())
    jobs = {}
    for path in Path(artifacts).rglob("job.json"):
        meta = json.loads(path.read_text())
        kind = meta["kind"]
        if kind in jobs or kind not in ["page30", "page50", "page100", "cli"]:
            raise ValueError("extra or duplicate job artifact")
        check_job(meta, digest, kind, len(inputs))
        if not meta.get("all_frozen_materials_unchanged") or meta["assets"] != bundle.index["assets"]:
            raise ValueError("changed inference assets")
        folder = path.parent
        nn_manifest = folder / "NN-images-only.json"
        if sha(nn_manifest.read_bytes()) != meta["NN_manifest_sha256"]:
            raise ValueError("NN manifest mismatch")
        original_work = Path(meta["builds"]["baseline"]["source_manifest"]).parent.parent
        expected_nn = {"inputs": [{"id": x["id"], "image": str(original_work / x["image"])} for x in inputs]}
        if json.loads(nn_manifest.read_text()) != expected_nn:
            raise ValueError("NN inputs differ from frozen pixels or include GT")
        hashes = {}
        for variant in ["baseline", "candidate"]:
            build = meta["builds"][variant]
            log = folder / "commands" / (variant + "-build.stdout")
            if sha(log.read_bytes()) != build["build_stdout_sha256"] or build["source_files"] != bundle.index["source_files"][variant]:
                raise ValueError("build evidence mismatch")
            primary = check_build_artifacts([json.loads(x) for x in log.read_text().splitlines() if x.strip()], build["source_manifest"])
            if primary != build["artifacts"]:
                raise ValueError("changed primary build artifacts")
            hashes[variant] = {k: x["sha256"] for k, x in build["binaries"].items()}
        check_binary_pair(hashes["baseline"], hashes["candidate"])
        expected_commands = 4 if kind.startswith("page") else len(inputs) * 4 + 2
        if len(meta["commands"]) != expected_commands:
            raise ValueError("missing command receipts")
        if kind.startswith("page") and meta["Page_API_results"] != len(inputs) * 4:
            raise ValueError("Page result count mismatch")
        if kind == "cli" and meta["CLI_processes"] != len(inputs) * 4:
            raise ValueError("CLI process count mismatch")
        assets = bundle.index["assets"]
        models = Path(meta["models_dir"])
        original_evidence = original_work.parent / "evidence"
        command_index = 2
        for variant in ["baseline", "candidate"]:
            if kind.startswith("page"):
                model = kind[4:]
                expected = [meta["builds"][variant]["binaries"]["page"]["path"], "--manifest", original_work / "NN-images-only.json",
                            "--det-model", models / assets["deim"]["name"], "--parseq-model", models / assets[model]["name"],
                            "--charset", models / assets["charset"]["name"], "--label", "runtime" + model,
                            "--output-dir", original_evidence / "page" / variant]
                receipt = verify_command(folder / "commands" / (variant + "-page" + model + ".json"), expected)
                if meta["commands"][command_index] != receipt:
                    raise ValueError("Page receipt differs from job journal")
                command_index += 1
            else:
                for mode in ["deim", "bands"]:
                    for row in inputs:
                        name = variant + "-cli-" + mode + "-" + row["id"]
                        expected = cli_command(meta["builds"][variant]["binaries"]["cli"]["path"],
                                               original_work / row["image"], models, assets, mode,
                                               original_evidence / "cli" / variant / mode / (row["id"] + ".txt"))
                        receipt = verify_command(folder / "commands" / (name + ".json"), expected)
                        if meta["commands"][command_index] != receipt:
                            raise ValueError("CLI receipt differs from job journal")
                        command_index += 1
        jobs[kind] = folder
    if set(jobs) != {"page30", "page50", "page100", "cli"}:
        raise ValueError("full eight-route matrix is incomplete")
    truth = output / "metrics/truth"
    truth.mkdir(parents=True)
    for row in scoring:
        (truth / (row["id"] + ".txt")).write_bytes(bundle.read("data.zip", row["truth"]))
    systems = {}
    literal_changes = []
    all_text = {}
    for kind, folder in jobs.items():
        for variant in ["baseline", "candidate"]:
            if kind.startswith("page"):
                routes = read_page_results(folder / "page" / variant, kind[4:], inputs)
                texts = {route: {ident: row["text"] for ident, row in values.items()} for route, values in routes.items()}
            else:
                texts = {}
                for mode in ["deim", "bands"]:
                    values = {}
                    for row in inputs:
                        ident = row["id"]
                        name = variant + "-cli-" + mode + "-" + ident
                        values[ident] = read_cli_result(folder / "commands" / (name + ".stdout"),
                                                       folder / "cli" / variant / mode / (ident + ".txt"))
                    texts["cli_" + mode] = values
            for route, predictions in texts.items():
                mapped = map_predictions(scoring, predictions)
                directory = output / "metrics" / (variant + "_" + route)
                directory.mkdir()
                for ident, text in mapped.items():
                    (directory / (ident + ".txt")).write_bytes(text.encode())
                systems[variant + "_" + route] = directory
                all_text[variant, route] = mapped
    routes = ["30_eval", "30_default", "50_eval", "50_default", "100_eval", "100_default", "cli_deim", "cli_bands"]
    for route in routes:
        for row in scoring:
            before, after = [all_text[x, route][row["id"]] for x in ["baseline", "candidate"]]
            if before != after:
                literal_changes.append({"id": row["id"], "physical_id": row["physical_id"], "route": route,
                                        "cohort": row["cohort"], "before": before, "after": after})
    def normalize_independently(text, level):
        if level == "none":
            return text
        if level == "width":
            return "".join(chr(ord(c) - 65248) if 65281 <= ord(c) <= 65374 else c for c in text)
        folded = "\n".join(" ".join(line.split()) for line in text.replace("\r\n", "\n").replace("\r", "\n").split("\n"))
        return "".join(c for c in folded if c not in string.punctuation and c != " ").strip()
    independent_file = work / "scoring/fast_distance.py"
    spec = importlib.util.spec_from_file_location("vertical_matrix_distance", independent_file)
    independent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(independent)
    scorer = work / "scoring/eval_ocr_metrics.py"
    if sha(scorer.read_bytes()) != bundle.index["eval_ocr_metrics.py_sha256"] or sha(independent_file.read_bytes()) != bundle.index["fast_distance.py_sha256"]:
        raise ValueError("changed scorers")
    comparisons, regressions, improvements, totals, commands = [], [], [], [], []
    verified = 0
    for normalization in ["none", "width", "strict"]:
        table = output / "metrics" / (normalization + ".csv")
        argv = [sys.executable, str(scorer), "--truth-dir", str(truth)]
        for name, directory in systems.items():
            argv += ["--system", name + "=" + str(directory)]
        argv += ["--normalize", normalization, "--require-all", "--baseline-system", "baseline_cli_deim",
                 "--output-csv", str(table), "--output-md", str(table.with_suffix(".md"))]
        process = subprocess.run(argv, capture_output=True)
        table.with_suffix(".stdout").write_bytes(process.stdout)
        table.with_suffix(".stderr").write_bytes(process.stderr)
        commands.append({"argv": argv, "exit_code": process.returncode})
        if process.returncode:
            raise ValueError("official scorer failed")
        with table.open(newline="") as stream:
            rows = list(csv.DictReader(stream))
        if len(rows) != len(scoring) * 16:
            raise ValueError("missing official metric rows")
        index = {(x["system"], x["file"]): x for x in rows}
        for row in rows:
            actual = normalize_independently((truth / row["file"]).read_text(), normalization)
            predicted = normalize_independently((systems[row["system"]] / row["file"]).read_text(), normalization)
            expected = {"char_errors": independent.distance(actual, predicted), "char_total": max(1, len(actual)),
                        "word_errors": independent.distance(actual.split(), predicted.split()), "word_total": max(1, len(actual.split()))}
            if any(int(row[key]) != value for key, value in expected.items()):
                raise ValueError("official/independent metric disagreement")
            verified += 1
        for route in routes:
            for source in scoring:
                a, b = [index[x + "_" + route, source["id"] + ".txt"] for x in ["baseline", "candidate"]]
                if a["char_total"] != b["char_total"] or a["word_total"] != b["word_total"]:
                    raise ValueError("GT denominator changed")
                row = {"id": source["id"], "physical_id": source["physical_id"], "cohort": source["cohort"],
                       "route": route, "normalization": normalization,
                       "baseline_char_errors": int(a["char_errors"]), "candidate_char_errors": int(b["char_errors"]),
                       "baseline_word_errors": int(a["word_errors"]), "candidate_word_errors": int(b["word_errors"])}
                comparisons.append(row)
                if row["candidate_char_errors"] > row["baseline_char_errors"] or row["candidate_word_errors"] > row["baseline_word_errors"]:
                    regressions.append(row)
                if row["candidate_char_errors"] < row["baseline_char_errors"] or row["candidate_word_errors"] < row["baseline_word_errors"]:
                    improvements.append(row)
            for variant in ["baseline", "candidate"]:
                group = [index[variant + "_" + route, row["id"] + ".txt"] for row in scoring]
                totals.append({"route": route, "normalization": normalization, "variant": variant,
                               **{key: sum(int(x[key]) for x in group) for key in ["char_errors", "char_total", "word_errors", "word_total"]}})
    fresh = [x for x in improvements if x["cohort"] == bundle.index["fresh_gain_cohort"] and x["normalization"] == "none"
             and x["candidate_char_errors"] < x["baseline_char_errors"] and x["candidate_word_errors"] <= x["baseline_word_errors"]]
    cli_changes = [x for x in literal_changes if x["route"].startswith("cli_")]
    summary = {"physical_input_IDs": len(inputs), "scoring_IDs": len(scoring),
               "declared_unique_RGB": bundle.index["expected"]["unique_RGB"],
               "independently_verified_CSV_rows": verified, "char_and_word_comparisons": len(comparisons) * 2,
               "per_input_regressions": regressions, "per_input_improvements": improvements,
               "fresh_raw_gain_pairs": fresh, "literal_text_changes": literal_changes, "CLI_text_changes": cli_changes,
               "totals": totals, "matrix_gate_pass": not regressions and bool(fresh) and not cli_changes,
               "root_inference_integrated": False, "new_NN_calls_this_audit": 0,
               "bundle_sha256": digest, "all_four_jobs_complete": True}
    for name, value in [("summary", summary), ("all-input-pairs", comparisons), ("commands", commands)]:
        write_json(output / (name + ".json"), value)
    print(json.dumps({key: value for key, value in summary.items() if key not in ["per_input_regressions", "per_input_improvements", "fresh_raw_gain_pairs", "literal_text_changes", "CLI_text_changes", "totals"]}, indent=2))
    if not summary["matrix_gate_pass"]:
        raise ValueError("FVB held: regression, missing fresh gain, or unintended CLI change")


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, default=Path(__file__).resolve().parents[1] / "docs/benchmarks/vertical-page-fallback")
    parser.add_argument("--kind", choices=["page30", "page50", "page100", "cli"])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--models", type=Path, default=Path("models"))
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--validate", action="store_true")
    args = parser.parse_args()
    bundle = Bundle(args.bundle)
    if args.validate:
        print(json.dumps(bundle.index["expected"], indent=2))
    elif args.artifacts and args.output:
        try:
            verify_matrix(bundle, args.artifacts, args.output)
        except Exception as error:
            write_json(args.output / "failure.json", {"matrix_gate_pass": False, "error": repr(error)})
            raise
    elif args.kind and args.output:
        run_kind(bundle, args.kind, args.output, args.models)
    else:
        parser.error("use --validate, --kind with --output, or --artifacts with --output")


if __name__ == "__main__":
    main()
