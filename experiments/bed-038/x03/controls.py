"""Inert preflight cases and prior expectations; importing never runs a case."""
import errno
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

import audit


class ControlError(RuntimeError):
    def __init__(self, code, detail):
        self.code = code
        self.detail = detail
        super().__init__(code + ": " + str(detail))


class BoundedWriter:
    def __init__(self, descriptor, ceiling):
        self.descriptor, self.left = descriptor, ceiling

    def write(self, data):
        if len(data) > self.left:
            raise ControlError("OUTPUT_BOUND", "rejected before write")
        view = memoryview(data)
        while view:
            count = os.write(self.descriptor, view)
            if count <= 0:
                raise ControlError("OUTPUT_IO", "no progress")
            self.left -= count
            view = view[count:]


EXPECTATIONS = {
    "finite-write": {"diagnostic": "OK", "pair": "output-bound", "bytes": 4096},
    "blocked-valid": {"diagnostic": "OK", "pair": "blocked-writes", "bytes": 1},
    "blocked-writes": {"diagnostic": "OK", "pair": "blocked-valid", "bytes": 0},
    "audit-valid": {"diagnostic": "OK", "pair": "audit-missing", "bytes": 1048576},
    "audit-missing": {"diagnostic": "AUDIT_MISSING_CONTROL", "pair": "audit-valid", "bytes": 1048576},
    "audit-core-negative": {"diagnostic":"AUDIT_CORE_POLICY","pair":"audit-valid","bytes":1048576,"injection":"observed nonzero soft and hard core limits; no actual limit change or crash"},
    "audit-wrong": {"diagnostic": "AUDIT_WRONG_CONTROL", "pair": "audit-valid", "bytes": 1048576},
    "mount-negative": {"diagnostic": "AUDIT_WRITABLE_MOUNT", "pair": "audit-valid", "parent_injection": "actual unexpected writable subordinate mount", "bytes": 1048576},
    "output-bound": {"diagnostic": "OUTPUT_BOUND", "pair": "finite-write", "bytes": 4096},
    "child-valid": {"diagnostic": "OK", "pair": "child-timeout", "bytes": 4096},
    "child-timeout": {"diagnostic": "CHILD_TIMEOUT_CEASED", "pair": "child-valid", "parent_injection": "short effective runtime; verify killed child and empty cgroup", "bytes": 4096},
    "status-below": {"diagnostic": "OK", "pair": "status-over", "status_bytes": 4095},
    "status-exact": {"diagnostic": "OK", "pair": "status-over", "status_bytes": 4096},
    "status-over": {"diagnostic": "STATUS_OVERSIZE", "pair": "status-exact", "attempted_status_bytes": 4097},
    "collection-valid": {"diagnostic": "OK", "pair": "collection-failure", "executor": "parent", "bytes": 4096,
                         "checks": "production collection; exact copied member set and retained bytes"},
    "collection-failure": {"diagnostic": "COLLECTION_TARGET_NOT_REGULAR", "pair": "collection-valid", "executor": "parent", "bytes": 4096,
                           "injection": "owned directory obstructs an actual production copy target; preserve primary, incomplete status and partial evidence"},
    "cleanup-valid": {"diagnostic": "OK", "pair": "cleanup-failure", "executor": "parent", "bytes": 4096,
                      "checks": "production cleanup handler on an owned fixture; repeated cleanup is idempotent"},
    "cleanup-failure": {"diagnostic": "CLEANUP_INCOMPLETE", "pair": "cleanup-valid", "executor": "parent", "bytes": 4096,
                        "injection": "bounded owned child holds an owned bind mount busy; production cleanup records the failure as secondary, then child ceases and cleanup retries"},
}
BLOCKED = {"/run/lock/b38-control": {errno.EACCES, errno.EROFS, errno.ENOENT},
           "/dev/shm/b38-control": {errno.EACCES, errno.EROFS, errno.ENOENT},
           "/dev/mqueue/b38-control": {errno.EACCES, errno.EROFS, errno.ENOENT},
           "/tmp/b38-control": {errno.EACCES, errno.EROFS, errno.ENOENT},
           "/etc/b38-control": {errno.EACCES, errno.EROFS},
           "/src/b38-control": {errno.EACCES, errno.EROFS}}


def write_read(name, data):
    path = Path("/task") / ("control-" + name + ".bin")
    with path.open("xb") as output:
        BoundedWriter(output.fileno(), 4096).write(data)
    if path.read_bytes() != data:
        raise ControlError("CONTROL_READBACK", name)
    return {"case": name, "bytes": len(data), "diagnostic": "OK"}


def run_case(name, **audit_kwargs):
    if name not in EXPECTATIONS:
        raise ControlError("CONTROL_CASE", name)
    if EXPECTATIONS[name].get("executor") == "parent":
        raise ControlError("CONTROL_PARENT_ONLY", name)
    if name in {"audit-valid", "mount-negative", "audit-missing", "audit-wrong", "audit-core-negative"}:
        injected = {"memory.max": None} if name == "audit-missing" else (
                    {"memory.max": "1"} if name == "audit-wrong" else (
                    {"core_limit": (1, 1)} if name == "audit-core-negative" else None))
        return audit.audit(**audit_kwargs, overrides=injected)
    if name in {"status-below", "status-exact", "status-over"}:
        return {"case": name}
    if name == "finite-write":
        return write_read(name, b"F" * 4096)
    if name == "blocked-valid":
        return write_read(name, b"B")
    if name == "blocked-writes":
        observed = []
        for path, allowed in BLOCKED.items():
            try:
                descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            except OSError as error:
                if error.errno not in allowed:
                    raise ControlError("BLOCKED_WRONG_DIAGNOSTIC", [path, error.errno]) from error
                observed.append([path, error.errno])
            else:
                os.close(descriptor)
                raise ControlError("BLOCKED_WRITE_ACCEPTED", path)
        return {"diagnostic": "OK", "rejections": observed}
    if name == "output-bound":
        with Path("/task/control-output-bound.bin").open("xb") as output:
            bounded = BoundedWriter(output.fileno(), 4096)
            bounded.write(b"O" * 4096)
            bounded.write(b"X")
        raise ControlError("CONTROL_NO_REJECTION", name)
    if name == "child-valid":
        child = subprocess.Popen([sys.executable, "-c", "pass"], close_fds=True)
        if child.wait(timeout=5) != 0:
            raise ControlError("CHILD_EXIT", child.returncode)
        return {"diagnostic": "OK", "child": child.pid}
    if name == "child-timeout":
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        code = ("import os,signal,time;from pathlib import Path;"
                "signal.signal(signal.SIGTERM,signal.SIG_IGN);"
                "Path('/task/child-ready').write_text(str(os.getpid()));"
                "time.sleep(600)")
        child = subprocess.Popen([sys.executable, "-c", code], close_fds=True)
        with Path("/task/child-pids.json").open("xb") as output:
            BoundedWriter(output.fileno(), 4096).write(
                json.dumps({"parent": os.getpid(), "child": child.pid}).encode())
        while True:
            time.sleep(1)
    raise ControlError("CONTROL_UNIMPLEMENTED", name)
