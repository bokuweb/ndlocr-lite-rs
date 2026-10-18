"""Diagnose unrepresented ink using frozen raw outputs; never select by truth."""

def uncovered_ink_stats(rgb, width, height, proposal, boxes):
    if width <= 0 or height <= 0 or len(rgb) != width * height * 3:
        raise ValueError('positive dimensions and exact RGB length required')

    def clipped(bounds):
        if len(bounds) != 4 or not all((isinstance(v, int) for v in bounds)):
            raise ValueError('four integer coordinates required')
        (x0, y0, x1, y1) = bounds
        if x1 <= x0 or y1 <= y0:
            raise ValueError('positive half-open rectangle required')
        return (max(0, min(x0, width)), max(0, min(y0, height)), max(0, min(x1, width)), max(0, min(y1, height)))
    (x0, y0, x1, y1) = clipped(proposal)
    covered = bytearray(width * height)
    for bounds in boxes:
        (a, b, c, d) = clipped(bounds)
        (a, b, c, d) = (max(a, x0), max(b, y0), min(c, x1), min(d, y1))
        if a >= c or b >= d:
            continue
        for y in range(b, d):
            covered[y * width + a:y * width + c] = b'\x01' * (c - a)
    foreground = covered_pixels = uncovered = 0
    columns = set()
    (left, top, right, bottom) = (width, height, 0, 0)
    for y in range(y0, y1):
        for x in range(x0, x1):
            offset = (y * width + x) * 3
            if sum(rgb[offset:offset + 3]) >= 660:
                continue
            foreground += 1
            if covered[y * width + x]:
                covered_pixels += 1
            else:
                uncovered += 1
                columns.add(x)
                (left, top, right, bottom) = (min(left, x), min(top, y), max(right, x + 1), max(bottom, y + 1))
    return {'foreground_pixels': foreground, 'covered_pixels': covered_pixels, 'uncovered_pixels': uncovered, 'uncovered_columns': len(columns), 'uncovered_bbox': [left, top, right, bottom] if uncovered else None}

def comparison_metrics(truth, baseline, candidate, full_ink):
    if not full_ink:
        return None
    from eval_ocr_metrics import normalize_text, levenshtein
    result = {}
    for level in ['none', 'width', 'strict']:
        (t, b, c) = [normalize_text(text, level) for text in [truth, baseline, candidate]]
        result[level] = {'char_total': max(1, len(t)), 'before_char_errors': levenshtein(list(t), list(b)), 'candidate_char_errors': levenshtein(list(t), list(c)), 'word_total': max(1, len(t.split())), 'before_word_errors': levenshtein(t.split(), b.split()), 'candidate_word_errors': levenshtein(t.split(), c.split())}
    return result

def candidate_strata(full_ink, native, single_band, uncovered_pixels, score, baseline_mean):
    import math
    if not (full_ink and native and single_band and (uncovered_pixels > 0)):
        return []
    if baseline_mean is None or not all((math.isfinite(x) and 0 < x <= 1 for x in [score, baseline_mean])):
        return []
    result = []
    if score >= 0.9:
        result.append('full_native_single_missing_high')
        if score >= baseline_mean:
            result.append('plus_confidence_dominance')
    if score >= baseline_mean:
        result.append('relative_without_fixed_floor')
    return result

def main():
    import argparse, hashlib, json, math, subprocess, zipfile
    from pathlib import Path
    from PIL import Image
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frozen-root', type=Path, required=True)
    parser.add_argument('--benchmark', type=Path, default=Path('docs/benchmarks/visible-repetitions'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--page-source-commit', required=True)
    parser.add_argument('--verify-model-assets', action='store_true')
    args = parser.parse_args()
    root = args.frozen_root
    sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
    manifest_path = root / 'evaluation-manifest.json'
    artifacts = root / 'diagnostics' if (root / 'diagnostics').is_dir() else root
    probe_path = artifacts / 'visible-probe.json'
    manifest = json.loads(manifest_path.read_text())
    probe = json.loads(probe_path.read_text())
    diagnostic = json.loads((args.benchmark / 'diagnostic-evidence.json').read_text())
    evidence = json.loads((args.benchmark / 'text-evidence.json').read_text())
    assert sha(probe_path) == diagnostic['members']['visible-probe.json']
    assert sha(manifest_path) == diagnostic['members']['evaluation-manifest.json']
    page_commit = subprocess.check_output(['git', 'rev-parse', args.page_source_commit], text=True).strip()
    for (name, digest) in manifest['final_source_sha256'].items():
        assert hashlib.sha256(subprocess.check_output(['git', 'show', page_commit + ':' + name])).hexdigest() == digest, name
    if args.verify_model_assets:
        for (name, digest) in manifest['models'].items():
            assert sha(name) == digest
        assert sha('models/NDLmoji.yaml') == manifest['charset_sha256']
    fixture_root = Path('tests/fixtures/repeated_glyph')
    fixture = json.loads((fixture_root / 'manifest.json').read_text())
    assert sha(fixture_root / 'corpus.zip') == fixture['archive_sha256']
    fixture_rows = {'repeat_' + x['id']: x for x in fixture['rows']}
    with zipfile.ZipFile(fixture_root / 'corpus.zip') as archive:
        fixture_images = {id: archive.read(row['image_member']) for id, row in fixture_rows.items()}
    def png_path(id, candidate):
        member = 'raw/inputs/' + id + '-' + candidate['kind'] + '.png'
        assert Path(candidate['png']).name == Path(member).name
        return member, artifacts / member
    inputs = {x['id']: x for x in manifest['inputs']}
    rows = []
    baselines = {}
    source_sha = {'evaluation-manifest.json': sha(manifest_path), 'visible-probe.json': sha(probe_path)}
    png_sha = {}
    summary = {}
    for model in probe['models']:
        (native_w, native_h) = model['input_dimensions']
        if native_w not in [256, 768]:
            continue
        label = 'runtime30' if native_w == 256 else 'runtime100'
        assert len(model['images']) == 184
        for suffix in ['eval', 'default']:
            mode = label + '_' + suffix
            path = root / 'after' / mode / 'results.json'
            assert sha(path) == evidence['members']['after/' + mode + '/results.json']
            source_sha['after/' + mode + '/results.json'] = sha(path)
            pages = {x['id']: x for x in json.loads(path.read_text())}
            for sample in model['images']:
                id = sample['id']
                source = inputs[id]
                image_bytes = fixture_images[id] if source['cohort'] == 'repeated_glyph' else Path(source['image']).read_bytes()
                assert hashlib.sha256(image_bytes).hexdigest() == source['image_sha256']
                truth_path = root / 'truth' / (id + '.txt')
                assert sha(truth_path) == source['truth_sha256']
                assert truth_path.read_text() == sample['truth']
                baseline = pages[id]
                assert (root / 'after' / mode / (id + '.txt')).read_text() == baseline['text']
                assert sha(root / 'after' / mode / (id + '.txt')) == manifest['prediction_sha256']['after/' + mode + '/' + id + '.txt']
                baselines[mode + '/' + id] = baseline
                original_input = next(x for x in sample['inputs'] if x['kind'] == 'original')
                original = Image.open(png_path(id, original_input)[1]).convert('RGB')
                rgb = original.tobytes()
                (w, h) = original.size
                bounds = [x['bbox'] for x in baseline['lines']]
                weights = [sum((not c.isspace() for c in x['text'])) for x in baseline['lines']]
                weight = sum(weights)
                mean = sum((n * x['confidence'] for (n, x) in zip(weights, baseline['lines']))) / weight if weight else None
                band_count = sum((x['kind'].startswith('bands-') for x in sample['inputs']))
                for candidate in sample['inputs']:
                    member, png = png_path(id, candidate)
                    digest = sha(png)
                    assert digest == diagnostic['members'][member]
                    png_sha[member] = digest
                    crop = Image.open(png).convert('RGB')
                    assert list(crop.size) == candidate['size']
                    assert original.crop(candidate['bbox']).tobytes() == crop.tobytes()
                    ink = uncovered_ink_stats(rgb, w, h, candidate['bbox'], bounds)
                    (cw, ch) = candidate['size']
                    native = cw * native_h <= ch * native_w
                    score = candidate['raw_confidence']
                    assert math.isfinite(score)
                    dominates = mean is not None and score >= mean
                    metrics = comparison_metrics(sample['truth'], baseline['text'], candidate['visible_text'], candidate['covers_all_220_ink'])
                    entry = {'mode': mode, 'id': id, 'cohort': sample['cohort'], 'truth': sample['truth'], 'baseline_text': baseline['text'], 'baseline_bounds': bounds, 'baseline_weighted_confidence': mean, 'candidate': candidate, 'source_rgb_sha256': hashlib.sha256(rgb).hexdigest(), 'source_image_sha256': source['image_sha256'], 'ink': ink, 'single_band': band_count == 1, 'fits_native': native, 'dominates_baseline_confidence': dominates, 'metrics': metrics}
                    rows.append(entry)
                    for stratum in candidate_strata(metrics is not None, native, band_count == 1, ink['uncovered_pixels'], score, mean):
                        key = mode + '/' + ('original' if candidate['kind'] == 'original' else 'bands') + '/' + stratum
                        stats = summary.setdefault(key, {'candidates': 0, 'char_improved': 0, 'char_equal': 0, 'char_worse': 0, 'word_worse': 0, 'regressions': []})
                        stats['candidates'] += 1
                        for (norm, dist) in metrics.items():
                            (before, after) = (dist['before_char_errors'], dist['candidate_char_errors'])
                            stats['char_improved' if after < before else 'char_worse' if after > before else 'char_equal'] += 1
                            if after > before:
                                stats['regressions'].append({'id': id, 'normalization': norm, 'before': before, 'candidate': after})
                            if dist['candidate_word_errors'] > dist['before_word_errors']:
                                stats['word_worse'] += 1
    assert len(baselines) == 736
    assert len(rows) == 1368
    assert len(png_sha) == 342
    result = {'raw_inference_source_commit': manifest['baseline_commit'], 'page_baseline_source_commit': page_commit, 'source_sha256': source_sha, 'models': manifest['models'], 'charset_sha256': manifest['charset_sha256'], 'png_sha256': png_sha, 'baseline_pairs': len(baselines), 'candidate_rows': len(rows), 'full_ink_candidate_rows': sum((x['metrics'] is not None for x in rows)), 'summary': summary, 'rows': rows}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'baseline_pairs': len(baselines), 'candidate_rows': len(rows), 'summary': summary}, ensure_ascii=False, indent=2))
if __name__ == '__main__':
    main()
