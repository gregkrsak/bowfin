"""Prepared Actions-only acquisition/admission; caller audits and bounds all descendants."""
import hashlib, json, os, select, subprocess, tarfile, time, urllib.request
from pathlib import Path, PurePosixPath

ARCHIVES = (
    ("rustc", "ef1692e3d67236868d32ef26f96f47792b1c3a3f9747bbe05c63742464307c4f"),
    ("rust-std", "c722cba93c9627e04a6a5ecc749cde9dda39f15e4d02fb6ae8d0b27e02e6488a"),
    ("cargo", "5602ba863f5276cfaa7ed3a8dd94d187fbd0319a1b4bbb9284e77fb6b7168a41"),
)

# Codes originate at the check, never from exception text or provider data.
ACQUISITION_CODES = frozenset({
    "ACQUIRE_REDIRECT", "ARCHIVE_BYTES", "ARCHIVE_IDENTITY", "ARCHIVE_ENVELOPE",
    "ARCHIVE_PATH", "ARCHIVE_LINK_OR_SPECIAL", "TOOLCHAIN_METADATA_BYTES", "ACQUISITION_IO_OR_FORMAT",
})


class AcquisitionStop(Exception):
    def __init__(self, code, *, safe_origin=None, safe_secondary=None):
        if code not in ACQUISITION_CODES:
            raise ValueError("UNRECOGNIZED_ACQUISITION_CODE")
        self.code = code
        self.safe_origin = safe_origin
        self.safe_secondary = safe_secondary
        super().__init__(code)


def require_acquisition(condition, code):
    if not condition:
        raise AcquisitionStop(code)


def safe_acquisition_failure(error):
    # Do not retain exception messages, provider text, or arbitrary class names.
    classes = (AcquisitionStop, PermissionError, FileNotFoundError,
               TimeoutError, OSError, ValueError, tarfile.TarError)
    kind = next((cls.__name__ for cls in classes if isinstance(error, cls)), "Exception")
    number = getattr(error, "errno", None)
    number = number if type(number) is int and -4096 <= number <= 4096 else None
    code = error.code if isinstance(error, AcquisitionStop) else "ACQUISITION_IO_OR_FORMAT"
    return {"failure_code": code, "failure_kind": kind, "errno": number,
            "classification_incomplete": kind == "Exception" or code == "ACQUISITION_IO_OR_FORMAT",
            "detail_omitted": True}


def relay_acquisition_failure(failure, operation, secondary=None):
    # Only the explicit projection is relayed to the bounded worker status.
    origin = {"operation": operation, "kind": failure["failure_kind"],
              "errno": failure["errno"], "incomplete": failure["classification_incomplete"],
              "detail_omitted": True}
    secondary = [{"code": item["failure"]["failure_code"], "operation": item["operation"],
                  "kind": item["failure"]["failure_kind"], "errno": item["failure"]["errno"],
                  "incomplete": item["failure"]["classification_incomplete"], "detail_omitted": True}
                 for item in (secondary or [])]
    return AcquisitionStop(failure["failure_code"], safe_origin=origin, safe_secondary=secondary)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args):
        raise AcquisitionStop("ACQUIRE_REDIRECT")

def run(mode):
    assert mode in ("acquire", "admit"), "ADMISSION_MODE"
    task, tool = Path("/task"), Path("/task/toolchain")
    if mode == "acquire":
        records, members, expanded = {}, 0, 0
        progress = {"version": "1.80.0", "files": records, "components": {},
                    "stage": "initial", "status": "incomplete"}
        metadata_path = task / "out/toolchain.json"
        temporary_path = task / "out/toolchain.json.tmp"

        def checkpoint():
            # Commit each stage atomically; a failed replacement retains the last
            # completed snapshot. Both files are inside the fixed task filesystem.
            data = json.dumps(progress, sort_keys=True).encode()
            require_acquisition(len(data) <= 1024**2 - 32768, "TOOLCHAIN_METADATA_BYTES")
            with temporary_path.open("wb") as output:
                output.write(data)
                output.flush()
                os.fsync(output.fileno())
            os.replace(temporary_path, metadata_path)

        def safe_event(event):
            # Fixed field names only: never print exception text, headers or tokens.
            values = {key: progress.get(key) for key in ("stage", "status", "component")}
            values.update(event)
            print("B38_ACQUISITION " + json.dumps(values, sort_keys=True), flush=True)

        def failure_event(event, secondary):
            # A broken log cannot replace the acquisition failure or an earlier
            # checkpoint failure. The worker status carries both bounded facts.
            try:
                safe_event(event)
            except Exception as event_error:
                secondary.append({"operation": "failure_event", "failure": safe_acquisition_failure(event_error)})

        operation = "initial_checkpoint"
        try:
            checkpoint()
            operation = "create_tool_directory"
            tool.mkdir()
            operation = "initialize_downloader"
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        except Exception as error:
            failure = safe_acquisition_failure(error)
            secondary = []
            failure_event({"identity": "primary", "operation": operation, **failure}, secondary)
            raise relay_acquisition_failure(failure, operation, secondary) from None
        for component, expected in ARCHIVES:
            name = component + "-1.80.0-x86_64-unknown-linux-gnu"
            archive, digest, received = task / (component + ".tar.xz"), hashlib.sha256(), 0
            component_record = {"expected_sha256": expected, "status": "downloading"}
            progress.update(component=component, stage="download")
            progress["components"][component] = component_record
            operation = "component_checkpoint"
            try:
                checkpoint()
                operation = "download_component"
                with opener.open("https://static.rust-lang.org/dist/" + name + ".tar.xz", timeout=30) as response, archive.open("xb") as output:
                    while chunk := response.read(1024 * 1024):
                        received += len(chunk)
                        digest.update(chunk)
                        require_acquisition(received <= 256 * 1024**2, "ARCHIVE_BYTES")
                        output.write(chunk)
                observed = digest.hexdigest()
                component_record.update(received_bytes=received, observed_sha256=observed,
                                        digest_scope="complete_archive")
                operation = "verify_archive"
                require_acquisition(observed == expected, "ARCHIVE_IDENTITY")
                component_record["status"] = "archive_verified"
                progress["stage"] = "extract"
                operation = "archive_checkpoint"
                checkpoint()
                operation = "extract_component"
                with tarfile.open(archive, "r|xz") as source:
                    for member in source:
                        members += 1
                        expanded += member.size
                        parts = PurePosixPath(member.name).parts
                        require_acquisition(members <= 8000 and 0 <= member.size and expanded <= 2 * 1024**3, "ARCHIVE_ENVELOPE")
                        require_acquisition(len(member.name) <= 4096 and parts and parts[0] == name and ".." not in parts, "ARCHIVE_PATH")
                        require_acquisition(member.isfile() or member.isdir(), "ARCHIVE_LINK_OR_SPECIAL")
                        if member.isdir() or len(parts) < 3 or parts[2] not in ("bin", "lib", "libexec"):
                            continue
                        destination = tool.joinpath(*parts[2:])
                        destination.parent.mkdir(parents=True, exist_ok=True)
                        file_digest = hashlib.sha256()
                        with source.extractfile(member) as input_file, destination.open("xb") as output:
                            while chunk := input_file.read(1024 * 1024):
                                file_digest.update(chunk)
                                output.write(chunk)
                        destination.chmod(0o755 if member.mode & 0o111 else 0o644)
                        records[str(destination.relative_to(tool))] = file_digest.hexdigest()
                archive.unlink()
                component_record.update(status="complete", cumulative_members=members,
                                        cumulative_expanded_bytes=expanded)
                progress["stage"] = "component_complete"
                operation = "completion_checkpoint"
                checkpoint()
                safe_event({"component_status": "complete", "received_bytes": received,
                            "observed_sha256": observed})
            except Exception as error:
                # Hash scope is explicit: an interrupted download digest is only
                # the received prefix, and never a verified archive identity.
                component_record.setdefault("received_bytes", received)
                component_record.setdefault("observed_sha256", digest.hexdigest())
                component_record.setdefault("digest_scope", "received_prefix")
                component_record["status"] = "failed"
                failure = safe_acquisition_failure(error)
                component_record.update(failure, identity="primary", operation=operation)
                progress["status"] = "incomplete"
                secondary = []
                try:
                    checkpoint()
                except Exception as checkpoint_error:
                    secondary.append({"operation": "failure_checkpoint", "failure": safe_acquisition_failure(checkpoint_error)})
                failure_event({"identity": "primary", "operation": operation, **failure,
                            "received_bytes": component_record["received_bytes"],
                            "observed_sha256": component_record["observed_sha256"],
                            "digest_scope": component_record["digest_scope"],
                            "secondary": [{"identity": "secondary", "operation": item["operation"],
                                           **item["failure"]} for item in secondary]}, secondary)
                raise relay_acquisition_failure(failure, operation, secondary) from None
        progress.update(stage="complete", status="complete")
        try:
            checkpoint()
        except Exception as error:
            failure = safe_acquisition_failure(error)
            secondary = []
            failure_event({"identity": "primary", "operation": "final_checkpoint", **failure}, secondary)
            raise relay_acquisition_failure(failure, "final_checkpoint", secondary) from None
        return
    manifest = json.loads(Path("/src/experiments/bed-038/x03/launch-manifest.json").read_text())
    assert manifest["live_sensor_limit"] == 1 and manifest["admission"]["traversal_depth"] == 22 and manifest["admission"]["working_storage"] == 1073741824, "ADMISSION_SELECTION"
    assert manifest["profile"] == "sensor-demo only for custody; Sample; roots monitor and b38_probe" and manifest["grants"].startswith("ALL; open/read/finish/release;"), "ADMISSION_AUTHORITY"
    for filename, expected in manifest["fixture_sha256"].items():
        assert hashlib.sha256((Path("/src/experiments/bed-038/x03") / filename).read_bytes()).hexdigest() == expected, "FIXTURE_IDENTITY"
    combined = Path("/src/examples/sentinel.bow").read_bytes() + b"\n" + Path("/src/experiments/bed-038/x03/companion-suffix.bow").read_bytes()
    assert len(combined) == 7866 and hashlib.sha256(combined).hexdigest() == manifest["combined_sha256"], "COMBINED_IDENTITY"
    environment = dict(PATH=str(tool / "bin") + ":/usr/bin:/bin", HOME="/task", TMPDIR="/task/tmp", LANG="C", CARGO_HOME="/task/cargo", CARGO_TARGET_DIR="/task/target", CARGO_BUILD_JOBS="2", RUSTC=str(tool / "bin/rustc"), RUSTDOC=str(tool / "bin/rustdoc"), RUSTFLAGS="--cfg b38_x03 --check-cfg=cfg(b38_x03)", B38_X03_OUTPUT="/task/out")
    for directory in ("tmp", "cargo"):
        (task / directory).mkdir(exist_ok=True)
    process = subprocess.Popen([str(tool / "bin/rustc"), "-vV"], stdout=subprocess.PIPE, env=environment)
    version, deadline = bytearray(), time.monotonic() + 10
    try:
        while True:
            assert select.select([process.stdout], [], [], max(0, deadline - time.monotonic()))[0], "VERSION_TIMEOUT"
            chunk = os.read(process.stdout.fileno(), 16385 - len(version))
            if not chunk:
                break
            version.extend(chunk)
            assert len(version) <= 16384, "VERSION_BYTES"
        assert process.wait(timeout=5) == 0, "VERSION_STATUS"
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        process.stdout.close()
    assert b"release: 1.80.0" in version and b"host: x86_64-unknown-linux-gnu" in version, "VERSION_IDENTITY"
    path = task / "out/toolchain.json"
    assert path.stat().st_size <= 1024**2 - 32768, "TOOLCHAIN_METADATA_BYTES"
    metadata = json.loads(path.read_bytes())
    metadata["rustc_vV"] = version.decode("utf-8")
    encoded = json.dumps(metadata, sort_keys=True).encode()
    assert len(encoded) <= 1024**2, "TOOLCHAIN_METADATA_BYTES"
    path.write_bytes(encoded)
    subprocess.run([str(tool / "bin/cargo"), "test", "--offline", "--locked", "--lib", "b38_x03_admission_inventory_only", "--", "--exact", "execution::machine::conformance::b38_x03_tests::investigation::b38_x03_admission_inventory_only", "--nocapture"], cwd="/src", env=environment, check=True, timeout=700)
