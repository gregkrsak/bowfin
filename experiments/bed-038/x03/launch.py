"""Actions-only admission preflight. No local Rust invocation is permitted."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = ROOT / "b38-x03-results"
TASK = Path("/mnt/b38-x03")
IMAGE = ROOT / "b38-x03-scratch.ext4"


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
        "workflow_sha256": hashlib.sha256((ROOT / ".github/workflows/bed-038-x03.yml").read_bytes()).hexdigest(),
        "no_interpreter_execution": True,
    }
    try:
        assert len(record["allowed_cpus"]) >= 2, "two CPUs unavailable"
        available = int(next(x.split()[1] for x in record["meminfo"].splitlines() if x.startswith("MemAvailable:"))) * 1024
        assert available >= 5 * 1024**3, "insufficient RAM headroom"
        assert record["free_disk"] >= 6 * 1024**3, "insufficient disk headroom"
        assert Path("/sys/fs/cgroup/cgroup.controllers").exists(), "cgroup v2 unavailable"
        subprocess.run(["fallocate", "-l", "4G", str(IMAGE)], check=True)
        subprocess.run(["mkfs.ext4", "-q", "-m", "0", str(IMAGE)], check=True)
        TASK.mkdir(exist_ok=False)
        subprocess.run(["mount", "-o", "loop,nodev,nosuid", str(IMAGE), str(TASK)], check=True)
        subprocess.run(["chown", "runner:runner", str(TASK)], check=True)
        cpus = ",".join(map(str, record["allowed_cpus"][:2]))
        argv = ["systemd-run", "--unit=b38-x03", "--wait", "--pipe", "--collect",
                "-p", "User=runner", "-p", "CPUQuota=200%", "-p", f"AllowedCPUs={cpus}",
                "-p", "MemoryMax=4G", "-p", "MemorySwapMax=0", "-p", "RuntimeMaxSec=720",
                "-p", "ProtectSystem=strict", "-p", "ProtectHome=read-only",
                "-p", f"ReadWritePaths={TASK}", "-p", "ReadOnlyPaths=/dev/shm",
                "-p", "NoNewPrivileges=true", "-p", "ProtectKernelTunables=true",
                "-p", f"WorkingDirectory={ROOT}", "-p", "Environment=PATH=/home/runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
                "bash", str(ROOT / "experiments/bed-038/x03/probe.sh")]
        (TASK / "out").mkdir(exist_ok=True)
        subprocess.run(["chown", "runner:runner", str(TASK / "out")], check=True)
        with (TASK / "out" / "group.log").open("w") as log:
            result = subprocess.run(argv, stdout=log, stderr=subprocess.STDOUT, timeout=780)
        record["process_exit"] = result.returncode
        for path in (TASK / "out").glob("*"):
            assert path.is_file() and path.stat().st_size <= 8 * 1024**2, "output ceiling"
            shutil.copyfile(path, OUT / path.name)
        assert result.returncode == 0, "enforcement or admission stage failed; inspect group.log"
        record["status"] = "inventory-complete-cross-check-not-launched"
    except Exception as error:
        record["stop"] = str(error)
    finally:
        record["finished"] = time.time()
        (OUT / "launch-result.json").write_text(json.dumps(record, indent=2) + "\n")
        hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in OUT.iterdir() if p.is_file()}
        (OUT / "sha256.json").write_text(json.dumps(hashes, indent=2) + "\n")
    return 0 if record["status"] == "inventory-complete-cross-check-not-launched" else 1


if __name__ == "__main__":
    sys.exit(main())
