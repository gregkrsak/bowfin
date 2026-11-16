"""Read effective worker controls before any protected payload; no import actions."""
import json
import os
from pathlib import Path
import re
import resource
import stat


class AuditError(RuntimeError):
    def __init__(self, code, detail):
        self.code = code
        super().__init__(code + ": " + str(detail))


def require(condition, code, detail):
    if not condition:
        raise AuditError(code, detail)


def read(path, ceiling=262144):
    try:
        with open(path, "rb") as source:
            data = source.read(ceiling + 1)
    except OSError as error:
        raise AuditError("AUDIT_MISSING_CONTROL", [str(path), error.errno]) from error
    require(len(data) <= ceiling, "AUDIT_READ_BOUND", str(path))
    return data.decode("utf-8")


def cpus(value):
    result = set()
    for part in value.split(","):
        ends = part.split("-")
        first, last = int(ends[0]), int(ends[-1])
        require(0 <= first <= last < 4096, "AUDIT_CPU", value)
        result.update(range(first, last + 1))
    return sorted(result)


def mount_path(value):
    return re.sub(r"\\([0-7]{3})", lambda match: chr(int(match[1], 8)), value)


def audit(task="/task", *, memory_bytes, expected_cpus, disk_bytes, host_namespaces, overrides=None):
    status = dict(line.split(":", 1) for line in read("/proc/self/status").splitlines())
    require(all(key in status for key in ["NoNewPrivs", "Seccomp", "CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"]),
            "AUDIT_MISSING_CONTROL", "process status fields")
    observed = {key: status[key].strip() for key in
                ["NoNewPrivs", "Seccomp", "CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"]}
    groups = os.getgroups()
    observed.update(uid=os.getuid(), gid=os.getgid(), groups=groups,
                    affinity=sorted(os.sched_getaffinity(0)), nofile=resource.getrlimit(resource.RLIMIT_NOFILE),
                    core_limit=resource.getrlimit(resource.RLIMIT_CORE),
                    core_pattern=read("/proc/sys/kernel/core_pattern",4096).strip())
    relative = next(line.split(":", 2)[2] for line in read("/proc/self/cgroup").splitlines()
                    if line.startswith("0:"))
    require(relative.startswith("/") and ".." not in Path(relative).parts, "AUDIT_CGROUP", relative)
    cgroup = Path("/sys/fs/cgroup") / relative.lstrip("/")
    for key in ["cpu.max", "cpuset.cpus.effective", "memory.max", "memory.swap.max", "pids.max"]:
        observed[key] = read(cgroup / key, 4096).strip()
    if overrides is not None:
        require(set(overrides) <= set(observed), "AUDIT_CONTROL_KEY", sorted(overrides))
        observed.update(overrides)
    require(all(value is not None for value in observed.values()), "AUDIT_MISSING_CONTROL", observed)
    require(tuple(observed["core_limit"])==(0,0),
            "AUDIT_CORE_POLICY", "zero hard/soft core limit required")
    require(observed["uid"] == observed["gid"] == 65534 and set(groups) <= {65534},
            "AUDIT_IDENTITY", [observed["uid"], observed["gid"], groups])
    require(observed["NoNewPrivs"] == "1" and observed["Seccomp"] == "2" and
            all(int(observed[key], 16) == 0 for key in observed if key.startswith("Cap")),
            "AUDIT_PRIVILEGE", observed)
    require(0 < observed["nofile"][0] <= observed["nofile"][1] <= 64,
            "AUDIT_DESCRIPTOR_LIMIT", observed["nofile"])
    try:
        quota, period = map(int, observed["cpu.max"].split())
    except ValueError as error:
        raise AuditError("AUDIT_WRONG_CONTROL", "CPU quota") from error
    require(0 < quota <= 2 * period and period > 0 and len(expected_cpus) == 2 and
            observed["affinity"] == sorted(expected_cpus) == cpus(observed["cpuset.cpus.effective"]),
            "AUDIT_WRONG_CONTROL", "CPU")
    require(int(observed["memory.max"]) == memory_bytes and observed["memory.swap.max"] == "0" and
            0 < int(observed["pids.max"]) <= 64, "AUDIT_WRONG_CONTROL", "memory/swap/processes")
    descriptors = []
    for name in os.listdir("/proc/self/fd"):
        try:
            number = int(name)
            target = os.readlink("/proc/self/fd/" + name)
            info = os.stat("/proc/self/fd/" + name)
        except FileNotFoundError:
            continue  # The directory enumeration descriptor has already closed.
        accepted = number == 0 and target == "/dev/null"
        accepted |= number in (1, 2) and (stat.S_ISFIFO(info.st_mode) or
                    stat.S_ISREG(info.st_mode) and target.startswith(task + "/") and
                    info.st_dev == os.stat(task).st_dev)
        require(accepted and not stat.S_ISSOCK(info.st_mode), "AUDIT_DESCRIPTOR", [number, target])
        descriptors.append([number, target, info.st_dev, info.st_ino])
    namespaces = {name: os.readlink("/proc/self/ns/" + name) for name in ["mnt", "ipc", "net", "pid", "user"]}
    require(all(name in host_namespaces and namespaces[name] != host_namespaces[name]
                for name in ["mnt", "ipc", "pid"]), "AUDIT_NAMESPACE", namespaces)
    raw = read("/proc/self/mountinfo", 524288).splitlines()
    require(len(raw) <= 512, "AUDIT_MOUNT_COUNT", len(raw))
    records = []
    for line in raw:
        fields, backing = line.split(" - ", 1)
        fields, backing = fields.split(), backing.split()
        records.append({"id": int(fields[0]), "parent": int(fields[1]), "device": fields[2],
                        "root": mount_path(fields[3]), "path": mount_path(fields[4]),
                        "options": fields[5], "filesystem": backing[0], "super": backing[2]})
    inactive, result, seen_task = [], [], False
    for row in sorted(records, key=lambda item: (item["path"].count("/"), item["id"])):
        path = row["path"]
        if any(path.startswith(prefix + "/") for prefix in inactive):
            row["decision"] = "covered by inaccessible ancestor"
        else:
            descriptor = os.open(path, os.O_PATH | os.O_CLOEXEC)
            try:
                actual = int(next(line.split(":", 1)[1] for line in
                                  read(f"/proc/self/fdinfo/{descriptor}", 4096).splitlines()
                                  if line.startswith("mnt_id:")))
                info = os.fstat(descriptor)
            finally:
                os.close(descriptor)
            if actual != row["id"]:
                row["decision"] = "covered by effective mount " + str(actual)
            elif stat.S_IMODE(info.st_mode) == 0 and not os.access(path, os.R_OK | os.W_OK | os.X_OK):
                inactive.append(path)
                row["decision"] = "inaccessible; zero mode and no effective access"
            else:
                require(f"{os.major(info.st_dev)}:{os.minor(info.st_dev)}" == row["device"],
                        "AUDIT_MOUNT_IDENTITY", row)
                volume = os.statvfs(path)
                row.update(read_only=bool(volume.f_flag & os.ST_RDONLY),
                           capacity=volume.f_blocks * volume.f_frsize)
                if path == task:
                    require(row["filesystem"] == "ext4" and not row["read_only"] and
                            0 < row["capacity"] <= disk_bytes and os.access(task, os.W_OK | os.X_OK),
                            "AUDIT_TASK_STORAGE", row)
                    seen_task = True
                    row["decision"] = "exact bounded task filesystem"
                else:
                    require(row["read_only"], "AUDIT_WRITABLE_MOUNT", row)
                    row["decision"] = "read-only input or API mount"
        result.append(row)
    require(seen_task, "AUDIT_TASK_STORAGE", "missing effective task mount")
    devices = []
    for entry in os.scandir("/dev"):
        info = entry.stat(follow_symlinks=False)
        require(len(devices) < 32, "AUDIT_DEVICE", "device member ceiling")
        if stat.S_ISCHR(info.st_mode):
            require((os.major(info.st_rdev), os.minor(info.st_rdev)) in
                    {(1, 3), (1, 5), (1, 7), (1, 8), (1, 9), (5, 0), (5, 2)},
                    "AUDIT_DEVICE", entry.path)
        else:
            require(entry.name in {"fd", "stdin", "stdout", "stderr", "core", "pts", "ptmx", "shm", "mqueue"}
                    and (stat.S_ISDIR(info.st_mode) or stat.S_ISLNK(info.st_mode)), "AUDIT_DEVICE", entry.path)
        devices.append([entry.name, info.st_mode, info.st_rdev])
    output = {"controls": observed, "descriptors": descriptors, "namespaces": namespaces,
              "mounts": result, "devices": devices, "cgroup": relative}
    require(len(json.dumps(output).encode()) <= 1048576, "AUDIT_OUTPUT_BOUND", "audit JSON")
    return output
