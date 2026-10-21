"""Verify the completed Page-only Linux packet without models or native binaries."""
import argparse
import csv
import gzip
import json
from pathlib import Path
import sys
import zipfile

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / 'tools'))
import vertical_page_matrix as matrix
from vertical_page_shards import partition_inputs
from vertical_page_model_choice import diagnose
from verify_low_stop_evidence import metric_values


def verify(output):
    folder = Path(__file__).resolve().parent
    index_path = folder / 'completed-pages-linux-v1.json'
    index = json.loads(index_path.read_text())
    member_bytes = (folder / index['members_file']).read_bytes()
    if matrix.sha(member_bytes) != index['members_sha256']:
        raise ValueError('member index changed')
    members = json.loads(gzip.decompress(member_bytes))
    blob = (folder / index['archive']).read_bytes()
    if matrix.sha(blob) != index['sha256'] or len(blob) != index['bytes']:
        raise ValueError('Page archive changed')
    output.mkdir(parents=True, exist_ok=False)
    with zipfile.ZipFile(folder / index['archive']) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)) or set(names) != set(members) or len(names) != index['member_count']:
            raise ValueError('missing, duplicate or extra Page members')
        for name, expected in members.items():
            safe = matrix.safe_member(name)
            data = archive.read(name)
            if len(data) != expected['bytes'] or matrix.sha(data) != expected['sha256']:
                raise ValueError('Page member changed: ' + name)
            target = output / safe
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    bundle = matrix.Bundle(folder)
    if matrix.sha((folder / 'bundle.json').read_bytes()) != index['bundle_sha256']:
        raise ValueError('frozen Page bundle changed')
    replay = output / 'replayed-model-choice'
    diagnostic = diagnose(bundle, output / 'artifacts', replay,
                          REPO / 'docs/benchmarks/vertical-low-stop-development/index.json')
    expected_cases = json.loads((output / 'analysis/saved-model-choice-v1/cases.json').read_text())
    if json.loads((replay / 'cases.json').read_text()) != expected_cases:
        raise ValueError('model-choice classification changed')
    predictions, metas = {}, []
    for path in sorted((output / 'artifacts').rglob('job.json')):
        meta = json.loads(path.read_text())
        metas.append(meta)
        shard = meta.get('shard')
        inputs = bundle.index['physical_inputs']
        selected = partition_inputs(inputs, shard['index'], shard['total']) if shard else inputs
        for variant in ['baseline', 'candidate']:
            for route, values in matrix.read_job_texts(meta['kind'], path.parent, variant, selected).items():
                merged = predictions.setdefault((variant, route), {})
                if not set(merged).isdisjoint(values):
                    raise ValueError('duplicate Page prediction')
                merged.update(values)
    scoring = {row['id'] + '.txt':row for row in bundle.index['scoring_inputs']}
    truths = {name:bundle.read('data.zip', row['truth']).decode() for name, row in scoring.items()}
    verified, regressions = 0, 0
    raw_gains = {}
    for model in ['30', '50', '100']:
        raw_gains[model] = 0
        for mode in ['none', 'width', 'strict']:
            table = output / 'analysis' / ('page' + model) / (mode + '.csv')
            with table.open(newline='') as stream:
                rows = list(csv.DictReader(stream))
            keyed = {(row['system'], row['file']):row for row in rows}
            if len(rows) != len(scoring) * 4 or len(keyed) != len(rows):
                raise ValueError('incomplete or duplicate official CSV rows')
            for row in rows:
                variant, route = row['system'].split('_', 1)
                physical = scoring[row['file']]['physical_id']
                text = predictions[variant, route][physical]
                actual = tuple(int(row[key]) for key in ['char_errors', 'char_total', 'word_errors', 'word_total'])
                if actual != metric_values(truths[row['file']], text, mode):
                    raise ValueError('official/independent CSV mismatch')
                verified += 1
            for name in scoring:
                for route in [model + '_eval', model + '_default']:
                    before, after = [keyed[variant + '_' + route, name] for variant in ['baseline', 'candidate']]
                    a, b = [int(row['char_errors']) for row in [before, after]]
                    c, d = [int(row['word_errors']) for row in [before, after]]
                    regressions += b > a or d > c
                    if mode == 'none' and b < a and d <= c:
                        raw_gains[model] += 1
    if (verified != index['official_CSV_rows_independently_verified'] or regressions != 0
            or raw_gains != index['per_model_raw_gain_pairs']):
        raise ValueError('Page-only metric conclusions changed')
    report = {'valid_Page_jobs':len(metas), 'saved_Page_results':sum(meta['Page_API_results'] for meta in metas),
              'saved_PageRecognizer_loads':sum(meta['PageRecognizer_loads'] for meta in metas),
              'saved_Page_model_session_loads':sum(meta['model_session_loads'] for meta in metas),
              'archive_members_verified':len(members), 'independently_verified_CSV_rows':verified,
              'per_model_raw_gain_pairs':raw_gains, 'per_input_regressions':regressions,
              'replayed_model_choice_observations':diagnostic['cases'],
              'model_choice_physical_input_IDs':diagnostic['physical_input_IDs'],
              'model_choice_unique_RGB':diagnostic['unique_RGB'],
              'new_NN_calls':0, 'new_model_loads':0, 'full_gate_pass':False,
              'scope':'completed Page kinds only;CLI and full gate still required',
              'packet_index_sha256':matrix.sha(index_path.read_bytes())}
    (output / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.output), indent=2))
