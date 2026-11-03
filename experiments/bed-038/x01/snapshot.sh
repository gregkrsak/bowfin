#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1 PYTHONNOUSERSITE=1
export RUSTUP_AUTO_INSTALL=0 RUSTUP_SKIP_UPDATE_CHECK=1
export TMPDIR=/mnt/b38-x01/tmp
mkdir -p /mnt/b38-x01/{out,tmp}
python3 - <<'PY'
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess

out = Path('/mnt/b38-x01/out')
root = Path('/sys/fs/cgroup') / next(x.split(':', 2)[2].strip().lstrip('/') for x in Path('/proc/self/cgroup').read_text().splitlines() if x.startswith('0:'))
data = {x: (root / x).read_text().strip() for x in ['cpu.max', 'cpuset.cpus.effective', 'memory.max', 'memory.swap.max']}
data['affinity'] = sorted(os.sched_getaffinity(0))
data['mountinfo'] = Path('/proc/self/mountinfo').read_text()
(out / 'enforcement.json').write_text(json.dumps(data, indent=2) + '\n')
quota, period = map(int, data['cpu.max'].split())
assert quota / period <= 2 and len(data['affinity']) == 2
assert int(data['memory.max']) == 6 * 1024**3 and data['memory.swap.max'] == '0'
fs = os.statvfs('/mnt/b38-x01')
assert fs.f_blocks * fs.f_frsize <= 10 * 1024**3 - 128 * 1024**2
checks = {}
for directory in [Path('/tmp'), Path.home(), Path.cwd(), Path('/dev/shm')]:
    try:
        (directory / 'b38-x01-must-not-write').write_text('x')
    except OSError as error:
        checks[str(directory)] = {'errno': error.errno, 'read_only': error.errno == 30}
        assert error.errno == 30, f'not verified read-only: {directory}'
    else:
        raise RuntimeError(f'writable outside task scratch: {directory}')
(out / 'confinement.json').write_text(json.dumps(checks, indent=2) + '\n')
tools = {}
for name in ['gcc', 'g++', 'as', 'ld', 'cmake', 'ninja', 'make', 'python3', 'rustup']:
    path = shutil.which(name)
    item = {'absolute_path': path, 'missing': path is None}
    if path:
        resolved = Path(path).resolve()
        digest = hashlib.sha256()
        with resolved.open('rb') as source:
            for block in iter(lambda: source.read(1024 * 1024), b''):
                digest.update(block)
        item.update({'resolved_path': str(resolved), 'sha256': digest.hexdigest()})
        try:
            result = subprocess.run([path, '--version'], capture_output=True, timeout=10, check=False)
            assert len(result.stdout) + len(result.stderr) <= 65536
            item.update({'version_exit': result.returncode, 'stdout': result.stdout.decode(errors='replace'), 'stderr': result.stderr.decode(errors='replace')})
        except subprocess.TimeoutExpired:
            item['version_status'] = 'timed-out-no-retry'
    tools[name] = item
(out / 'installed-tools.json').write_text(json.dumps(tools, indent=2) + '\n')
with (out / 'installed-packages.tsv').open('wb') as target:
    subprocess.run(['/usr/bin/dpkg-query', '--show', '--showformat=${Package}\t${Version}\t${Architecture}\n'], stdout=target, check=True, timeout=15)
assert sum(p.stat().st_size for p in out.iterdir()) <= 16 * 1024**2
(out / 'scope.txt').write_text('Metadata only: no image build, compiler invocation on input, Rust install, ABI probe, loading or source execution. Bootstrap/build closure and launch gates remain unresolved.\n')
PY
