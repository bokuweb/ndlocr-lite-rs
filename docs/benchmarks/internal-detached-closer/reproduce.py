#!/usr/bin/env python3
"""Restore fixed internal-closer evidence and reproduce comparisons without ONNX."""
from pathlib import Path
import argparse
import csv
import hashlib
import importlib.util
import json
import struct
import sys
import unicodedata


def sha(b):
    return hashlib.sha256(b).hexdigest()


def read(root, path):
    return json.loads((root / path).read_text())


def load_parent(packet, root):
    refs = read(packet, 'parents.json')
    directory = packet.parent / refs['packet']
    script = directory / 'reproduce.py'
    assert sha(script.read_bytes()) == refs['reproducer_sha256']
    assert sha((directory / 'parents.json').read_bytes()) == refs['nested_parents_json_sha256']
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('verified_detached_parent', script)
    parent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parent)
    bridge, nested = parent.load_parent(directory, root / 'parent')
    names = ['initial-images', 'initial-inputs', 'positive-images', 'positive-inputs', 'diagnostics', 'text']
    hashes = parent.restore(directory, root / 'parent', names)
    assert hashes == refs['archives']
    return parent, bridge, hashes, nested


def verify_pixels(root):
    from PIL import Image, ImageChops
    captures, noise, commands = 0, 0, 0
    for folder, count in [('fresh', 5088), ('horizontal', 56)]:
        data = root / 'data' / folder
        m = read(data, 'evaluation-manifest.json')
        rows = m['inputs']
        assert len(rows) == count and len({x['id'] for x in rows}) == count
        def resolve(path):
            p = Path(path)
            assert p.is_relative_to('tmp/internal-detached')
            return root / 'data' / p.relative_to('tmp/internal-detached')
        atlas = Image.open(data / 'atlas.png').convert('RGB')
        assert sha((data / 'atlas.png').read_bytes()) == m['atlas_sha256']
        assert sha((data / 'lines.json').read_bytes()) == m['lines_sha256']
        for x in rows:
            for key in ['rgb', 'effective_rgb', 'truth', 'image', 'original']:
                assert sha(resolve(x[key]).read_bytes()) == x[key + '_sha256']
            original = Image.open(resolve(x['image'])).convert('RGB')
            assert original.tobytes() == resolve(x['rgb']).read_bytes()
            assert Image.open(resolve(x['original'])).convert('RGB').tobytes() == original.tobytes()
            effective = resolve(x['effective_rgb']).read_bytes()
            assert atlas.crop(x['bounds']).tobytes() == effective
            if folder == 'fresh':
                rotated = 5 * x['height'] > 4 * x['width']
                assert rotated == x['rotated_for_auto']
                transformed = original.transpose(Image.Transpose.ROTATE_90) if rotated else original
                assert transformed.tobytes() == effective
                if x['kind'] == 'noise' and x['format'] == 'png':
                    stem = f'i_t{x["body_id"]}_{x["font"]}_{x["size"]}_p{x["palt"]}_bare'
                    full = Image.open(data / 'native' / (stem + '.png')).convert('RGB')
                    box = ImageChops.difference(full, Image.new('RGB', full.size, 'white')).getbbox()
                    body = full.crop((box[0] - 2, box[1] - 2, box[2] + 2, box[3] + 2))
                    ink = ImageChops.difference(body, Image.new('RGB', body.size, 'white')).getbbox()
                    white = [xx for xx in range(ink[0] + 1, ink[2] - 1) if all(body.getpixel((xx, yy)) == (255, 255, 255) for yy in range(body.height))]
                    cut = min(white, key=lambda xx: (abs(2 * xx + 1 - ink[0] - ink[2]), xx)) if white else (ink[0] + ink[2]) // 2
                    assert (cut, 'white' if white else 'bbox-center') == (x['cut'], x['cut_kind'])
                    sx, sy, ex, ey = x['shape_bounds']
                    shift = ex - sx + 2 * x['gap']
                    assert shift == x['shift']
                    expected = Image.new('RGB', (body.width + shift, body.height + 2 * x['paper']), 'white')
                    for yy in range(body.height):
                        for xx in range(body.width):
                            dx, dy = xx + (shift if xx >= cut else 0), yy + x['paper']
                            v = body.getpixel((xx, yy))
                            assert not (v != (255, 255, 255) and sx <= dx < ex and sy <= dy < ey)
                            expected.putpixel((dx, dy), v)
                    if ex > sx:
                        expected.paste((0, 0, 0), (sx, sy, ex, ey))
                    assert expected.tobytes() == original.tobytes()
                    noise += 1
            else:
                assert x['requested_orientation'] == 'horizontal' and x['original_auto_would_rotate']
                assert effective == original.tobytes()
        cap = read(data, 'whole-rgb-capture/inputs.json')
        assert cap['model_input_dimensions'] == [256, 24] and len(cap['chunks']) == count
        ids = {x['id']: x for x in rows}
        assert {c['line_id'] for c in cap['chunks']} == set(ids)
        for c in cap['chunks']:
            x = ids[c['line_id']]
            im = Image.open(data / 'whole-rgb-capture' / c['image']).convert('RGB')
            assert list(im.size) == c['dimensions'] == [x['forward_width'], x['forward_height']]
            assert c['chunk_index'] == 0 and c['x_range'] == [0, x['forward_width']]
            assert sha(im.tobytes()) == x['effective_rgb_sha256']
            captures += 1
        if folder == 'fresh':
            assert len(m['native_commands']) == 1104
            for command in m['native_commands']:
                native = data / 'native' / command['id']
                tb = native.with_suffix('.json').read_bytes()
                assert sha(tb) == command['trace_sha256']
                assert sha(native.with_suffix('.png').read_bytes()) == command['png_sha256']
                trace = json.loads(tb)
                normalize = lambda p: unicodedata.normalize('NFC', p)
                assert normalize(trace['font_file']) == normalize(command['argv'][1])
                for run in trace['runs']:
                    assert normalize(run['font_file']) == normalize(trace['font_file'])
                    assert run['font'] == trace['font'] and 0 not in run['glyphs']
                assert sorted(i for run in trace['runs'] for i in run['string_indices']) == list(range(len(command['argv'][4])))
                commands += 1
            proof = read(data, 'tensor-proof.json')
            assert proof == {'images': 5088, 'model_shapes': 3, 'all_original_auto_and_effective_horizontal_tensors_bit_equal': True}
            portable = {**m, 'inputs': [{**x, **{k: str(resolve(x[k])) for k in ['rgb', 'effective_rgb']}} for x in rows]}
            harness = root / 'tensor' / 'fresh'
            harness.mkdir(parents=True)
            (harness / 'evaluation-manifest.json').write_text(json.dumps(portable, ensure_ascii=False, indent=2) + '\n')
    return captures, noise, commands


def verify_page_sources(root):
    current = root / 'diagnostics/verification-source'
    prior = root / 'parent/diagnostics/initial/verification-source'
    for path in ['src/app.rs', 'src/infer/page_pool.rs', 'src/infer/parseq.rs', 'src/pipeline/run_page.rs']:
        assert (current / path).read_bytes() == (prior / path).read_bytes()
    block = b'\n#[cfg(test)]\n#[path = "../../tests/support/internal_detached_model.rs"]\nmod internal_detached_model_tests;\n'
    b = (current / 'src/infer/cached.rs').read_bytes()
    assert block in b and b.replace(block, b'') == (prior / 'src/infer/cached.rs').read_bytes()
    anchor = b'#[cfg(any(feature = "onnx", test))]\npub(crate) fn sanitize_visible_repetitions'
    a = (current / 'src/infer/visible_repetitions.rs').read_bytes()
    b = (prior / 'src/infer/visible_repetitions.rs').read_bytes()
    assert a[a.index(anchor):] == b[b.index(anchor):]


def main():
    from PIL import Image
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    root = parser.parse_args().output.resolve()
    root.mkdir(parents=True)
    packet = Path(__file__).resolve().parent
    parent, bridge, parent_hashes, nested = load_parent(packet, root)
    hashes = parent.restore(packet, root, ['images', 'inputs', 'diagnostics', 'text'])
    cases = read(root, 'diagnostics/portable-cases.json')['cases']
    assert len(cases) == 115023 and sum(x['selected'] for x in cases) == 136
    refs = {(x['corpus'], x['model'], x['id']): x for x in cases}
    assert len(refs) == len(cases)
    for x in read(root, 'parent/diagnostics/portable-cases.json')['cases']:
        y = refs[x['corpus'], x['model'], x['id']]
        assert (x['raw'], x['truth'], x['expected_after'], x['rgb_sha256']) == (y['raw'], y['truth'], y['expected_before'], y['rgb_sha256'])
    seen = set()
    for x in cases:
        assert x['model'] in ['30', '50', '100'] and all(c.isascii() and (c.isalnum() or c in '_-') for c in x['id'])
        if x['rgb'] not in seen:
            assert sha((root / x['rgb']).read_bytes()) == x['rgb_sha256']
            seen.add(x['rgb'])
        truth = root / 'truth' / x['corpus'] / (x['id'] + '.txt')
        truth.parent.mkdir(parents=True, exist_ok=True)
        if truth.exists():
            assert truth.read_bytes() == x['truth'].encode()
        truth.write_bytes(x['truth'].encode())
        for phase, field in [('baseline', 'expected_before'), ('candidate', 'expected_after')]:
            p = root / 'generated/core' / x['corpus'] / x['model'] / phase / (x['id'] + '.txt')
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(x[field].encode())
    tables = 0
    for corpora, directory, audit in [
        (['old', 'spaces', 'fresh', 'horizontal', 'new', 'positive', 'positive-horizontal'], 'text/scores', 'diagnostics/audit.json'),
        (['internal'], 'text/fresh-policy/scores', 'diagnostics/fresh-policy/audit.json'),
        (['internal-horizontal'], 'text/horizontal-policy/scores', 'diagnostics/horizontal-policy/audit.json'),
    ]:
        tables += parent.reproduce_audit(bridge, root, cases, corpora, directory, audit)
    assert tables == 162
    means = {}
    old = read(root, 'parent/parent/diagnostics/parents/old-probe.json')
    for model in old['models']:
        for x in model['rows']:
            means['old', model['label'].removeprefix('single'), x['id']] = struct.unpack('<I', struct.pack('<f', x['mean_confidence']))[0]
    forwards = 0
    for folder, corpus, count in [('fresh', 'internal', 5088), ('horizontal', 'internal-horizontal', 56)]:
        inputs = {x['id']: x for x in read(root, 'data/' + folder + '/evaluation-manifest.json')['inputs']}
        for model in ['30', '50', '100']:
            rows = read(root, 'diagnostics/' + folder + '/' + model + '-export.json')
            assert len(rows) == count and {x['id'] for x in rows} == set(inputs)
            for x in rows:
                key = corpus, model, x['id']
                assert x['text'] == refs[key]['raw'] and x['bounds'] == inputs[x['id']]['bounds']
                assert len(x['chunks']) == 1 and x['chunks'][0]['x_range'] == [0, inputs[x['id']]['forward_width']]
                assert not x['chunks'][0]['inserted_space_before']
                means[key] = bridge.emitted_mean_bits(x)
                forwards += 1
    assert forwards == 15432
    native = read(root, 'diagnostics/native-manifest.json')['cases']
    assert len(native) == 2176
    pins = {(x['corpus'], x['model'], x['id']): x for x in native}
    seen = set()
    for model in ['30', '50', '100']:
        for x in read(root, 'diagnostics/native' + model + '-results.json'):
            key = x['corpus'], model, x['id']
            assert key not in seen and x['raw'] == refs[key]['raw'] and x['text'] == refs[key]['expected_after']
            assert x['mean_bits'] == pins[key]['mean_bits'] == means[key]
            seen.add(key)
    assert seen == set(pins)
    cli = read(root, 'diagnostics/line-cli-results.json')
    inputs = {(x['corpus'], x['model'], x['id']): x for x in read(root, 'diagnostics/line-cli-command.json')['cases']}
    assert len(cli) == 712 and sum(x['selected'] for x in cli) == 136
    for x in cli:
        key = x['corpus'], x['model'], x['id']
        assert x['exit_code'] == 0 and x['text'] == refs[key]['expected_after']
        name = x['model'] + '-' + x['corpus'] + '-' + x['id']
        stdout = (root / 'text/line-cli' / (name + '.stdout')).read_bytes()
        assert stdout[-1:] == b'\n' and stdout[:-1].decode() == x['text']
        im = root / 'text/line-cli/images' / (x['corpus'] + '-' + x['id'] + '.png')
        assert sha(im.read_bytes()) == inputs[key]['image_sha256']
        assert Image.open(im).convert('RGB').tobytes() == (root / refs[key]['rgb']).read_bytes()
    captures, noise, commands = verify_pixels(root)
    assert (captures, noise, commands) == (5144, 384, 1104)
    verify_page_sources(root)
    portable = [x | {'rgb': str(root / x['rgb'])} for x in cases]
    (root / 'product-manifest.json').write_text(json.dumps({'cases': portable}, ensure_ascii=False, indent=2) + '\n')
    result = {'parent_archives': parent_hashes, 'nested_parent_archives': nested, 'archive_sha256': hashes, 'table_files_byte_equal': tables, 'audits_recomputed_equal': 3, 'product_pairs': len(cases), 'new_whole_roi_nn_forwards': forwards, 'native_pairs': len(native), 'cli_processes': len(cli), 'actual_rgb_captures_verified': captures, 'pre_jpeg_noise_pairs_verified': noise, 'native_renderer_commands_verified': commands, 'page_fragment_source_proof_verified': True, 'new_page_nn_runs': 0, 'char_and_word_regressions': 0}
    (root / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
