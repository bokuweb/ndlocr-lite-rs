"""Audit complete-context candidates; no truth-based candidate selection."""
import argparse
import json
from pathlib import Path


def summarize(probe):
    conditions = {
        "all_full": lambda row: True,
        "full_native_high": lambda row: row["fits_native"] and row["candidate"]["raw_confidence"] >= 0.9,
        "full_native_relative": lambda row: row["fits_native"] and row["dominates_baseline_confidence"],
    }
    result = {}
    for name, predicate in conditions.items():
        rows = [row for row in probe["rows"] if row["metrics"] is not None and predicate(row)]
        regressions = []
        for row in rows:
            for normalization, distance in row["metrics"].items():
                before = distance["before_char_errors"]
                after = distance["candidate_char_errors"]
                if after > before:
                    regressions.append({
                        "mode": row["mode"], "id": row["id"],
                        "kind": row["candidate"]["kind"], "normalization": normalization,
                        "before": before, "candidate": after,
                        "confidence": row["candidate"]["raw_confidence"],
                    })
        result[name] = {"candidates": len(rows), "char_regressions": regressions}
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = summarize(json.loads(args.input.read_text()))
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print({name: (row["candidates"], len(row["char_regressions"])) for name, row in result.items()})
