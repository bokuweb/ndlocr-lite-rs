"""Complete, deterministic partitioning for frozen Page/CLI matrix retries."""


def partition_inputs(inputs, index=None, total=None):
    if index is None and total is None:
        return list(inputs)
    if (type(index) is not int or type(total) is not int
            or not 1 <= total <= len(inputs) or not 0 <= index < total):
        raise ValueError('invalid shard')
    return list(inputs[index::total])


def validate_coverage(metas, inputs):
    groups = {}
    for meta in metas:
        if meta.get('complete') is not True or meta.get('kind') not in ('page30', 'page50', 'page100', 'cli'):
            raise ValueError('partial or unknown job')
        shard = meta.get('shard')
        selected = partition_inputs(inputs, shard['index'], shard['total']) if shard is not None else inputs
        if meta.get('physical_inputs') != len(selected):
            raise ValueError('job input count mismatch')
        if shard is not None and (meta.get('global_physical_inputs') != len(inputs)
                                  or shard.get('physical_ids') != [row['id'] for row in selected]):
            raise ValueError('shard input binding mismatch')
        groups.setdefault(meta['kind'], []).append(meta)
    for group in groups.values():
        sharded = [meta for meta in group if meta.get('shard') is not None]
        if not sharded:
            if len(group) != 1:
                raise ValueError('duplicate unsharded job')
            continue
        totals = {meta['shard']['total'] for meta in sharded}
        if len(sharded) != len(group) or len(totals) != 1:
            raise ValueError('mixed job partitions')
        total = totals.pop()
        indices = [meta['shard']['index'] for meta in sharded]
        if len(indices) != total or set(indices) != set(range(total)):
            raise ValueError('missing or duplicate shard')
        group.sort(key=lambda meta: meta['shard']['index'])
    return groups
