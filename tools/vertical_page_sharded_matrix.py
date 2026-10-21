"""Retry incomplete frozen matrix kinds in complete deterministic shards."""
import json
from pathlib import Path
import vertical_page_matrix as matrix
from vertical_page_shards import partition_inputs, validate_coverage


class InputSubset:
    def __init__(self, bundle, inputs):
        self.original = bundle
        self.index = dict(bundle.index, physical_inputs=inputs)

    def __getattr__(self, name):
        return getattr(self.original, name)


def run_shard(bundle, kind, output, models, index, total):
    if kind not in ('page30', 'page50', 'page100', 'cli'):
        raise ValueError('unknown matrix kind')
    inputs = partition_inputs(bundle.index['physical_inputs'], index, total)
    identity = {'index':index, 'total':total, 'physical_ids':[row['id'] for row in inputs]}
    plan = Path(output).with_name(Path(output).name + '-plan.json')
    if plan.exists():
        raise ValueError('shard plan already exists; preserve the previous attempt')
    matrix.write_json(plan, {'kind':kind, 'shard':identity,
                           'global_physical_inputs':len(bundle.index['physical_inputs']),
                           'bundle_sha256':matrix.sha((bundle.folder / 'bundle.json').read_bytes())})
    try:
        matrix.run_kind(InputSubset(bundle, inputs), kind, output, models)
    finally:
        path = Path(output) / 'evidence/job.json'
        if path.exists():
            meta = json.loads(path.read_text())
            meta['global_physical_inputs'] = len(bundle.index['physical_inputs'])
            meta['shard'] = identity
            meta['shard_plan_sha256'] = matrix.sha(plan.read_bytes())
            matrix.write_json(path, meta)


def verify_shards(bundle, artifacts, output):
    inputs = bundle.index['physical_inputs']
    paths = list(Path(artifacts).rglob('job.json'))
    metas = [json.loads(path.read_text()) for path in paths]
    groups = validate_coverage(metas, inputs)
    if set(groups) != {'page30', 'page50', 'page100', 'cli'}:
        raise ValueError('full eight-route matrix is incomplete')
    plans = {}
    for path in Path(artifacts).rglob('*-plan.json'):
        payload = path.read_bytes()
        digest = matrix.sha(payload)
        if digest in plans:
            raise ValueError('duplicate shard plan')
        plans[digest] = json.loads(payload)
    used_plans = set()
    for meta in metas:
        if meta.get('shard') is None:
            continue
        digest = meta.get('shard_plan_sha256')
        expected = {'kind':meta['kind'], 'shard':meta['shard'],
                    'global_physical_inputs':len(inputs), 'bundle_sha256':meta['bundle_sha256']}
        if digest not in plans or plans[digest] != expected:
            raise ValueError('missing or mismatched pre-NN shard plan')
        used_plans.add(digest)
    if used_plans != set(plans):
        raise ValueError('unreferenced shard plan')
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    work = output / 'work'
    bundle.materialize(work)
    predictions = {}
    counts = {'successful_job_artifacts':len(paths), 'successful_PageRecognizer_loads':0,
              'successful_Page_model_session_loads':0, 'successful_CLI_processes':0}
    for path, meta in zip(paths, metas):
        shard = meta.get('shard')
        selected = partition_inputs(inputs, shard['index'], shard['total']) if shard is not None else inputs
        kind, folder = matrix.verify_job_artifact(bundle, path, selected)
        if kind.startswith('page'):
            if meta.get('PageRecognizer_loads') != 2 or meta.get('model_session_loads') != 4:
                raise ValueError('Page model load count mismatch')
            counts['successful_PageRecognizer_loads'] += meta['PageRecognizer_loads']
            counts['successful_Page_model_session_loads'] += meta['model_session_loads']
        else:
            counts['successful_CLI_processes'] += meta['CLI_processes']
        for variant in ['baseline', 'candidate']:
            for route, values in matrix.read_job_texts(kind, folder, variant, selected).items():
                merged = predictions.setdefault((variant, route), {})
                if set(merged).intersection(values):
                    raise ValueError('duplicate physical prediction')
                merged.update(values)
    matrix.score_predictions(bundle, predictions, output, work, counts)


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parents[1] / 'docs/benchmarks/vertical-page-fallback')
    parser.add_argument('--kind', choices=['page30','page50','page100','cli'])
    parser.add_argument('--index', type=int)
    parser.add_argument('--total', type=int)
    parser.add_argument('--models', type=Path, default=Path('models'))
    parser.add_argument('--artifacts', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.artifacts:
        if args.kind or args.index is not None or args.total is not None:
            parser.error('comparison cannot select only one shard')
    elif args.kind is None or args.index is None or args.total is None:
        parser.error('recognition requires kind, index and total')
    bundle = matrix.Bundle(args.bundle)
    try:
        if args.artifacts:
            verify_shards(bundle,args.artifacts,args.output)
        else:
            run_shard(bundle,args.kind,args.output,args.models,args.index,args.total)
    except Exception as error:
        args.output.mkdir(parents=True, exist_ok=True)
        matrix.write_json(args.output/'failure.json',{'error':repr(error),'full_gate_pass':False})
        raise


if __name__ == '__main__':
    main()
