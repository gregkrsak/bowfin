"""Future probe verifier source; not connected to any authorized workflow phase.

A separate build/probe authorization and reviewed outer resource/import gates are
required. This verifier cannot qualify those gates or grant execution authority.
"""
import csv
import hashlib
import json
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import time


def invoke(binary, arguments):
    process = subprocess.Popen([str(binary), *arguments], stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True,
        start_new_session=True, env=dict(PATH='/opt/bootstrap/bin', HOME='/work/home',
        TMPDIR='/work/tmp', LC_ALL='C', TZ='UTC'))
    streams, deadline, failure = {'stdout':bytearray(),'stderr':bytearray()}, time.monotonic()+10, None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ, 'stdout')
    selector.register(process.stderr, selectors.EVENT_READ, 'stderr')
    try:
        while selector.get_map():
            if time.monotonic() >= deadline:
                raise RuntimeError('X01_TIMEOUT')
            for key, _ in selector.select(.05):
                size=sum(map(len,streams.values()))
                data = os.read(key.fileobj.fileno(), min(4096, 16385-size))
                if not data:
                    selector.unregister(key.fileobj)
                    continue
                streams[key.data].extend(data)
                if sum(map(len,streams.values()))>16384:
                    raise RuntimeError('X01_OUTPUT_BOUND')
        process.wait(timeout=max(.01, deadline-time.monotonic()))
    except Exception as error:
        failure=str(error)
    finally:
        # The exact retained Rust fixture spawns no child. Still kill any surviving
        # group before returning/hashing; the separately reviewed outer cgroup owns
        # final descendant-cessation evidence on every path, including timeout.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        if process.poll() is None:
            process.wait(timeout=2)
        process.stdout.close()
        process.stderr.close()
        selector.close()
    return {'status':process.returncode,'failure':failure,
            'stdout_hex':bytes(streams['stdout']).hex(),
            'stderr_hex':bytes(streams['stderr']).hex()}


def main():
    if os.environ.get('B38_FUTURE_PROBE_AUTH')!='separate-reviewed-authorization-required':
        raise SystemExit('X01_NOT_AUTHORIZED_BY_PREPARATION')
    binary, native_object = map(Path, sys.argv[1:3])
    # The header discriminator never executes modified bytes. Full imports,
    # executable provenance and protection gates remain outer prerequisites.
    with native_object.open('rb') as source:
        header = source.read(64)
    if len(header)!=64 or header[:7]!=b'\x7fELF\x02\x01\x01' or header[16:20]!=b'\x01\x00\x3e\x00':
        raise SystemExit('X01_ELF_TARGET')
    path = Path('/work/probes/EXPECTATIONS.csv')
    with path.open('rb') as source:
        data = source.read(803)
    if len(data)!=802 or hashlib.sha256(data).hexdigest()!='acf6d6c63491aab19e8984e449f4318871885c0a1120aa0cc51b15c891232d3b':
        raise SystemExit('X01_EXPECTATIONS_IDENTITY')
    for row in csv.DictReader(data.decode().splitlines()):
        record = invoke(binary, [row['a'], row['b'], row['context']])
        # Always retain bounded observed output/status; nonzero negatives are not
        # collapsed by check=True. The called fixture is not trusted by its output.
        print(json.dumps({'id':row['id'],**record},sort_keys=True), flush=True)
        expected = [row[key] for key in ('lo','hi','status','context_after')]
        output=bytes.fromhex(record['stdout_hex'])
        if record['failure'] or record['status'] or record['stderr_hex'] or output.decode(errors='replace').split()!=expected:
            raise SystemExit('X01_VALUE:'+row['id'])


if __name__=='__main__':
    main()
