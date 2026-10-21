"""Verify frozen development observations without executing native binaries or NN."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import string
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "docs/benchmarks/vertical-partition-agreement"))
from fast_distance import distance
import zipfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def safe_name(name):
    return isinstance(name, str) and bool(name) and not PurePosixPath(name).is_absolute() and '..' not in PurePosixPath(name).parts and '\\' not in name


class Packet:
    def __init__(self, index):
        self.index = json.loads(Path(index).read_text())
        self._cache = {}
        value = self.index
        name = value['archive']
        if value['schema'] != 1 or not safe_name(name) or '/' in name:
            raise ValueError('invalid packet archive')
        self.archive = Path(index).parent / name
        data = self.archive.read_bytes()
        if len(data) != value['bytes'] or digest(data) != value['sha256']:
            raise ValueError('archive binding mismatch')
        objects = value['backing_objects']
        logical = value['logical_files']
        if {m['sha256'] for m in logical.values()} != set(objects):
            raise ValueError('missing or unreferenced object')
        for path, meta in logical.items():
            if not safe_name(path) or meta['bytes'] != objects[meta['sha256']]['bytes']:
                raise ValueError('invalid logical binding')
        with zipfile.ZipFile(self.archive) as archive:
            names = archive.namelist()
            wanted = [m['member'] for m in objects.values()]
            if len(set(names)) != len(names) or len(set(wanted)) != len(wanted) or set(names) != set(wanted):
                raise ValueError('archive members do not match objects')
            for key, meta in objects.items():
                if not re.fullmatch('[0-9a-f]{64}', key) or not safe_name(meta['member']):
                    raise ValueError('unsafe object binding')
                payload = archive.read(meta['member'])
                if len(payload) != meta['bytes'] or digest(payload) != key:
                    raise ValueError('object binding mismatch')

    def read(self, name):
        if not safe_name(name) or name not in self.index['logical_files']:
            raise ValueError('unknown logical file')
        meta = self.index['logical_files'][name]
        key = meta['sha256']
        if key not in self._cache:
            with zipfile.ZipFile(self.archive) as archive:
                self._cache[key] = archive.read(self.index['backing_objects'][key]['member'])
        return self._cache[key]

    def json(self, name):
        return json.loads(self.read(name))

    def logical_path(self, path):
        root = self.index['recorded_workspace_root'].rstrip('/') + '/'
        return path[len(root):] if path.startswith(root) else path


def metric_values(truth, predicted, mode):
    def normalized(text):
        if mode == 'none':
            return text
        if mode == 'width':
            return ''.join(chr(ord(c) - 65248) if 65281 <= ord(c) <= 65374 else c for c in text)
        if mode != 'strict':
            raise ValueError('unsupported normalization')
        text = '\n'.join(' '.join(line.split()) for line in text.replace('\r\n', '\n').replace('\r', '\n').split('\n'))
        return ''.join(c for c in text if c not in string.punctuation and c != ' ').strip()
    truth, predicted = normalized(truth), normalized(predicted)
    return (distance(truth, predicted), max(1, len(truth)),
            distance(truth.split(), predicted.split()), max(1, len(truth.split())))


def verify_csv(rows, expected):
    seen = set()
    for row in rows:
        key = (row['system'], row['file'])
        if key in seen or key not in expected:
            raise ValueError('unexpected or duplicate CSV row')
        seen.add(key)
        actual = tuple(int(row[k]) for k in ('char_errors', 'char_total', 'word_errors', 'word_total'))
        if actual != expected[key]:
            raise ValueError('independent CSV metric mismatch')
    if seen != set(expected):
        raise ValueError('incomplete CSV')
    return len(seen)


def cut_glyphs(boxes, boundaries):
    return [(i, boundary) for i, box in enumerate(boxes) if box is not None
            for boundary in boundaries if box[1] < boundary < box[3]]


def verify_native(packet, prefix):
    import csv
    import io
    import math
    import struct
    inputs = packet.json(prefix + '/new-manifest.json')['inputs']
    assert len(inputs) == 72 and len({x['id'] for x in inputs}) == 72
    for row in inputs:
        for key in ('image', 'rgb', 'truth'):
            assert digest(packet.read(packet.logical_path(row[key]))) == row[key + '_sha256']
    routes = ('raw_auto', 'raw_horizontal', 'normal_auto', 'normal_horizontal', 'fragment_auto',
              'fragment_horizontal', 'fragment_batch_auto', 'fragment_exclusive_batch_auto')
    allowed = {'fragment_auto', 'fragment_batch_auto', 'fragment_exclusive_batch_auto'}
    saved = packet.json(prefix + '/native-preflight-evaluation/summary.json')
    commands = packet.json(prefix + '/native-preflight-complete.json')
    assert commands['all_six_commands_exit_zero'] and commands['all_frozen_bytes_unchanged']
    assert len(commands['commands']) == 6 and all(x['exit_code'] == 0 for x in commands['commands'])
    verified = changes = regressions = fresh = 0
    for model in ('30', '50', '100'):
        data = {}
        for variant in ('baseline', 'candidate'):
            folder = prefix + '/native-new72/' + variant + '/' + model
            rows = packet.json(folder + '/results.json')
            assert [x['id'] for x in rows] == [x['id'] for x in inputs]
            assert packet.json(folder + '/summary.json')['API_errors'] == 0
            assert packet.json(folder + '/models-loaded.json')['model_loads'] == 3
            for row in rows:
                assert row['fragment_auto'] == row['fragment_batch_auto'] == row['fragment_exclusive_batch_auto']
                for route in routes:
                    value = row[route]
                    assert set(value) == {'text', 'mean_bits'}
                    mean = struct.unpack('<f', struct.pack('<I', value['mean_bits']))[0]
                    assert math.isfinite(mean) and 0 <= mean <= 1
            data[variant] = {x['id']: x for x in rows}
        for row in inputs:
            for route in routes:
                if data['baseline'][row['id']][route] != data['candidate'][row['id']][route]:
                    assert route in allowed
                    changes += 1
        for mode in ('none', 'width', 'strict'):
            expected = {}
            for row in inputs:
                truth = packet.read(packet.logical_path(row['truth'])).decode()
                for route in routes:
                    for variant in ('baseline', 'candidate'):
                        key = (variant + '_' + route, row['id'] + '.txt')
                        expected[key] = metric_values(truth, data[variant][row['id']][route]['text'], mode)
                    a = expected['baseline_' + route, row['id'] + '.txt']
                    b = expected['candidate_' + route, row['id'] + '.txt']
                    regressions += int(b[0] > a[0] or b[2] > a[2])
                    fresh += int(mode == 'none' and row['cohort'].endswith('_text') and b[0] < a[0] and b[2] <= a[2])
            table = packet.read(prefix + '/native-preflight-evaluation/' + model + '-' + mode + '.csv').decode()
            verified += verify_csv(list(csv.DictReader(io.StringIO(table))), expected)
    assert verified == 10368
    assert changes == len(saved['changes']) and regressions == len(saved['regressions']) and fresh == len(saved['fresh_raw_gain_pairs'])
    assert saved['preflight_pass'] == (not regressions and bool(fresh))
    assert saved['adoption_gate_pass'] is False
    return {'verified_CSV_rows': verified, 'changed_input_routes': changes,
            'regressions': regressions, 'fresh_raw_gain_pairs': fresh, 'decision': saved['decision']}


def verify_geometry(packet, phase):
    import gzip
    import io
    from PIL import Image
    prefix = 'tmp/vertical-low-stop-recovery'
    fingerprints = json.loads(gzip.decompress(packet.read(prefix + '/geometry-' + phase + '-crop-fingerprints.json.gz')))
    archives = {x['logical_archive']: x['members'] for x in fingerprints}
    if phase == 'Q':
        prefix = 'tmp/vertical-unique-insertion'
        inputs = packet.json(prefix + '/geometry-manifest.json')['inputs']
        records = packet.json(prefix + '/geometry-v1/records.json')
        assert len(inputs) == 270 and len(records) == 6480
    else:
        inputs = packet.json(prefix + '/all-native-manifest.json')['inputs']
        records = packet.json(prefix + '/finer-geometry2-v1/records.json')
        assert len(inputs) == 558 and len(records) == 1674
    inputs = {x['id']: x for x in inputs}
    used = set()
    cuts = {'legacy': 0, 'supported': 0}
    verified = 0
    images = {}
    for record in records:
        row = inputs[record['id']]
        if phase == 'S' and not record['eligible']:
            assert record['routes'] == []
            continue
        if phase == 'Q' and 'error' in record:
            continue
        if row['id'] not in images:
            rgb = packet.read(packet.logical_path(row['rgb']))
            assert digest(rgb) == row['rgb_sha256'] and len(rgb) == row['width'] * row['height'] * 3
            png = packet.read(packet.logical_path(row['image']))
            assert digest(png) == row['image_sha256']
            with Image.open(io.BytesIO(png)) as image:
                image = image.convert('RGB')
                assert image.size == (row['width'], row['height']) and image.tobytes() == rgb
                images[row['id']] = image.transpose(Image.Transpose.ROTATE_90)
        rotated = images[row['id']]
        if phase == 'Q':
            routes = [(record['profile'], prefix + '/geometry-v1/' + part['archive'],
                       part['file'], part['x_range'], part) for part in record['chunks']]
        else:
            assert [route['divisor'] for route in record['routes']] == [2, 4]
            routes = [('supported', prefix + '/finer-geometry2-v1/original-crops.zip',
                       part['member'], part['range'], part) for route in record['routes'] for part in route['parts']]
        for profile, archive, member, bounds, part in routes:
            a, b = bounds
            assert 0 <= a < b <= rotated.width and part['space_before'] is False
            assert (part['width'], part['height']) == (b - a, rotated.height)
            key = (archive, member)
            assert key not in used
            used.add(key)
            meta = archives[archive][member]
            rgb = rotated.crop((a, 0, b, rotated.height)).tobytes()
            assert len(rgb) == meta['bytes'] and digest(rgb) == meta['sha256']
            cuts[profile] += len(cut_glyphs(row.get('glyph_boxes', []), (a, b)))
            verified += 1
    assert len(used) == sum(len(members) for members in archives.values())
    assert cuts['supported'] == 0
    assert verified == (26506 if phase == 'Q' else 24147)
    assert cuts['legacy'] == (156 if phase == 'Q' else 0)
    return {'independently_reconstructed_crop_RGB_hashes': verified, 'glyph_cut_events': cuts,
            'geometry_is_not_actual_NN_eligibility': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--index', type=Path, default=Path('docs/benchmarks/vertical-low-stop-development/index.json'))
    args = parser.parse_args()
    packet = Packet(args.index)
    result = {'logical_files':len(packet.index['logical_files']), 'objects':len(packet.index['backing_objects']),
              'all_archive_and_logical_bindings_verified':True, 'new_NN_calls':0}
    result['native'] = {phase: verify_native(packet, prefix) for phase, prefix in
                        [('Q','tmp/vertical-unique-insertion'), ('S','tmp/vertical-low-stop-recovery')]}
    result['geometry'] = {phase: verify_geometry(packet, phase) for phase in ('Q','S')}
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
