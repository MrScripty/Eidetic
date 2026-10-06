#!/usr/bin/env python3
"""Reject Cargo feature unions that permit native ONNX Runtime downloads.

Input is `cargo metadata --locked --all-features --format-version 1`, which
resolves dependencies without running their build scripts.
"""
import json
from pathlib import Path
import sys


def check(metadata):
    packages = {package['id']: package for package in metadata['packages']}
    observed = {}
    for node in metadata['resolve']['nodes']:
        package = packages[node['id']]
        name = package['name']
        if name not in ('ort', 'ort-sys'):
            continue
        features = set(node['features'])
        forbidden = features & {'download-binaries', 'copy-dylibs'}
        if forbidden:
            raise ValueError(f'{name} enables forbidden native SDK features: {sorted(forbidden)}')
        required = 'load-dynamic' if name == 'ort' else 'disable-linking'
        if required not in features:
            raise ValueError(f'{name} must enable {required}')
        if name in observed:
            raise ValueError(f'multiple {name} versions require a separate compatibility audit')
        observed[name] = {'version': package['version'], 'features': sorted(features)}
    if set(observed) != {'ort', 'ort-sys'}:
        raise ValueError('expected the qualified ort and ort-sys dependencies')
    return observed


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Usage: check-onnx-no-download.py <cargo-metadata.json>')
    try:
        result = check(json.loads(Path(sys.argv[1]).read_text(encoding='utf-8')))
    except (KeyError, ValueError) as error:
        raise SystemExit(f'ONNX dependency gate failed: {error}') from error
    print(json.dumps(result, indent=2))
