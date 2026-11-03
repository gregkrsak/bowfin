#!/usr/bin/env bash
set -euo pipefail
export CARGO_HOME=/mnt/b38-x03/cargo RUSTUP_HOME=/mnt/b38-x03/rustup
export CARGO_TARGET_DIR=/mnt/b38-x03/target TMPDIR=/mnt/b38-x03/tmp
mkdir -p /mnt/b38-x03/{out,tmp}
python3 - <<'PY'
from pathlib import Path
import hashlib,json,os
root=Path('/sys/fs/cgroup')/next(x.split(':',2)[2].strip().lstrip('/') for x in Path('/proc/self/cgroup').read_text().splitlines() if x.startswith('0:'))
data={x:(root/x).read_text().strip() for x in ['cpu.max','cpuset.cpus.effective','memory.max','memory.swap.max']}
data['affinity']=sorted(os.sched_getaffinity(0))
Path('/mnt/b38-x03/out/enforcement.json').write_text(json.dumps(data,indent=2)+'\n')
quota,period=map(int,data['cpu.max'].split())
assert quota/period<=2 and len(data['affinity'])<=2
assert int(data['memory.max'])==4*1024**3 and data['memory.swap.max']=='0'
assert os.statvfs('/mnt/b38-x03').f_blocks*os.statvfs('/mnt/b38-x03').f_frsize<=4*1024**3
try:
    Path('/tmp/b38-x03-must-not-write').write_text('x')
except OSError as e:
    assert e.errno==30
else:
    raise RuntimeError('scratch confinement failed')
for name,n,h in [('computational.bow',947,'1c9a863981ec28467de4f1850ba797a6ad19652fb6613897d94ceae402d4fe42'),('companion-suffix.bow',713,'ba74eceee3661196f85e89d21aebc41c52a009fe6bc876822cb353abfbfaa055')]:
    b=(Path('experiments/bed-038/x03')/name).read_bytes()
    assert len(b)==n and hashlib.sha256(b).hexdigest()==h
b=Path('examples/sentinel.bow').read_bytes()+b'\n'+Path('experiments/bed-038/x03/companion-suffix.bow').read_bytes()
assert len(b)==7866 and hashlib.sha256(b).hexdigest()=='c0c3725889590b60a1df713f4d25a252f0774b1eab4373a7e8256cbb4ca02934'
PY
rustup toolchain install 1.80.0 --profile minimal --no-self-update
rustc +1.80.0 -vV > /mnt/b38-x03/out/rustc.txt
find "$RUSTUP_HOME/toolchains" -type f -print0 | sort -z | xargs -0 sha256sum > /mnt/b38-x03/out/toolchain.sha256
export RUSTFLAGS='--cfg b38_x03 --check-cfg=cfg(b38_x03)'
export B38_X03_OUTPUT=/mnt/b38-x03/out
cargo +1.80.0 test --locked --lib b38_x03_admission_inventory_only -- --exact execution::machine::conformance::b38_x03_tests::investigation::b38_x03_admission_inventory_only --nocapture > /mnt/b38-x03/out/admission.log 2>&1
test -s /mnt/b38-x03/out/inventory.txt
grep -q "END static-v1" /mnt/b38-x03/out/inventory.txt
