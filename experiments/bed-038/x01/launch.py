"""Actions-only resource and bootstrap metadata snapshot. No build/probe launch."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = ROOT / "b38-x01-results"
TASK = Path("/mnt/b38-x01")
IMAGE = ROOT / "b38-x01-scratch.ext4"


def command(*argv):
    return subprocess.check_output(argv, text=True).strip()


def main():
    assert os.environ.get("GITHUB_ACTIONS") == "true", "Actions only"
    OUT.mkdir(exist_ok=True)
    record = {
        "stage": "resource-preflight", "status": "stopped", "started": time.time(),
        "commit": os.environ["GITHUB_SHA"], "tree": command("git", "rev-parse", "HEAD^{tree}"),
        "runner_image": os.environ.get("ImageOS"),
        "runner_image_version": os.environ.get("ImageVersion"),
        "kernel": command("uname", "-a"), "allocated_vcpus": os.cpu_count(),
        "allowed_cpus": sorted(os.sched_getaffinity(0)),
        "meminfo": Path("/proc/meminfo").read_text(),
        "free_disk": shutil.disk_usage(ROOT).free,
        "baseline_runner_class_observed_cpus": os.environ.get("B38_BASELINE_CPUS"),
        "workflow_sha256": hashlib.sha256((ROOT / ".github/workflows/bed-038-x01.yml").read_bytes()).hexdigest(),
        "no_interpreter_execution": True, "no_image_build_or_abi_probe": True,
    }
    try:
        outside_bytes = sum(p.stat().st_size for p in ROOT.rglob("*") if p.is_file())
        record["outside_task_input_bytes"] = outside_bytes
        assert outside_bytes + 16 * 1024**2 < 128 * 1024**2, "outside input/output reserve exceeded"
        assert len(record["allowed_cpus"]) >= 2, "two CPUs unavailable"
        available = int(next(x.split()[1] for x in record["meminfo"].splitlines() if x.startswith("MemAvailable:"))) * 1024
        assert available >= 7 * 1024**3, "insufficient RAM headroom"
        assert record["free_disk"] >= 12 * 1024**3, "insufficient disk headroom"
        assert Path("/sys/fs/cgroup/cgroup.controllers").exists(), "cgroup v2 unavailable"
        record["checkout_bytes"] = sum(p.stat().st_size for p in ROOT.rglob("*") if p.is_file())
        assert record["checkout_bytes"] <= 64 * 1024**2, "checkout exceeds reserved disk account"
        subprocess.run(["fallocate", "-l", str(10 * 1024**3 - 128 * 1024**2), str(IMAGE)], check=True)
        subprocess.run(["mkfs.ext4", "-q", "-m", "0", str(IMAGE)], check=True)
        TASK.mkdir(exist_ok=False)
        subprocess.run(["mount", "-o", "loop,nodev,nosuid", str(IMAGE), str(TASK)], check=True)
        subprocess.run(["chown", "runner:runner", str(TASK)], check=True)
        record["scratch_image_bytes"] = IMAGE.stat().st_size
        record["disk_account"] = "128 MiB of 10 GiB reserved: checkout <=64 MiB, snapshot <=32 MiB, upload overhead <=32 MiB; 10112 MiB filesystem"
        cpus = ",".join(map(str, record["allowed_cpus"][:2]))
        argv = ["systemd-run", "--unit=b38-x01", "--wait", "--pipe", "--collect",
                "-p", "User=runner", "-p", "CPUQuota=200%", "-p", f"AllowedCPUs={cpus}",
                "-p", "MemoryMax=6G", "-p", "MemorySwapMax=0", "-p", "RuntimeMaxSec=240",
                "-p", "ProtectSystem=strict", "-p", "ProtectHome=read-only",
                "-p", f"ReadWritePaths={TASK}", "-p", "ReadOnlyPaths=/dev/shm",
                "-p", "NoNewPrivileges=true", "-p", "ProtectKernelTunables=true",
                "-p", f"WorkingDirectory={ROOT}", "-p", "Environment=PATH=/home/runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
                "bash", str(ROOT / "experiments/bed-038/x01/snapshot.sh")]
        (TASK / "out").mkdir(exist_ok=True)
        subprocess.run(["chown", "runner:runner", str(TASK / "out")], check=True)
        with (TASK / "out" / "group.log").open("w") as log:
            result = subprocess.run(argv, stdout=log, stderr=subprocess.STDOUT, timeout=270)
        record["process_exit"] = result.returncode
        assert sum(p.stat().st_size for p in (TASK / "out").glob("*")) <= 16 * 1024**2, "aggregate output ceiling"
        for path in (TASK / "out").glob("*"):
            assert path.is_file() and path.stat().st_size <= 8 * 1024**2, "output ceiling"
            shutil.copyfile(path, OUT / path.name)
        assert result.returncode == 0, "resource or metadata stage failed; inspect group.log"
        record["status"] = "resource-and-bootstrap-snapshot-only-gates-unresolved"
    except Exception as error:
        record["stop"] = str(error)
    finally:
        if TASK.is_mount():
            subprocess.run(["umount", str(TASK)], check=False)
        if not TASK.is_mount() and IMAGE.exists():
            IMAGE.unlink()
        record["finished"] = time.time()
        (OUT / "launch-result.json").write_text(json.dumps(record, indent=2) + "\n")
        hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in OUT.iterdir() if p.is_file()}
        (OUT / "sha256.json").write_text(json.dumps(hashes, indent=2) + "\n")
    return 0 if record["status"] == "resource-and-bootstrap-snapshot-only-gates-unresolved" else 1


if __name__ == "__main__":
    sys.exit(main())
