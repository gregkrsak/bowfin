"""Bounded installed-host metadata only; never installs or compiles anything."""
import hashlib
import json
import os
from pathlib import Path
import time

TOOLS = ('gcc', 'g++', 'as', 'ld', 'cmake', 'ninja', 'make', 'python3')


def run(bounded):
    deadline = time.monotonic() + 32
    records = {}
    destination = Path('/task/out/installed-tools.json')
    def persist():
        encoded = json.dumps(records, sort_keys=True).encode()
        assert len(encoded) <= 1024**2, 'METADATA_RECORD_BOUND'
        destination.write_bytes(encoded)
    hashed = 0
    environment = dict(PATH='/usr/bin:/bin', HOME='/task', TMPDIR='/task/tmp',
                       LC_ALL='C', LANG='C', PYTHONDONTWRITEBYTECODE='1')
    for name in TOOLS:
        path = Path('/usr/bin') / name
        item = {'requested_path': str(path), 'missing': not path.exists()}
        records[name] = item
        persist()
        if path.exists():
            real = path.resolve(strict=True)
            size = real.stat().st_size
            assert real.is_file() and size <= 64 * 1024**2, 'METADATA_TOOL_BOUND'
            hashed += size
            assert hashed <= 256 * 1024**2, 'METADATA_HASH_BOUND'
            digest = hashlib.sha256()
            with real.open('rb') as source:
                remaining = size
                while remaining:
                    block = source.read(min(65536, remaining))
                    assert block, 'METADATA_TOOL_SHORT_READ'
                    digest.update(block)
                    remaining -= len(block)
                assert not source.read(1), 'METADATA_TOOL_CHANGED'
            seconds = min(2, deadline-time.monotonic())
            assert seconds > 0, 'METADATA_DEADLINE'
            item.update(resolved_path=str(real), bytes=size, sha256=digest.hexdigest())
            persist()
            try:
                code, output = bounded([str(real), '--version'], seconds, 65536, environment)
            except Exception as error:
                item.update(failure=getattr(error,'code',type(error).__name__),
                            partial=getattr(error,'partial',{}))
                persist()
                raise
            item.update(version_exit=code, version_output=output.decode(errors='replace'))
            persist()
            assert code == 0, 'METADATA_VERSION_STATUS'
        records[name] = item
    # Rust proxies are not invoked, discovered or installed in this amendment.
    records['rust'] = {'status': 'Not invoked; distribution closure remains unresolved'}
    persist()
    seconds = min(5, deadline-time.monotonic())
    assert seconds > 0, 'METADATA_DEADLINE'
    try:
        code, data = bounded(['/usr/bin/dpkg-query', '--show',
            '--showformat=${Package}\t${Version}\t${Architecture}\n'], seconds, 1024**2, environment)
    except Exception as error:
        records['packages']={'failure':getattr(error,'code',type(error).__name__),
                             'partial':getattr(error,'partial',{})}
        persist()
        raise
    Path('/task/out/installed-packages.tsv').write_bytes(data)
    records['packages']={'exit':code,'bytes':len(data)}
    persist()
    assert code == 0, 'METADATA_PACKAGE_STATUS'
    return {'tools': len(TOOLS), 'tool_hash_bytes': hashed, 'package_bytes': len(data),
            'scope': 'Installed host metadata; no candidate selection, Rust or input compilation'}
