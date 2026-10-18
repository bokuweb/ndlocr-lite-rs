#!/usr/bin/env python3
"""固定位置の文字候補と独立原文から、補正前の回復余地を測る。"""
import argparse
from collections import Counter
import json
from pathlib import Path


def distances(truth, choices):
    """各位置で候補の一文字を選ぶ最小編集距離の表を作る。"""
    table = [list(range(len(choices) + 1))]
    for i, ch in enumerate(truth, 1):
        previous = table[-1]
        current = [i]
        for j, alternatives in enumerate(choices, 1):
            current.append(min(previous[j] + 1, current[-1] + 1,
                               previous[j-1] + (ch not in alternatives)))
        table.append(current)
    return table


def evaluate(truth, row, k):
    """一定の最短経路の候補残存率と、全候補位置のoracle距離を別々に返す。"""
    if not isinstance(k, int) or isinstance(k, bool) or k < 1:
        raise ValueError('k must be a positive integer')
    text, positions = row['text'], row['positions']
    if not isinstance(truth, str) or not isinstance(text, str) or len(text) != len(positions):
        raise ValueError('text and position count mismatch')
    choices = []
    for ch, position in zip(text, positions):
        candidates = position['candidates']
        if (position['character'] != ch or not isinstance(candidates, list) or not candidates
                or candidates[0] != ch
                or any(c is not None and (not isinstance(c, str) or len(c) != 1) for c in candidates)):
            raise ValueError('invalid character or candidate mapping')
        choices.append([c for c in candidates[:k] if c is not None])
    table = distances(truth, [[ch] for ch in text])
    result = dict(errors=table[-1][-1], reference_characters=len(truth), matches=0,
                  substitutions=0, deletions=0, insertions=0,
                  substitutions_outside_top_k=0, correct_with_alternatives=0,
                  oracle_errors=distances(truth, choices)[-1][-1])
    ranks = Counter()
    edits = []
    i, j = len(truth), len(text)
    while i or j:
        if i and j and table[i][j] == table[i-1][j-1] + (truth[i-1] != text[j-1]):
            if truth[i-1] == text[j-1]:
                result['matches'] += 1
                result['correct_with_alternatives'] += any(c != text[j-1] for c in choices[j-1])
            else:
                result['substitutions'] += 1
                candidates = positions[j-1]['candidates'][:k]
                rank = candidates.index(truth[i-1]) + 1 if truth[i-1] in candidates else None
                if rank is None:
                    result['substitutions_outside_top_k'] += 1
                else:
                    ranks[str(rank)] += 1
                edits.append(dict(kind='substitution', truth_index=i-1, text_index=j-1,
                                  expected=truth[i-1], actual=text[j-1], rank=rank))
            i, j = i-1, j-1
        elif i and table[i][j] == table[i-1][j] + 1:
            result['deletions'] += 1
            edits.append(dict(kind='deletion', truth_index=i-1, text_index=j,
                              expected=truth[i-1], actual=None))
            i -= 1
        else:
            result['insertions'] += 1
            edits.append(dict(kind='insertion', truth_index=i, text_index=j-1,
                              expected=None, actual=text[j-1]))
            j -= 1
    result['substitution_ranks'] = dict(sorted(ranks.items(), key=lambda pair: int(pair[0])))
    result['edits'] = list(reversed(edits))
    return result


def unique_rows(rows):
    """重複したIDを無言で上書きせずに拒否する。"""
    result = {}
    for row in rows:
        identifier = row['id']
        if not isinstance(identifier, str) or not identifier or identifier in result:
            raise ValueError('invalid or duplicate row id')
        result[identifier] = row
    return result


def evaluate_manifest(manifest, predictions, ks):
    """原文と予測のID集合を厳密に一致させて集計する。"""
    truth, actual = unique_rows(manifest), unique_rows(predictions)
    if truth.keys() != actual.keys() or not truth:
        raise ValueError('truth and prediction ids must match and be nonempty')
    report = {'lines': len(truth), 'by_k': {}}
    for k in ks:
        rows = [dict(id=identifier, **evaluate(expected['truth'], actual[identifier], k))
                for identifier, expected in truth.items()]
        totals = {name: sum(row[name] for row in rows) for name in rows[0]
                  if name not in ('id', 'edits', 'substitution_ranks')}
        ranks = Counter()
        for row in rows:
            ranks.update(row['substitution_ranks'])
        totals['substitution_ranks'] = dict(sorted(ranks.items(), key=lambda pair: int(pair[0])))
        totals['rows'] = rows
        report['by_k'][str(k)] = totals
    return report


def main():
    """正規化しないJSON評価レポートをファイルへ保存する。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', required=True, type=Path)
    parser.add_argument('--predictions', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--k', nargs='+', type=int, default=[1, 3, 5, 10])
    args = parser.parse_args()
    report = evaluate_manifest(json.loads(args.manifest.read_text(encoding='utf-8')),
                               json.loads(args.predictions.read_text(encoding='utf-8')), args.k)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
