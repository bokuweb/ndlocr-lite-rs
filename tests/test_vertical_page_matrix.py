"""The full matrix must reject stale executables, partial output, and altered data."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from vertical_page_matrix import (
    Bundle, check_binary_pair, check_build_artifacts, check_job, cli_command, map_predictions,
    read_cli_result, read_page_results, verify_command, verify_matrix,
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


class MatrixTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def bundle(self, unsafe=False):
        payloads = {
            "data.zip": {"images/image.png": b"pixel fixture", "truths/a.txt": " 本文　".encode()},
            "sources.zip": {"baseline/src/lib.rs": b"baseline", "candidate/src/lib.rs": b"candidate"},
        }
        repository = Path(__file__).resolve().parents[1]
        for name, path in [("eval_ocr_metrics.py", repository / "tools/eval_ocr_metrics.py"),
                           ("fast_distance.py", repository / "docs/benchmarks/vertical-partition-agreement/fast_distance.py")]:
            payloads["sources.zip"]["scoring/" + name] = path.read_bytes()
        if unsafe:
            payloads["data.zip"]["../escape"] = b"bad"
        index = {"schema": 1, "archives": {}, "members": {},
                 "physical_inputs": [{"id": "one", "image": "images/image.png", "image_sha256": digest(b"pixel fixture"),
                                      "width": 1, "height": 1, "rgb_sha256": "a" * 64}],
                 "scoring_inputs": [{"id": "truth_a", "physical_id": "one", "truth": "truths/a.txt", "truth_sha256": digest(" 本文　".encode())},
                                    {"id": "truth_b", "physical_id": "one", "truth": "truths/a.txt", "truth_sha256": digest(" 本文　".encode())}],
                 "source_files": {"baseline": {"src/lib.rs": digest(b"baseline")}, "candidate": {"src/lib.rs": digest(b"candidate")}},
                 "expected": {"physical_inputs": 1, "scoring_inputs": 2, "unique_RGB": 1},
                 "assets": {key: {"name": key + ".asset", "sha256": "a" * 64} for key in ["deim", "charset", "30", "50", "100"]},
                 "fresh_gain_cohort": "fresh"}
        for row in index["scoring_inputs"]:
            row["cohort"] = "fresh"
        for name in ["eval_ocr_metrics.py", "fast_distance.py"]:
            index[name + "_sha256"] = digest(payloads["sources.zip"]["scoring/" + name])
        for archive, members in payloads.items():
            path = self.root / archive
            with zipfile.ZipFile(path, "w") as output:
                for name, data in members.items():
                    output.writestr(name, data)
            index["archives"][archive] = {"sha256": digest(path.read_bytes()), "bytes": path.stat().st_size}
            index["members"][archive] = {name: {"sha256": digest(data), "bytes": len(data)} for name, data in members.items()}
        self.write_index(index)
        return index

    def write_index(self, index):
        (self.root / "bundle.json").write_text(json.dumps(index))

    def test_verified_bundle_keeps_aliases_and_literal_whitespace(self):
        self.bundle()
        bundle = Bundle(self.root)
        self.assertEqual(bundle.read("data.zip", "truths/a.txt"), " 本文　".encode())
        self.assertEqual(map_predictions(bundle.index["scoring_inputs"], {"one": " \r\n本文　"}),
                         {"truth_a": " \r\n本文　", "truth_b": " \r\n本文　"})
        destination = self.root / "restored"
        bundle.materialize(destination)
        self.assertEqual((destination / "baseline/src/lib.rs").read_bytes(), b"baseline")

    def test_corrupt_archive_and_unsafe_members_are_rejected(self):
        self.bundle()
        with (self.root / "data.zip").open("ab") as output:
            output.write(b"altered")
        with self.assertRaises(ValueError):
            Bundle(self.root)
        self.bundle(unsafe=True)
        with self.assertRaises(ValueError):
            Bundle(self.root)

    def test_duplicate_ids_missing_bindings_and_changed_source_are_rejected(self):
        original = self.bundle()
        for mutate in [lambda x: x["physical_inputs"].append(copy.deepcopy(x["physical_inputs"][0])),
                       lambda x: x["scoring_inputs"][0].update(physical_id="missing"),
                       lambda x: x["source_files"]["candidate"].update({"src/lib.rs": "f" * 64})]:
            index = copy.deepcopy(original)
            mutate(index)
            self.write_index(index)
            with self.assertRaises(ValueError):
                Bundle(self.root)

    def artifacts(self):
        path = str((self.root / "Cargo.toml").resolve())
        return [{"reason": "compiler-artifact", "manifest_path": path, "fresh": False,
                 "features": ["default", "onnx"], "profile": {"test": False},
                 "target": {"kind": [kind], "name": name}, "executable": str(self.root / name)}
                for kind, name in [("lib", "ndlocr_lite_rs"), ("bin", "ndlocr-lite-rs"), ("example", "capture_page_matrix")]]

    def test_each_primary_artifact_must_be_fresh_and_non_test(self):
        events = self.artifacts()
        self.assertEqual(set(check_build_artifacts(events, self.root / "Cargo.toml")), {"lib", "cli", "page"})
        for key in ["fresh", "test"]:
            altered = copy.deepcopy(events)
            if key == "fresh":
                altered[0]["fresh"] = True
            else:
                altered[2]["profile"]["test"] = True
            with self.assertRaises(ValueError):
                check_build_artifacts(altered, self.root / "Cargo.toml")

    def test_equal_page_binary_cannot_be_presented_as_candidate(self):
        baseline = {"page": "a" * 64, "cli": "b" * 64}
        with self.assertRaises(ValueError):
            check_binary_pair(baseline, baseline)
        check_binary_pair(baseline, {"page": "c" * 64, "cli": "b" * 64})

    def page_output(self):
        inputs = [{"id": "one", "width": 2, "height": 3}]
        for mode in ["eval", "default"]:
            directory = self.root / ("runtime30_" + mode)
            directory.mkdir(exist_ok=True)
            text = " \r\n本文　"
            (directory / "one.txt").write_bytes(text.encode())
            (directory / "results.json").write_text(json.dumps([{
                "id": "one", "page_index": 0, "text": text,
                "lines": [{"bbox": [0, 0, 2, 3], "text": text, "confidence": 0.95}],
            }]))
        return inputs

    def test_page_output_needs_both_settings_and_complete_metadata(self):
        inputs = self.page_output()
        self.assertEqual(read_page_results(self.root, "30", inputs)["30_eval"]["one"]["text"], " \r\n本文　")
        path = self.root / "runtime30_default/results.json"
        path.unlink()
        with self.assertRaises(ValueError):
            read_page_results(self.root, "30", inputs)

    def test_wrong_page_ids_and_changed_literal_text_are_rejected(self):
        inputs = self.page_output()
        path = self.root / "runtime30_eval/results.json"
        original = json.loads(path.read_text())
        for altered in [[original[0], original[0]], [dict(original[0], id="wrong")]]:
            path.write_text(json.dumps(altered))
            with self.assertRaises(ValueError):
                read_page_results(self.root, "30", inputs)
        path.write_text(json.dumps(original))
        (self.root / "runtime30_eval/one.txt").write_bytes(b"trimmed")
        with self.assertRaises(ValueError):
            read_page_results(self.root, "30", inputs)

    def test_cli_removes_exactly_one_transport_lf(self):
        body = self.root / "body.txt"
        stdout = self.root / "stdout"
        body.write_bytes(b" \r\n\n")
        stdout.write_bytes(body.read_bytes() + b"\n")
        self.assertEqual(read_cli_result(stdout, body), " \r\n\n")
        stdout.write_bytes(body.read_bytes().strip() + b"\n")
        with self.assertRaises(ValueError):
            read_cli_result(stdout, body)

    def test_partial_job_and_wrong_bundle_cannot_enter_aggregate(self):
        meta = {"kind": "page30", "complete": True, "bundle_sha256": "a" * 64,
                "physical_inputs": 1, "commands": [{"exit_code": 0}]}
        check_job(meta, "a" * 64, "page30", 1)
        for changes in [{"complete": False}, {"bundle_sha256": "b" * 64},
                        {"physical_inputs": 2}, {"commands": [{"exit_code": 1}]}]:
            with self.assertRaises(ValueError):
                check_job(meta | changes, "a" * 64, "page30", 1)

    def test_actual_command_must_use_the_declared_model_and_executable(self):
        path = self.root / "command.json"
        expected = ["candidate-page", "--parseq-model", "tiny30.onnx"]
        path.write_text(json.dumps({"argv": expected, "exit_code": 0}))
        verify_command(path, expected)
        path.write_text(json.dumps({"argv": ["baseline-page", "--parseq-model", "tiny50.onnx"], "exit_code": 0}))
        with self.assertRaises(ValueError):
            verify_command(path, expected)

    def test_complete_model_free_fixture_replays_all_routes_and_both_gt_aliases(self):
        self.bundle()
        bundle = Bundle(self.root)
        artifacts = self.root / "artifacts"
        for kind in ["page30", "page50", "page100", "cli"]:
            folder = artifacts / kind
            (folder / "commands").mkdir(parents=True)
            original_work = self.root / "original-runner" / kind / "work"
            manifest = {"inputs": [{"id": "one", "image": str(original_work / "images/image.png")}]}
            (folder / "NN-images-only.json").write_text(json.dumps(manifest))
            meta = {"kind": kind, "complete": True, "bundle_sha256": digest((self.root / "bundle.json").read_bytes()),
                    "physical_inputs": 1, "assets": bundle.index["assets"], "models_dir": str(self.root / "original-models"),
                    "all_frozen_materials_unchanged": True,
                    "NN_manifest_sha256": digest((folder / "NN-images-only.json").read_bytes()), "builds": {},
                    "commands": [{"exit_code": 0}] * 2}
            for variant in ["baseline", "candidate"]:
                source_manifest = str(original_work / variant / "Cargo.toml")
                events = self.artifacts()
                for event in events:
                    event["manifest_path"] = source_manifest
                log = folder / "commands" / (variant + "-build.stdout")
                log.write_text("\n".join(json.dumps(x) for x in events))
                meta["builds"][variant] = {"source_manifest": source_manifest,
                    "build_stdout_sha256": digest(log.read_bytes()), "source_files": bundle.index["source_files"][variant],
                    "artifacts": check_build_artifacts(events, source_manifest),
                    "binaries": {"page": {"sha256": ("a" if variant == "baseline" else "c") * 64,
                                          "path": str(original_work / "binaries" / (variant + "-page"))},
                                 "cli": {"sha256": "b" * 64, "path": str(original_work / "binaries" / (variant + "-cli"))}}}
                assets = bundle.index["assets"]
                models = Path(meta["models_dir"])
                original_evidence = original_work.parent / "evidence"
                if kind.startswith("page"):
                    for mode in ["eval", "default"]:
                        directory = folder / "page" / variant / ("runtime" + kind[4:] + "_" + mode)
                        directory.mkdir(parents=True)
                        text = "wrong" if variant == "baseline" else " 本文　"
                        (directory / "one.txt").write_bytes(text.encode())
                        (directory / "results.json").write_text(json.dumps([{"id": "one", "page_index": 0, "text": text,
                            "lines": [{"bbox": [0, 0, 1, 1], "text": text, "confidence": 0.95}]}]))
                    model = kind[4:]
                    argv = [str(original_work / "binaries" / (variant + "-page")), "--manifest", str(original_work / "NN-images-only.json"),
                            "--det-model", str(models / assets["deim"]["name"]), "--parseq-model", str(models / assets[model]["name"]),
                            "--charset", str(models / assets["charset"]["name"]), "--label", "runtime" + model,
                            "--output-dir", str(original_evidence / "page" / variant)]
                    receipt = {"argv": argv, "exit_code": 0}
                    (folder / "commands" / (variant + "-page" + model + ".json")).write_text(json.dumps(receipt))
                    meta["commands"].append(receipt)
                else:
                    for mode in ["deim", "bands"]:
                        directory = folder / "cli" / variant / mode
                        directory.mkdir(parents=True)
                        (directory / "one.txt").write_bytes(b"wrong")
                        (folder / "commands" / (variant + "-cli-" + mode + "-one.stdout")).write_bytes(b"wrong\n")
                        receipt = {"argv": cli_command(original_work / "binaries" / (variant + "-cli"), original_work / "images/image.png",
                                   models, assets, mode, original_evidence / "cli" / variant / mode / "one.txt"), "exit_code": 0}
                        (folder / "commands" / (variant + "-cli-" + mode + "-one.json")).write_text(json.dumps(receipt))
                        meta["commands"].append(receipt)
            if kind.startswith("page"):
                meta["Page_API_results"] = 4
            else:
                meta["CLI_processes"] = 4
            (folder / "job.json").write_text(json.dumps(meta))
        output = self.root / "verified"
        verify_matrix(bundle, artifacts, output)
        summary = json.loads((output / "summary.json").read_text())
        self.assertTrue(summary["matrix_gate_pass"])
        self.assertEqual(summary["independently_verified_CSV_rows"], 96)
        self.assertEqual(summary["char_and_word_comparisons"], 96)
        self.assertEqual(summary["new_NN_calls_this_audit"], 0)


if __name__ == "__main__":
    unittest.main()
