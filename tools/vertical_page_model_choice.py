"""Classify saved Page observations; never selects an inference result."""
import math
from verify_low_stop_evidence import metric_values


def _portrait_line(record):
    lines = record.get('lines', [])
    if len(lines) != 1 or record.get('text') != lines[0].get('text'):
        return None
    line = lines[0]
    bounds = line.get('bbox', [])
    if len(bounds) != 4 or any(type(value) is not int for value in bounds):
        return None
    x0, y0, x1, y1 = bounds
    if not (0 <= x0 < x1 and 0 <= y0 < y1 and x1 - x0 >= 12
            and y1 - y0 >= 8 * (x1 - x0)):
        return None
    return line


def classify_model_choice(smaller, model100, truths, raw_limit):
    line = _portrait_line(smaller)
    if (line is None or not truths or any(not isinstance(text, str) or not text for text in truths)
            or type(raw_limit) is not int or raw_limit <= 0):
        return None
    text = line['text']
    confidence = line.get('confidence')
    if (not isinstance(text, str) or len(text) <= raw_limit
            or type(confidence) not in (int, float) or not math.isfinite(confidence)
            or not .9 <= confidence <= 1.0):
        return None
    other_text = model100['text']
    lower_metrics = [metric_values(truth, text, 'none') for truth in truths]
    other_metrics = [metric_values(truth, other_text, 'none') for truth in truths]
    if any(a[0] >= b[0] or a[2] > b[2] for a, b in zip(lower_metrics, other_metrics)):
        return None
    other = _portrait_line(model100)
    same_crop = other is not None and other['bbox'] == line['bbox']
    category = ('shorter_same_crop' if len(other_text) < len(text) else 'different_text_same_crop') if same_crop else 'different_or_absent_crop'
    return {'category':category, 'smaller_text':text, 'model100_text':other_text,
            'smaller_bounds':line['bbox'], 'model100_bounds':other['bbox'] if other else None,
            'smaller_confidence':confidence, 'raw_limit':raw_limit,
            'lower_metrics':lower_metrics, 'model100_metrics':other_metrics,
            'model100_is_complete_prefix':bool(same_crop and other_text and text.startswith(other_text)),
            'raw_limit_proxy_only':True, 'actual_CLI_observed':False}


def diagnose(bundle, artifact_root, output, metadata_index, models=None):
    """Replay complete saved Page jobs; no model runtime is imported or called."""
    import json
    from pathlib import Path
    from collections import Counter
    import vertical_page_matrix as matrix
    from vertical_page_shards import partition_inputs, validate_coverage
    from verify_low_stop_evidence import Packet

    inputs = bundle.index['physical_inputs']
    folders = [artifact_root / ('remote-page' + label + '-full-v1') for label in ['30', '50']]
    folders += [artifact_root / f'remote-page100-shard{i}-v1' for i in range(4)]
    paths = []
    for folder in folders:
        found = list(folder.rglob('job.json'))
        if len(found) != 1:
            raise ValueError('missing or duplicate complete Page job')
        paths.append(found[0])
    metas = [json.loads(path.read_text()) for path in paths]
    if set(validate_coverage(metas, inputs)) != {'page30', 'page50', 'page100'}:
        raise ValueError('incomplete Page model coverage')
    observations, bindings = {}, []
    for artifact, path, meta in zip(folders, paths, metas):
        shard = meta.get('shard')
        selected = partition_inputs(inputs, shard['index'], shard['total']) if shard else inputs
        kind, folder = matrix.verify_job_artifact(bundle, path, selected)
        if shard:
            plans = list(artifact.rglob('shard-plan.json'))
            expected = {'kind':kind, 'shard':shard, 'global_physical_inputs':len(inputs),
                        'bundle_sha256':meta['bundle_sha256']}
            if (len(plans) != 1 or matrix.sha(plans[0].read_bytes()) != meta['shard_plan_sha256']
                    or json.loads(plans[0].read_bytes()) != expected):
                raise ValueError('mismatched pre-NN shard plan')
        bindings.append({'kind':kind, 'shard':shard, 'job_sha256':matrix.sha(path.read_bytes())})
        for variant in ['baseline', 'candidate']:
            for route, rows in matrix.read_page_results(folder / 'page' / variant, kind[4:], selected).items():
                merged = observations.setdefault((variant, route), {})
                if not set(merged).isdisjoint(rows):
                    raise ValueError('duplicate Page predictions')
                merged.update(rows)
    if (len(observations) != 12
            or any(set(rows) != {row['id'] for row in inputs} for rows in observations.values())):
        raise ValueError('missing or extra Page predictions')
    packet = Packet(metadata_index)
    member = 'tmp/vertical-output-capacity/metadata/results.json'
    metadata = packet.json(member)
    limits = {row['label']:row['outputs'][0]['max_raw_scalars_from_static_shape'] for row in metadata}
    if set(limits) != {'30', '50', '100'}:
        raise ValueError('missing model metadata')
    for row in metadata:
        if (row['outputs'][0]['dimensions'][1] != limits[row['label']]
                or Path(row['path']).name != bundle.index['assets'][row['label']]['name']):
            raise ValueError('model metadata mismatch')
    assets = {key:asset['sha256'] for key, asset in bundle.index['assets'].items()}
    if models is not None:
        for key, asset in bundle.index['assets'].items():
            if matrix.sha((models / asset['name']).read_bytes()) != assets[key]:
                raise ValueError('live model or charset changed')
    truths = {row['id']:[] for row in inputs}
    cohorts = {row['physical_id']:row['cohort'] for row in bundle.index['scoring_inputs']}
    for row in bundle.index['scoring_inputs']:
        truths[row['physical_id']].append((row['id'], bundle.read('data.zip', row['truth']).decode()))
    cases = []
    for variant in ['baseline', 'candidate']:
        for setting in ['eval', 'default']:
            for model in ['30', '50']:
                for row in inputs:
                    ident = row['id']
                    aliases = truths[ident]
                    result = classify_model_choice(observations[variant, model + '_' + setting][ident],
                                                   observations[variant, '100_' + setting][ident],
                                                   [text for _, text in aliases], limits[model])
                    if result:
                        cases.append({'variant':variant, 'model':model, 'setting':setting, 'id':ident,
                                      'cohort':cohorts[ident], 'scoring_ids':[name for name, _ in aliases], **result})
    identifiers = {row['id'] for row in cases}
    summary = {'candidate_comparisons':len(inputs) * 2 * 2 * 2, 'cases':len(cases),
               'by_variant_category':dict(Counter(row['variant'] + '/' + row['category'] for row in cases)),
               'complete_prefix_cases':sum(row['model100_is_complete_prefix'] for row in cases),
               'physical_input_IDs':len(identifiers),
               'unique_RGB':len({(row['width'], row['height'], row['rgb_sha256']) for row in inputs if row['id'] in identifiers}),
               'new_NN_calls':0, 'model_loads':0, 'actual_CLI_trace':False,
               'runtime_policy_changed':False, 'adoption_gate_pass':False,
               'scope':'saved Linux Page only;bothOandisolatedFVB;GTusedfordiagnosisonly;raw-limitproxyisnotdirectH/Otrace'}
    plan = {'bundle_sha256':matrix.sha((bundle.folder / 'bundle.json').read_bytes()),
            'model_assets_SHA':assets, 'live_model_files_checked':models is not None,
            'metadata_packet_index_SHA':matrix.sha(metadata_index.read_bytes()),
            'metadata_member_SHA':matrix.sha(packet.read(member)), 'raw_limits':limits,
            'classifier_SHA':matrix.sha(Path(__file__).read_bytes()), 'parent_job_bindings':bindings,
            'new_NN_calls':0, 'model_loads':0}
    output.mkdir(parents=True, exist_ok=False)
    for name, value in [('before-classification', plan), ('cases', cases), ('summary', summary)]:
        (output / (name + '.json')).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    return summary


def main():
    import argparse
    import json
    from pathlib import Path
    import vertical_page_matrix as matrix
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, default=Path('docs/benchmarks/vertical-page-fallback'))
    parser.add_argument('--metadata-index', type=Path, default=Path('docs/benchmarks/vertical-low-stop-development/index.json'))
    parser.add_argument('--models', type=Path)
    args = parser.parse_args()
    print(json.dumps(diagnose(matrix.Bundle(args.bundle), args.artifacts, args.output,
                              args.metadata_index, args.models), ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
