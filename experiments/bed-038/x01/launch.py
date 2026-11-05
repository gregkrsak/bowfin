"""Prepared Actions-only lifecycle. No import actions and no automatic progression."""
import hashlib
import json
import os
from pathlib import Path
import selectors
import signal
import stat
import subprocess
import sys
import time
import zipfile
sys.path.insert(0, str(Path(__file__).parent))
from controls import BoundedWriter, ControlError, EXPECTATIONS
import audit

PACKAGE = Path(__file__).parent.name
GIB, MIB = 1024**3, 1024**2
TOTAL, MEMORY = (10*GIB, 6*GIB) if PACKAGE == 'x01' else (4*GIB, 4*GIB)
W = Path(os.environ.get('GITHUB_WORKSPACE', '/not-actions'))
S = Path('/mnt/b38-' + PACKAGE + '-scratch')
IMAGE = Path(str(S) + '.ext4')
SLICE = 'b38-' + PACKAGE + '.slice'
UNITS, RECORDS = [], []
SLICES, COMPLETED, WITNESSES = {}, {}, {}
UPLOAD_EXPECTED, UPLOAD_OBJECT = None, None
PROPS = ['RuntimeMaxUSec','TimeoutStartUSec','TimeoutStopUSec','KillMode','SendSIGKILL',
         'Restart','KillSignal','FinalKillSignal','Id','InvocationID','LoadState','ControlGroup','Job','Result','ExecMainCode','ExecMainStatus','ActiveState','SubState','ExitType','RemainAfterExit',
         'NoNewPrivileges','PrivateDevices','PrivateIPC','PrivateMounts','PrivateNetwork',
         'ProtectSystem','RootDirectory','RestrictAddressFamilies','RestrictNamespaces',
         'CapabilityBoundingSet','AmbientCapabilities','LimitCORE','LimitNOFILE','TasksMax','SystemCallFilter']

def attach_secondary(primary,error,operation):
    # Only bounded, fixed metadata survives a secondary cleanup error.
    kinds={'ControlError','AuditError','OSError','PermissionError','FileNotFoundError','TimeoutError','ValueError','BrokenPipeError','Exception'}
    kind=type(error).__name__;number=getattr(error,'errno',None)
    code=getattr(error,'code',None)
    if not isinstance(code,str) or code not in globals().get('PARENT_CODES',()): code='CLEANUP_SECONDARY'
    row={'stop':code,'stage':'finalization','operation':operation,
         'kind':kind if kind in kinds else 'Exception',
         'errno':number if type(number) is int and -4096<=number<=4096 else None,
         'incomplete':True}
    existing=getattr(primary,'secondary_errors',None)
    if not isinstance(existing,list): existing=[]
    primary.secondary_errors=(existing+[row])[:64]
    if len(existing)>=64: primary.secondary_omitted=getattr(primary,'secondary_omitted',0)+1

def close_fd_preserving(fd):
    primary=sys.exc_info()[1]
    try: os.close(fd)
    except BaseException as error:
        if primary is None: raise
        attach_secondary(primary,error,'descriptor_close')

class preserving_close:
    def __init__(self,obj): self.obj=obj
    def __enter__(self): return self.obj
    def __exit__(self,kind,primary,tb):
        try: self.obj.close()
        except BaseException as error:
            if primary is None: raise
            attach_secondary(primary,error,'descriptor_close')
        return False

def control_environment(env=None):
    # Do not forward the upload parent's provider credentials to controls.
    allowed={'PATH','HOME','TMPDIR','LC_ALL','LANG','PYTHONDONTWRITEBYTECODE'}
    result={'PATH':'/usr/sbin:/usr/bin:/sbin:/bin','LC_ALL':'C','LANG':'C'}
    if env is not None:
        if set(env)-allowed: raise ControlError('COMMAND_FAILED','invalid control environment')
        result.update(env)
    return result

def bounded(argv, seconds=10, cap=MIB, env=None):
    p = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT, close_fds=True, start_new_session=True, env=control_environment(env))
    poll=None
    chunks, size, deadline = [], 0, time.monotonic()+seconds
    failure=None
    try:
        poll=selectors.DefaultSelector();poll.register(p.stdout, selectors.EVENT_READ)
        while poll.get_map():
            if time.monotonic() >= deadline: raise ControlError('COMMAND_TIMEOUT', argv[0])
            for key, _ in poll.select(min(.1, max(0, deadline-time.monotonic()))):
                b = os.read(key.fileobj.fileno(), min(65536, cap-size+1))
                if not b: poll.unregister(key.fileobj); continue
                if size+len(b)>cap: raise ControlError('OUTPUT_BOUND', argv[0])
                size+=len(b); chunks.append(b)
        return p.wait(timeout=max(.01, deadline-time.monotonic())), b''.join(chunks)
    except BaseException as error:
        failure=error
        raise
    finally:
        cleanup=[]
        try:
            if p.poll() is None: os.killpg(p.pid, signal.SIGKILL)
        except BaseException as error: cleanup.append(('client_kill',error))
        try:
            if p.poll() is None: p.wait(timeout=2)
        except BaseException as error: cleanup.append(('client_wait',error))
        try: p.stdout.close()
        except BaseException as error: cleanup.append(('descriptor_close',error))
        try:
            if poll is not None: poll.close()
        except BaseException as error: cleanup.append(('descriptor_close',error))
        if failure is not None:
            failure.partial={'bytes':size,'returncode':p.returncode,'incomplete':True}
            for operation,error in cleanup: attach_secondary(failure,error,operation)
        elif cleanup:
            first=cleanup[0][1]
            for operation,error in cleanup[1:]: attach_secondary(first,error,operation)
            raise first

def command(*args, seconds=10):
    code, data = bounded(list(args), seconds)
    if code: raise ControlError('COMMAND_FAILED', [args[0], code, data[-256:].decode(errors='replace')])
    return data.decode().strip()

def write(path, value):
    b=json.dumps(value, sort_keys=True).encode()
    if len(b)>MIB: raise ControlError('RECORD_BOUND', path.name)
    with preserving_close(path.open('xb')) as f: BoundedWriter(f.fileno(), MIB).write(b)

STATUS_LIMIT = 4096
STATUS_EXERCISES = {'status-below':4095, 'status-exact':4096, 'status-over':4097}

def status_bytes(case, diagnostic, result=None, detail=None, failure=None):
    frame={'case':case,'diagnostic':diagnostic}
    if result is not None: frame['result']=result
    if detail is not None: frame['detail']=detail[:256]
    if failure is not None: frame['failure']=failure
    target=STATUS_EXERCISES.get(case)
    if target is not None and diagnostic=='OK':
        frame['padding']=''
        base=json.dumps(frame,sort_keys=True,separators=(',',':')).encode()
        if len(base)>target: raise ControlError('STATUS_EXERCISE_INVALID',case)
        frame['padding']='x'*(target-len(base))
    encoded=json.dumps(frame,sort_keys=True,separators=(',',':')).encode()
    if len(encoded)>STATUS_LIMIT:
        encoded=json.dumps({'case':case,'diagnostic':'STATUS_OVERSIZE',
              'original_diagnostic':diagnostic,'attempted_bytes':len(encoded)},
              sort_keys=True,separators=(',',':')).encode()
        if len(encoded)>STATUS_LIMIT: raise ControlError('STATUS_FALLBACK_BOUND',case)
        return encoded, True
    return encoded, False

def write_status(case, diagnostic, result=None, detail=None, failure=None):
    encoded, rejected=status_bytes(case,diagnostic,result,detail,failure)
    path=Path('/task')/('status-'+case+'.json')
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
    try: BoundedWriter(fd,STATUS_LIMIT).write(encoded)
    finally: close_fd_preserving(fd)
    return rejected

def read_status(case):
    path=S/'work'/('status-'+case+'.json')
    try: fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
    except FileNotFoundError as error: raise ControlError('STATUS_MISSING',case) from error
    try:
        info=os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size>STATUS_LIMIT:
            raise ControlError('STATUS_FILE_OVERSIZE',case)
        with preserving_close(os.fdopen(fd,'rb',closefd=False)) as stream: encoded=stream.read(STATUS_LIMIT+1)
        if len(encoded)!=info.st_size or len(encoded)>STATUS_LIMIT:
            raise ControlError('STATUS_FILE_OVERSIZE',case)
    finally: close_fd_preserving(fd)
    try: frame=json.loads(encoded)
    except (UnicodeDecodeError,ValueError) as error: raise ControlError('STATUS_MALFORMED',case) from error
    allowed={'case','diagnostic','result','detail','padding','original_diagnostic','attempted_bytes','failure'}
    if not isinstance(frame,dict) or set(frame)-allowed or frame.get('case')!=case or not isinstance(frame.get('diagnostic'),str):
        raise ControlError('STATUS_MALFORMED',case)
    if 'failure' in frame and (not valid_failure(frame['failure']) or frame['failure']['code']!=frame['diagnostic']):
        raise ControlError('STATUS_MALFORMED',case)
    if case in ('status-below','status-exact') and len(encoded)!=STATUS_EXERCISES[case]:
        raise ControlError('STATUS_EXERCISE_MISMATCH',[case,len(encoded)])
    if case=='status-over' and (frame.get('diagnostic')!='STATUS_OVERSIZE' or frame.get('attempted_bytes')!=4097):
        raise ControlError('STATUS_EXERCISE_MISMATCH',case)
    return frame,len(encoded)

SAFE_CODES = frozenset(('WORKER_FAILURE','STATUS_WRITE_FAILED','PERMIT_TIMEOUT','PERMIT_IDENTITY',
    'PHASE','RECORD_BOUND','OUTPUT_BOUND','OUTPUT_IO','UPLOAD_AUDIT_BOUND','UPLOAD_AUDIT_DUPLICATE',
    'UPLOAD_COPY_INVALID','UPLOAD_AUDIT_INVALID','AUDIT_CGROUP','AUDIT_CONTROL_KEY','AUDIT_CORE_POLICY',
    'AUDIT_CPU','AUDIT_DESCRIPTOR','AUDIT_DESCRIPTOR_LIMIT','AUDIT_DEVICE','AUDIT_IDENTITY',
    'AUDIT_MISSING_CONTROL','AUDIT_MOUNT_COUNT','AUDIT_MOUNT_IDENTITY','AUDIT_NAMESPACE',
    'AUDIT_OUTPUT_BOUND','AUDIT_PRIVILEGE','AUDIT_READ_BOUND','AUDIT_TASK_STORAGE',
    'AUDIT_WRITABLE_MOUNT','AUDIT_WRONG_CONTROL','CONTROL_CASE','CONTROL_NO_REJECTION',
    'CONTROL_PARENT_ONLY','CONTROL_READBACK','CONTROL_UNIMPLEMENTED','BLOCKED_WRITE_ACCEPTED',
    'BLOCKED_WRONG_DIAGNOSTIC','CHILD_EXIT','COMMAND_FAILED','COMMAND_TIMEOUT',
    'ADMISSION_AUTHORITY','ADMISSION_MODE','ADMISSION_SELECTION','ARCHIVE_BYTES','ARCHIVE_IDENTITY',
    'ARCHIVE_ENVELOPE','ARCHIVE_PATH','ARCHIVE_LINK_OR_SPECIAL','TOOLCHAIN_METADATA_BYTES',
    'ACQUIRE_REDIRECT','ACQUISITION_FAILED','ACQUISITION_IO_OR_FORMAT','UPLOAD_PROVIDER_FAILURE','UPLOAD_COPY_SETUP_FAILURE','CLEANUP_SECONDARY','ARCHIVE_PRIMARY_INVALID','STATUS_MALFORMED','STATUS_MISSING','STATUS_FALLBACK_MALFORMED','STATUS_FALLBACK_LOG_BOUND','TRANSPORT_RECORD_BOUND','UPLOAD_STATUS_MISMATCH'))
SAFE_KINDS = frozenset(('ControlError','AuditError','OSError','PermissionError','FileNotFoundError',
    'FileExistsError','IsADirectoryError','NotADirectoryError','TimeoutError','ValueError',
    'UnicodeDecodeError','KeyError','AssertionError','BadZipFile','AcquisitionStop','Exception',
    'BrokenPipeError','ChildProcessError','ProcessLookupError','TarError'))
SAFE_OPERATIONS = frozenset(('permit','audit','control','metadata','acquisition','audit_read',
    'archive_open','archive_append','archive_verify','object_record','adapter_prepare','adapter_exec','provider_execute','status_write'))

ACQUISITION_OPERATIONS = frozenset(('initial_checkpoint','create_tool_directory','initialize_downloader',
    'component_checkpoint','download_component','verify_archive','archive_checkpoint','extract_component',
    'completion_checkpoint','failure_checkpoint','failure_event','final_checkpoint','descriptor_close','client_kill','client_wait','status_recovery'))

def valid_acquisition_details(origin,secondary):
    def numeric(value): return value is None or type(value) is int and -4096<=value<=4096
    def valid(item,origin_item):
        expected={'operation','kind','errno','incomplete','detail_omitted'} | (set() if origin_item else {'code'})
        return (isinstance(item,dict) and set(item)==expected and isinstance(item['operation'],str) and item['operation'] in ACQUISITION_OPERATIONS
          and isinstance(item['kind'],str) and item['kind'] in SAFE_KINDS and numeric(item['errno']) and type(item['incomplete']) is bool
          and item['detail_omitted'] is True and (origin_item or isinstance(item['code'],str) and item['code'] in SAFE_CODES))
    return ((origin is None or valid(origin,True)) and isinstance(secondary,list) and len(secondary)<=4
            and all(valid(item,False) for item in secondary))

def safe_failure(error, operation='status_write'):
    code=getattr(error,'code',None)
    known=isinstance(code,str) and code in SAFE_CODES
    kind=type(error).__name__
    known_kind=kind in SAFE_KINDS
    number=getattr(error,'errno',None)
    valid_errno=number is None or type(number) is int and -4096<=number<=4096
    supplied_status=getattr(error,'status',None)
    valid_status=supplied_status is None or type(supplied_status) is int and -255<=supplied_status<=255
    safe_origin=getattr(error,'safe_origin',None)
    incoming=getattr(error,'safe_secondary',[])
    nested_omitted=not isinstance(incoming,(list,type(None)))
    safe_secondary=list(incoming[:4]) if isinstance(incoming,list) else []
    if isinstance(incoming,list) and len(incoming)>4: nested_omitted=True
    extra=getattr(error,'secondary_errors',[])
    if not isinstance(extra,list): extra=[];nested_omitted=True
    if len(safe_secondary)+len(extra)>4: nested_omitted=True
    for detail in extra[:max(0,4-len(safe_secondary))]:
        if isinstance(detail,dict):
            safe_secondary.append({'code':detail.get('stop') if isinstance(detail.get('stop'),str) and detail.get('stop') in SAFE_CODES else 'CLEANUP_SECONDARY','operation':detail.get('operation','descriptor_close'),
                'kind':detail.get('kind','Exception'),'errno':detail.get('errno'),'incomplete':True,'detail_omitted':True})
    if not valid_acquisition_details(safe_origin,[]):
        safe_origin=None;nested_omitted=True
    retained=[]
    for item in safe_secondary:
        if valid_acquisition_details(None,[item]): retained.append(item)
        else: nested_omitted=True
    safe_secondary=retained
    return {'stage':'worker_setup','operation':operation if operation in SAFE_OPERATIONS else 'status_write',
            'code':code if known else 'WORKER_FAILURE','kind':kind if known_kind else 'Exception',
            'errno':number if valid_errno else None, 'status':getattr(error,'status',None) if type(getattr(error,'status',None)) is int and -255<=getattr(error,'status',None)<=255 else None,
            'incomplete':not known or not known_kind or not valid_errno or not valid_status or operation not in SAFE_OPERATIONS or nested_omitted or bool(getattr(error,'secondary_omitted',0)),
            'nested_fields_omitted':nested_omitted,'detail_omitted':True, 'safe_origin':safe_origin, 'safe_secondary':safe_secondary}

def valid_failure(item):
    return (isinstance(item,dict) and set(item)=={'stage','operation','code','kind','errno','status','incomplete','nested_fields_omitted','detail_omitted','safe_origin','safe_secondary'}
        and item['stage'] in ('worker_setup','upload_copy') and isinstance(item['operation'],str) and item['operation'] in SAFE_OPERATIONS
        and isinstance(item['code'],str) and item['code'] in SAFE_CODES
        and isinstance(item['kind'],str) and item['kind'] in SAFE_KINDS
        and (item['errno'] is None or type(item['errno']) is int and -4096<=item['errno']<=4096)
        and (item['status'] is None or type(item['status']) is int and -255<=item['status']<=255)
        and type(item['incomplete']) is bool and type(item['nested_fields_omitted']) is bool and item['detail_omitted'] is True
        and valid_acquisition_details(item['safe_origin'],item['safe_secondary']))

def status_write_fallback(case, first, status_error):
    # Fixed safe fields only: no exception text, paths, environment or tokens.
    record={'case':case,'primary':first,'status_write':safe_failure(status_error)}
    encoded=json.dumps(record,sort_keys=True,separators=(',',':'))
    if len(encoded.encode())>4096: raise ControlError('STATUS_FALLBACK_OVERSIZE',case)
    try: print('B38_STATUS_WRITE_FAILURE '+encoded,flush=True)
    except BaseException:
        try: print('B38_STATUS_WRITE_FAILURE '+encoded,file=sys.stderr,flush=True)
        except BaseException: pass  # No output channel: parent reports missing evidence.

def read_status_fallback(case):
    path=W/'out'/(case+'.log')
    with path.open('rb') as stream: data=stream.read(MIB+1)
    if len(data)>MIB: raise ControlError('STATUS_FALLBACK_LOG_BOUND',case)
    prefix=b'B38_STATUS_WRITE_FAILURE '
    rows=[line[len(prefix):] for line in data.splitlines() if line.startswith(prefix)]
    if not rows: return None
    if len(rows)!=1 or len(rows[0])>4096: raise ControlError('STATUS_FALLBACK_MALFORMED',case)
    try: record=json.loads(rows[0])
    except (ValueError,UnicodeDecodeError) as error: raise ControlError('STATUS_FALLBACK_MALFORMED',case) from error
    if not isinstance(record,dict) or set(record)!={'case','primary','status_write'} or record.get('case')!=case:
        raise ControlError('STATUS_FALLBACK_MALFORMED',case)
    if not all(valid_failure(record[name]) for name in ('primary','status_write')):
        raise ControlError('STATUS_FALLBACK_MALFORMED',case)
    return record

def duration(text):
    import re
    units={'us':.000001,'ms':.001,'s':1,'min':60,'h':3600}
    parts=re.findall(r'([0-9]+)(us|ms|min|s|h)',text)
    if not parts or ''.join(n+u for n,u in parts)!=text.replace(' ',''): raise ControlError('UNIT_DEADLINE_FORMAT',text)
    return sum(int(n)*units[u] for n,u in parts)

def props(unit, seconds=3):
    return dict(line.split('=',1) for line in command('systemctl','show',unit,
                '--property='+','.join(PROPS),seconds=seconds).splitlines() if '=' in line)

# Precommitted only for the fixed timeout control; the manager's recorded
# main-process signal is evidence of that process, not of each descendant.
CHILD_TIMEOUT_PLAN = {'kind':'systemd_runtime_timeout','runtime_seconds':3,
                      'kill_signal':9,'expected_code':2,'expected_status':9,
                      'readiness_absent':True}
INVENTORY = None

def inventory_write():
    if INVENTORY is None:
        raise ControlError('UNIT_INVENTORY_UNAVAILABLE','inventory')
    target='inventory-'+INVENTORY['mode']+'.json'
    directory=os.open(W/'out',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC)
    try:
        info=os.fstat(directory)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid!=0:
            raise ControlError('UNIT_INVENTORY_PARENT','ownership')
        try:
            old=os.stat(target,dir_fd=directory,follow_symlinks=False)
        except FileNotFoundError:
            old=None
        if old is not None and (not stat.S_ISREG(old.st_mode) or old.st_uid!=0 or old.st_nlink!=1 or stat.S_IMODE(old.st_mode)!=0o600):
            raise ControlError('UNIT_INVENTORY_PARENT','target identity')
        raw=json.dumps(INVENTORY,sort_keys=True,separators=(',',':')).encode()
        if len(raw)>MIB: raise ControlError('UNIT_INVENTORY_BOUND','existing evidence ceiling')
        temporary='.'+target+'.tmp'
        fd=os.open(temporary,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600,dir_fd=directory)
        try:
            os.fchmod(fd,0o600)
            BoundedWriter(fd,MIB).write(raw)
            os.fsync(fd)
        finally: close_fd_preserving(fd)
        os.replace(temporary,target,src_dir_fd=directory,dst_dir_fd=directory)
        os.fsync(directory)
    finally: close_fd_preserving(directory)

def inventory_parent_identity(mode):
    # Self-identify inside the already-confined parent before launching workers.
    import re
    unit='b38-'+PACKAGE+('-parent.service' if mode=='run' else '-upload-parent.service')
    actual=props(unit)
    group=actual.get('ControlGroup','');invocation=actual.get('InvocationID','')
    if (actual.get('Id')!=unit or not re.fullmatch(r'[0-9a-f]{32}',invocation)
        or not group.startswith('/') or not group.endswith('/'+unit)
        or '/b38-'+PACKAGE+'.slice/' not in group):
        raise ControlError('UNIT_INVENTORY_PARENT','parent service identity')
    raw_job=actual.get('Job','').split()
    if raw_job and not raw_job[0].isdigit(): raise ControlError('UNIT_INVENTORY_PARENT','parent job identity')
    job=None if not raw_job or raw_job[0]=='0' else raw_job[0]
    record={'version':1,'package':PACKAGE,'run_id':os.environ['GITHUB_RUN_ID'],
            'commit':os.environ['GITHUB_SHA'],'unit':{'Id':unit,'InvocationID':invocation,
            'ControlGroup':group,'Job':job}}
    path=(W/'out/inventory-parent.json') if mode=='run' else (W/'inventory-upload-parent.json')
    raw=json.dumps(record,sort_keys=True,separators=(',',':')).encode()
    if len(raw)>4096: raise ControlError('UNIT_INVENTORY_BOUND','parent record')
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
    try:
        os.fchmod(fd,0o600);info=os.fstat(fd)
        if info.st_uid!=0 or info.st_nlink!=1: raise ControlError('UNIT_INVENTORY_PARENT','record ownership')
        BoundedWriter(fd,4096).write(raw);os.fsync(fd)
    finally: close_fd_preserving(fd)

def inventory_worker(unit, **updates):
    if INVENTORY is None or unit not in INVENTORY['workers']:
        raise ControlError('UNIT_INVENTORY_UNAVAILABLE','worker')
    INVENTORY['workers'][unit].update(updates)
    inventory_write()

def launcher_start_time(pid):
    raw=Path('/proc')/str(pid)/'stat'
    data=raw.read_text()
    if len(data)>4096 or ')' not in data:
        raise ControlError('UNIT_INVENTORY_LAUNCHER','stat')
    fields=data[data.rfind(')')+2:].split()
    if len(fields)<=19 or not fields[19].isdigit():
        raise ControlError('UNIT_INVENTORY_LAUNCHER','start time')
    return int(fields[19])


def child_timeout_evidence():
    # These PIDs are local to the worker PID namespace, not host PIDs. The
    # independently observed empty slice, not these numbers, proves cessation.
    def read_owned(name):
        fd=os.open(S/'work'/name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
        try:
            before=os.fstat(fd)
            if (not stat.S_ISREG(before.st_mode) or before.st_uid!=65534
                or before.st_nlink!=1 or before.st_size>4096):
                raise ValueError('identity')
            chunks=[];size=0
            while size<=4096:
                chunk=os.read(fd,4097-size)
                if not chunk: break
                chunks.append(chunk);size+=len(chunk)
            after=os.fstat(fd)
            identity=lambda st:(st.st_dev,st.st_ino,st.st_uid,st.st_mode,st.st_nlink,
                                st.st_size,st.st_mtime_ns,st.st_ctime_ns)
            if size>4096 or size!=before.st_size or identity(before)!=identity(after):
                raise ValueError('changed')
            return b''.join(chunks).decode('ascii')
        finally: close_fd_preserving(fd)
    try:
        ready=read_owned('child-ready'); raw=read_owned('child-pids.json')
        pairs=json.loads(raw,object_pairs_hook=lambda value:value)
        if (not isinstance(pairs,list) or len(pairs)!=2
            or any(not isinstance(pair,tuple) or len(pair)!=2 for pair in pairs)
            or {pair[0] for pair in pairs}!={'parent','child'}):
            raise ValueError('pid fields')
        pids=dict(pairs);parent=pids['parent'];child=pids['child']
        if (type(parent) is not int or type(child) is not int
            or not 0<parent<=2147483647 or not 0<child<=2147483647
            or parent==child or ready!=str(child)):
            raise ValueError('pid identity')
        return {'pid_namespace':'worker','parent':parent,'child':child,
                'ready':ready,'pids_json':raw}
    except (OSError,UnicodeError,ValueError,TypeError) as error:
        raise ControlError('CHILD_DISCRIMINATOR','pid evidence') from error

def finish_witness(unit, require_valid_termination=True):
    try:
        record=WITNESSES[unit]
        # A failed stop is incomplete; retry only outstanding bounded cleanup, using
        # the already-captured result and cgroup witness. Never infer result/cessation
        # from a vanished unit or repeat a completed cleanup action.
        def require_stop_identity(current, retained, expected):
            invocation=retained.get('InvocationID')
            if (retained.get('Id')!=expected or current.get('Id')!=expected
                or not isinstance(invocation,str) or len(invocation)!=32
                or invocation=='0'*32 or any(c not in '0123456789abcdef' for c in invocation)
                or current.get('InvocationID')!=invocation
                or not retained.get('ControlGroup') or current.get('ControlGroup')!=retained['ControlGroup']):
                raise ControlError('UNIT_IDENTITY_LOST',current)
        if record['service_stop_required']:
            current=props(unit)
            require_stop_identity(current,record['terminal'],unit)
            if current.get('ActiveState')!='active' or current.get('SubState')!='exited':
                raise ControlError('UNIT_TERMINAL_UNPROVEN',current)
            command('systemctl','stop',unit,seconds=8)
            record['service_stop_required']=False
        if record['slice_stop_required']:
            current=props(record['slice']['Id'])
            require_stop_identity(current,record['slice'],record['slice']['Id'])
            if current.get('ActiveState')!='active' or not current.get('ControlGroup'):
                raise ControlError('UNIT_SLICE_UNPROVEN',record['slice']['Id'])
            command('systemctl','stop',record['slice']['Id'],seconds=8)
            record['slice_stop_required']=False
        record['cleanup_complete']=True
        inventory_worker(unit,stage='cleaned',cleanup_complete=True)
        COMPLETED[unit]=record['terminal']
    except BaseException as error:
        error.after_worker_cessation=True
        raise
    if require_valid_termination and record.get('termination_error'):
        raise ControlError(record['termination_error'],unit)
    return COMPLETED[unit]

def cease(unit, before, require_valid_termination=True):
    # A cached qualification includes actual prior evidence, never mere absence.
    if unit in COMPLETED:
        if require_valid_termination and WITNESSES[unit].get('termination_error'):
            raise ControlError(WITNESSES[unit]['termination_error'],unit)
        return COMPLETED[unit]
    if unit in WITNESSES:
        return finish_witness(unit,require_valid_termination)
    slice_name=SLICES[unit]
    terminal=props(unit)
    state=terminal.get('ActiveState'); substate=terminal.get('SubState')
    if state not in ('failed','inactive') and not (state=='active' and substate=='exited'):
        # A unit name alone does not authorize intervention on a replacement.
        inventory_worker(unit,pre_stop_terminal=terminal)
        invocation=(before or {}).get('InvocationID')
        if ((before or {}).get('Id')!=unit or terminal.get('Id')!=unit
            or not isinstance(invocation,str) or len(invocation)!=32 or invocation=='0'*32
            or any(c not in '0123456789abcdef' for c in invocation)
            or terminal.get('InvocationID')!=invocation
            or not (before or {}).get('ControlGroup')
            or terminal.get('ControlGroup')!=(before or {}).get('ControlGroup')):
            raise ControlError('UNIT_IDENTITY_LOST',terminal)
        # Persist the stop intent before it can influence the terminal result.
        # A stop attempt always disqualifies the result, even if the command fails.
        inventory_worker(unit,forced_stop=True)
        command('systemctl','stop',unit,seconds=8)
        terminal=props(unit)
        state=terminal.get('ActiveState');substate=terminal.get('SubState')
    if state not in ('failed','inactive') and not (state=='active' and substate=='exited'):
        raise ControlError('UNIT_TERMINAL_UNPROVEN',unit)
    # Retain even incomplete terminal data before validation or cleanup.
    inventory_worker(unit,stage='terminal_observed',terminal=terminal)
    if terminal.get('Id')!=unit or not terminal.get('Result') or not terminal.get('InvocationID') or terminal.get('InvocationID')!=(before or {}).get('InvocationID'):
        raise ControlError('UNIT_RESULT_UNPROVEN',terminal)
    parent=props(slice_name)
    group_name=parent.get('ControlGroup','')
    unit_group=(before or {}).get('ControlGroup','')
    if parent.get('Id')!=slice_name or parent.get('ActiveState')!='active' or not group_name or not unit_group.startswith(group_name+'/'):
        raise ControlError('UNIT_SLICE_UNPROVEN',[unit,group_name,unit_group])
    group=Path('/sys/fs/cgroup')/group_name.lstrip('/')
    # The explicitly active per-unit parent slice remains an independent witness
    # after systemd prunes the failed service's own cgroup. Absence is incomplete.
    try:
        events=(group/'cgroup.events').read_text().strip()
        processes=(group/'cgroup.procs').read_text().strip()
    except OSError as error:
        raise ControlError('UNIT_CESSATION_UNPROVEN',[unit,type(error).__name__,error.errno]) from error
    if 'populated 0' not in events.splitlines() or processes:
        raise ControlError('DESCENDANTS_REMAIN',[unit,events,processes[:256]])
    record={'unit':unit,'effective':before,'terminal':terminal,'slice':parent,
            'forced_stop':INVENTORY['workers'][unit].get('forced_stop'),
            'pre_stop_terminal':INVENTORY['workers'][unit].get('pre_stop_terminal'),
            'slice_events':events,'slice_processes':processes,'cgroup_empty':True,
            'cleanup_complete':False,'service_stop_required':state=='active',
            'slice_stop_required':True,'termination_error':'UNIT_RESULT_UNPROVEN'}
    RECORDS.append(record)
    # Preserve result and witness *before* any fallible service/slice cleanup.
    inventory_worker(unit,stage='witness_captured',effective=before,terminal=terminal,
                     slice_evidence=parent,slice_events=events,slice_processes=processes)
    WITNESSES[unit]=record
    # systemd's ExecMainCode is the numeric siginfo CLD_* value. Only
    # CLD_EXITED with a concrete exit status is normally admissible; in
    # particular, CLD_DUMPED and absent or unknown statuses never qualify.
    code=terminal.get('ExecMainCode'); status=terminal.get('ExecMainStatus')
    valid_status=isinstance(status,str) and len(status)<=3 and status.isascii() and status.isdecimal() and 0<=int(status)<=255
    info=INVENTORY['workers'][unit]
    child_control=(PACKAGE=='x01' and PHASE=='preflight' and info['case']=='child-timeout'
                   and info['role']=='control' and info['ordinal']==10
                   and unit=='b38-'+PACKAGE+'-control-10.service'
                   and info['timeout']==3 and not info['injected'])
    termination_error=None; child_evidence=None
    if child_control:
        try: child_evidence=child_timeout_evidence()
        except ControlError: termination_error='CHILD_DISCRIMINATOR'
    if (info.get('forced_stop') is not False or info.get('pre_stop_terminal') is not None
        or terminal['Result']=='core-dump' or code=='3' or not valid_status):
        termination_error='UNIT_RESULT_UNPROVEN'
    elif child_control:
        if not (info.get('child_timeout_plan')==CHILD_TIMEOUT_PLAN
                and (before or {}).get('KillSignal') in ('9','SIGKILL')
                and (before or {}).get('KillMode')=='control-group'
                and (before or {}).get('SendSIGKILL')=='yes'
                and (before or {}).get('FinalKillSignal') in ('9','SIGKILL')
                and duration((before or {}).get('RuntimeMaxUSec',''))==3
                and duration((before or {}).get('TimeoutStopUSec',''))==2
                and terminal['Result']=='timeout' and child_evidence is not None
                and code=='2' and int(status)==signal.SIGKILL):
            termination_error='CHILD_DISCRIMINATOR'
    elif not (code=='1' and ((terminal['Result']=='success' and int(status)==0)
                            or (terminal['Result']=='exit-code' and int(status)!=0))):
        termination_error='UNIT_RESULT_UNPROVEN'
    record['termination_error']=termination_error
    record['child_timeout_plan']=info.get('child_timeout_plan')
    record['child_timeout_evidence']=child_evidence
    inventory_worker(unit,termination_error=termination_error,child_timeout_evidence=child_evidence)
    if termination_error:
        raise ControlError(termination_error,unit)
    return finish_witness(unit,require_valid_termination)

def run_unit(case, role='control', timeout=8, injected=False):
    global UPLOAD_OBJECT
    ordinal=len(UNITS)
    unit='b38-'+PACKAGE+'-'+role+'-'+str(ordinal)+'.service'
    slice_name='b38-'+PACKAGE+'-u'+str(ordinal)+'.slice'
    child_control=(PACKAGE=='x01' and PHASE=='preflight' and case=='child-timeout'
                   and role=='control' and ordinal==10 and timeout==3 and not injected)
    if case=='child-timeout' and not child_control:
        raise ControlError('CHILD_DISCRIMINATOR',case)
    if child_control:
        for name in ('child-ready','child-pids.json'):
            try: os.lstat(S/'work'/name)
            except FileNotFoundError: continue
            except OSError as error:
                raise ControlError('CHILD_DISCRIMINATOR','readiness unknown') from error
            raise ControlError('CHILD_DISCRIMINATOR','stale readiness')
    UNITS.append(unit);SLICES[unit]=slice_name
    INVENTORY['workers'][unit]={'case':case,'role':role,'timeout':timeout,'injected':injected,
        'ordinal':ordinal,'unit':unit,
        'slice':slice_name,'stage':'slice_start_intent','launcher_pid':None,'launcher_start':None,
        'invocation':None,'effective':None,'terminal':None,'pre_stop_terminal':None,'slice_evidence':None,
        'slice_events':None,'slice_processes':None,'forced_stop':False,
        'termination_error':'UNIT_RESULT_UNPROVEN',
        'child_timeout_evidence':None,'child_timeout_plan':dict(CHILD_TIMEOUT_PLAN) if child_control else None,
        'cleanup_complete':False,
        'client_reaped':False,'client_returncode':None}
    inventory_write()
    command('systemctl','start',slice_name,seconds=5)
    permit=S/'work/permit';permit.unlink(missing_ok=True)
    kw={'memory_bytes':MEMORY,'disk_bytes':TOTAL-128*MIB,'expected_cpus':CPUS,
        'host_namespaces':HOST_NS}
    settings={'Slice':slice_name,'ExitType':'cgroup','RemainAfterExit':'yes','User':'root','Group':'root','MemoryMax':str(MEMORY),
      'MemorySwapMax':'0','CPUQuota':'200%','AllowedCPUs':','.join(map(str,CPUS)),
      'TasksMax':'64','LimitCORE':'0','LimitNOFILE':'64','LimitFSIZE':str(2*GIB if role in ('acquire','admit') else 20*MIB),'RuntimeMaxSec':str(timeout),
      'TimeoutStartSec':'5','TimeoutStopSec':'2','KillMode':'control-group','SendSIGKILL':'yes',
      'Restart':'no','FinalKillSignal':'SIGKILL','RootDirectory':str(S/'root'),'MountAPIVFS':'yes','PrivateDevices':'yes',
      'PrivateIPC':'yes','PrivateMounts':'yes','PrivateNetwork':'no' if role in ('acquire','upload') else 'yes',
      'ProtectSystem':'strict','ReadOnlyPaths':'/ /dev /proc /sys',
      'InaccessiblePaths':'/run /dev/mqueue /dev/shm','ReadWritePaths':'/task',
      'BindPaths':str(S/'work')+':/task','BindReadOnlyPaths':
      '/usr /lib /lib64 /etc /var/lib/dpkg '+str(W/'source')+':/src /sys/fs/cgroup',
      'UnsetEnvironment':'LD_PRELOAD LD_LIBRARY_PATH PYTHONPATH PYTHONHOME','NoNewPrivileges':'yes','CapabilityBoundingSet':'CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_SETPCAP',
      'AmbientCapabilities':'','RestrictNamespaces':'pid mnt',
      'RestrictAddressFamilies':'AF_INET AF_INET6','SystemCallFilter':'~@keyring @module @reboot @swap @raw-io @debug @obsolete shmget shmat shmdt shmctl semget semctl semop semtimedop msgget msgsnd msgrcv msgctl mq_open mq_unlink mq_timedsend mq_timedreceive mq_notify mq_getsetattr io_uring_setup io_uring_enter io_uring_register','RestrictSUIDSGID':'yes','LockPersonality':'yes',
      'ProtectKernelTunables':'yes','ProtectKernelModules':'yes','ProtectKernelLogs':'yes',
      'StandardInput':'null','WorkingDirectory':'/task','UMask':'0077'}
    if child_control: settings['KillSignal']='SIGKILL'
    if injected:
        settings['BindPaths']+=' '+str(S/'injected')+':/surprise';settings['ReadWritePaths']+=' /surprise'
    if role=='upload': settings['BindReadOnlyPaths']+=' '+os.environ['B38_NODE']+':/node'
    args=['systemd-run','--quiet','--pipe','--unit='+unit]
    for k,v in settings.items(): args+=['-p',k+'='+v]
    args+=['--setenv=PYTHONDONTWRITEBYTECODE=1','--setenv=LC_ALL=C','--setenv=LANG=C']
    for name in ('GITHUB_ACTIONS','GITHUB_RUN_ATTEMPT','GITHUB_SHA','B38_PHASE','GITHUB_SERVER_URL','GITHUB_REPOSITORY','GITHUB_RUN_ID'):
        args+=['--setenv='+name+'='+os.environ.get(name,'')]
    args+=['--setenv=HOME=/task','--setenv=PATH=/usr/bin:/bin']
    if role=='upload':
        for name in ('ACTIONS_RUNTIME_TOKEN','ACTIONS_RESULTS_URL'): args+=['--setenv='+name]
    args+=['/usr/bin/unshare','--mount','--propagation','private','--pid','--fork','--mount-proc=/proc',
           '/bin/sh','/src/experiments/bed-038/'+PACKAGE+'/isolate.sh',role,case,json.dumps(kw)]
    inventory_worker(unit,stage='service_start_intent')
    client_env=control_environment()
    if role=='upload':
        for name in ('ACTIONS_RUNTIME_TOKEN','ACTIONS_RESULTS_URL'):
            if name in os.environ: client_env[name]=os.environ[name]
    p=subprocess.Popen(args,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,
                       close_fds=True,start_new_session=True,env=client_env)
    observed=None; deadline=time.monotonic()+timeout+12
    unit_failure=None; cleanup_errors=[]
    try:
        inventory_worker(unit,stage='client_started',launcher_pid=p.pid,launcher_start=launcher_start_time(p.pid))
        startup_deadline=time.monotonic()+4
        startup_error=None
        while time.monotonic()<startup_deadline:
            try:
                observed=props(unit,seconds=1)
                if observed.get('ControlGroup') and observed.get('InvocationID'): break
            except ControlError as error:
                # The manager may not have created the unit yet. A persistent
                # query failure or disappearance is a stop, never a success.
                startup_error=str(error)[:512]
            if p.poll() is not None:
                raise ControlError('UNIT_START_FAILED',[unit,p.returncode,startup_error])
            time.sleep(.05)
        if not observed or not observed.get('ControlGroup') or not observed.get('InvocationID'):
            raise ControlError('UNIT_START_TIMEOUT',[unit,startup_error])
        inventory_worker(unit,stage='unit_observed',invocation=observed['InvocationID'],effective=observed)
        required={'KillMode':'control-group','SendSIGKILL':'yes','Restart':'no',
                  'NoNewPrivileges':'yes','PrivateDevices':'yes','PrivateIPC':'yes',
                  'PrivateMounts':'yes','ProtectSystem':'strict','RootDirectory':str(S/'root'),
                  'Id':unit,'ExitType':'cgroup','RemainAfterExit':'yes'}
        if any(observed.get(k)!=v for k,v in required.items()): raise ControlError('UNIT_POLICY', observed)
        if duration(observed['RuntimeMaxUSec'])!=timeout or duration(observed['TimeoutStopUSec'])!=2 or duration(observed['TimeoutStartUSec'])!=5:
            raise ControlError('UNIT_DEADLINE', observed)
        if child_control and observed.get('KillSignal') not in ('9','SIGKILL'):
            raise ControlError('UNIT_POLICY','timeout intervention signal')
        if set(observed['RestrictAddressFamilies'].split())!={'AF_INET','AF_INET6'} or observed['LimitNOFILE']!='64' or observed['TasksMax']!='64' or observed['LimitCORE']!='0':
            raise ControlError('UNIT_POLICY', 'families/descriptors/processes')
        if set(observed['RestrictNamespaces'].split())!={'mnt','pid'} or not observed['SystemCallFilter']:
            raise ControlError('UNIT_POLICY','namespaces/seccomp')
        if observed['PrivateNetwork']!=('no' if role in ('acquire','upload') else 'yes'):
            raise ControlError('UNIT_POLICY','network')
        if set(observed['CapabilityBoundingSet'].lower().split())!={'cap_sys_admin','cap_setuid','cap_setgid','cap_setpcap'} or observed['AmbientCapabilities']:
            raise ControlError('UNIT_POLICY','bootstrap capabilities')
        if observed['FinalKillSignal'] not in ('9','SIGKILL') or not all(n in observed['SystemCallFilter'] for n in ('shmget','msgget','mq_open','io_uring_setup')):
            raise ControlError('UNIT_POLICY','final kill or IPC filters')
        if p.poll() is not None: raise ControlError('UNIT_CLIENT_EXITED_BEFORE_PERMIT',[unit,p.returncode])
        permit.write_text(case);permit.chmod(0o644)
        poll=selectors.DefaultSelector();poll.register(p.stdout,selectors.EVENT_READ)
        with preserving_close((W/'out'/(case+'.log')).open('xb')) as log:
            sink=BoundedWriter(log.fileno(), MIB)
            next_observation=0
            while poll.get_map():
                if time.monotonic()>deadline: raise ControlError('UNIT_CLIENT_TIMEOUT', case)
                if time.monotonic()>=next_observation and unit not in COMPLETED:
                    current=props(unit,seconds=1)
                    if current.get('Id')!=unit or current.get('InvocationID')!=observed.get('InvocationID'):
                        raise ControlError('UNIT_IDENTITY_LOST',[unit,current])
                    if current.get('ActiveState')=='failed' or (current.get('ActiveState')=='active' and current.get('SubState')=='exited'):
                        stopped=cease(unit,observed)
                    elif current.get('ActiveState')=='inactive':
                        raise ControlError('UNIT_TERMINAL_UNPROVEN',current)
                    next_observation=time.monotonic()+.2
                for key,_ in poll.select(.1):
                    b=os.read(key.fileobj.fileno(),65536)
                    if not b: poll.unregister(key.fileobj);continue
                    sink.write(b)
        if unit not in COMPLETED: stopped=cease(unit,observed)
        poll.close();p.wait(timeout=2)
    except BaseException as error:
        unit_failure=error
    finally:
        try:
            if observed and observed.get('ControlGroup'):
                stopped=cease(unit, observed)
            else:
                raise ControlError('UNIT_CREATION_UNRESOLVED',unit)
        except BaseException as error:
            cleanup_errors.append({'stop':getattr(error,'code',type(error).__name__),
                                   'stage':'unit.cleanup','operation':'cease',
                                   'detail':str(error)[:512]})
        try:
            if p.poll() is None: os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=2)
        except BaseException as error:
            cleanup_errors.append({'stop':type(error).__name__,'stage':'unit.cleanup',
                                   'operation':'client_reap','detail':str(error)[:512]})
        try:
            if p.poll() is None: raise ControlError('UNIT_CLIENT_UNREAPED',unit)
            inventory_worker(unit,client_reaped=True,client_returncode=p.returncode)
        except BaseException as error:
            cleanup_errors.append({'stop':getattr(error,'code','UNIT_INVENTORY_FAILED'),
                                   'stage':'unit.cleanup','operation':'inventory_client'})
        try: p.stdout.close()
        except BaseException as error:
            cleanup_errors.append({'stop':type(error).__name__,'stage':'unit.cleanup',
                                   'operation':'close_stdout'})
        if 'poll' in locals():
            try: poll.close()
            except BaseException as error:
                cleanup_errors.append({'stop':'CLEANUP_SECONDARY','stage':'unit.cleanup','operation':'descriptor_close','kind':type(error).__name__})
    if cleanup_errors:
        if unit_failure is None: unit_failure=ControlError('UNIT_CLEANUP_INCOMPLETE',unit)
        unit_failure.secondary_errors=(getattr(unit_failure,'secondary_errors',[])+cleanup_errors)[:64]
    if unit_failure is not None:
        # A completed per-unit witness makes the worker's fixed status stable.
        # Only a failure known to arise afterwards yields precedence to it;
        # earlier supervisor errors retain their existing primary position.
        if unit in WITNESSES:
            try:
                fallback=read_status_fallback(case)
                if fallback:
                    frame=None;worker=fallback['primary']
                elif role=='upload':
                    with preserving_close((W/'out/upload.log').open('rb')) as upload_log: raw=upload_log.read(MIB+1)
                    if len(raw)>MIB: raise ControlError('TRANSPORT_RECORD_BOUND','upload log')
                    rows=[line[len(b'B38_UPLOAD_FAILURE '):] for line in raw.splitlines() if line.startswith(b'B38_UPLOAD_FAILURE ')]
                    if len(rows)!=1 or len(rows[0])>4096: raise ControlError('UPLOAD_STATUS_MISMATCH','failure frame')
                    worker=json.loads(rows[0])
                    if not valid_failure(worker): raise ControlError('UPLOAD_STATUS_MISMATCH','failure frame')
                    frame={'case':case,'diagnostic':worker['code'],'failure':worker}
                else:
                    frame=read_status(case)[0];worker=frame.get('failure')
                expected=EXPECTATIONS.get(case,{}).get('diagnostic') if role=='control' else 'OK'
                if isinstance(worker,dict) and worker['code']!=expected:
                    if getattr(unit_failure,'after_worker_cessation',False) or getattr(unit_failure,'code',None)=='UNIT_CLEANUP_INCOMPLETE':
                        original=unit_failure
                        unit_failure=ControlError(worker['code'],case)
                        unit_failure.operation='worker.status_write' if fallback else 'run'
                        if fallback: unit_failure.worker_fallback=fallback
                        else: unit_failure.worker_status=frame
                        unit_failure.secondary_errors=[{'stop':getattr(original,'code','UNIT_CLEANUP_INCOMPLETE'),
                            'stage':'finalization','operation':'unit.cleanup','kind':type(original).__name__}]+getattr(original,'secondary_errors',[])
                    else:
                        # Both failures are preserved; neither observation alone
                        # proves which occurred first.
                        unit_failure.precedence_unknown=True
                        if fallback: unit_failure.worker_fallback=fallback
                        else: unit_failure.worker_status=frame
            except (FileNotFoundError,ControlError,ValueError,OSError) as recovery_error:
                # A failed stable-status read is itself secondary, never proof.
                attach_secondary(unit_failure,recovery_error,'status_recovery')
        raise unit_failure
    if case=='child-timeout':
        if not (S/'work/child-ready').is_file() or stopped['Result']!='timeout' or not (S/'work/child-pids.json').is_file():
            raise ControlError('CHILD_DISCRIMINATOR', case)
        return 'CHILD_TIMEOUT_CEASED'
    fallback=read_status_fallback(case)
    if fallback is not None:
        RECORDS.append({'unit':unit,'worker_status_fallback':fallback})
        error=ControlError(fallback['primary']['code'],{'case':case,'status_write':fallback['status_write']})
        error.operation='worker.status_write';error.worker_fallback=fallback
        raise error
    if role=='upload':
        # The provider can write /task; files there are never diagnostic authority.
        with preserving_close((W/'out/upload.log').open('rb')) as upload_log: raw=upload_log.read(MIB+1)
        if len(raw)>MIB: raise ControlError('TRANSPORT_RECORD_BOUND','upload log')
        rows=raw.splitlines(); objects=[x[len(b'B38_UPLOAD_OBJECT '):] for x in rows if x.startswith(b'B38_UPLOAD_OBJECT ')]
        failures=[x[len(b'B38_UPLOAD_FAILURE '):] for x in rows if x.startswith(b'B38_UPLOAD_FAILURE ')]
        if len(objects)>1 or len(failures)>1 or any(len(x)>4096 for x in objects+failures):
            raise ControlError('TRANSPORT_RECORD_BOUND','upload status')
        object_error=None
        if objects:
            try:
                try: data=json.loads(objects[0])
                except (ValueError,UnicodeDecodeError) as cause: raise ControlError('TRANSPORT_RECORD_BOUND','upload object') from cause
                keys={'archive_sha256','bytes','upload_audit_sha256','dev','ino','primary_sha256','primary_bytes'}
                if (not isinstance(data,dict) or set(data)!=keys or any(type(data[k]) is not int for k in ('bytes','dev','ino','primary_bytes'))
                    or not 0<data['bytes']<=17*MIB or not 0<data['primary_bytes']<=17*MIB
                    or any(not isinstance(data[k],str) or len(data[k])!=64 or any(c not in '0123456789abcdef' for c in data[k])
                           for k in ('archive_sha256','upload_audit_sha256','primary_sha256'))
                    or not isinstance(UPLOAD_EXPECTED,dict)
                    or any(data[k]!=UPLOAD_EXPECTED[k] for k in ('dev','ino','primary_sha256','primary_bytes'))):
                    raise ControlError('UPLOAD_OBJECT_CHANGED','worker identity')
                UPLOAD_OBJECT=data
            except BaseException as cause: object_error=cause
        if failures:
            try: failure=json.loads(failures[0])
            except (ValueError,UnicodeDecodeError) as cause: raise ControlError('UPLOAD_STATUS_MISMATCH','failure frame') from cause
            if not valid_failure(failure): raise ControlError('UPLOAD_STATUS_MISMATCH','failure frame')
            error=ControlError(failure['code'],case);error.operation='upload.worker_setup'
            error.worker_status={'case':case,'diagnostic':failure['code'],'failure':failure}
            if object_error is not None: attach_secondary(error,object_error,'object_record')
            raise error
        if object_error is not None: raise object_error
        if p.returncode or stopped['Result']!='success': raise ControlError('UPLOAD_STOP','artifact transport')
        if len(objects)!=1: raise ControlError('TRANSPORT_RECORD_BOUND','upload object missing')
        return 'OK'
    result,status_length=read_status(case)
    RECORDS.append({'unit':unit,'worker_status':result,'status_bytes':status_length})
    if result['diagnostic']=='OK' and (p.returncode or stopped['Result']!='success'): raise ControlError('WORKER_STATUS',case)
    if result['diagnostic']!='OK' and p.returncode==0: raise ControlError('WORKER_STATUS',case)
    if role != 'control' and result['diagnostic']!='OK':
        error=ControlError(result['diagnostic'],case);error.operation='upload.worker_setup' if role=='upload' else 'run';error.worker_status=result
        raise error
    return result['diagnostic']

PARENT_CODES = SAFE_CODES | frozenset(('ADMISSION_STOP', 'ARCHIVE_BOUND', 'ARCHIVE_MEMBER', 'ARCHIVE_READY_EXISTS', 'ARCHIVE_READY_INVALID', 'CHILD_DISCRIMINATOR', 'CLEANUP_CONTROL_TEARDOWN', 'CLEANUP_DISCRIMINATOR', 'CLEANUP_INCOMPLETE', 'COLLECTION_BOUND', 'COLLECTION_CHANGED', 'COLLECTION_DISCRIMINATOR', 'COLLECTION_MEMBERS', 'COLLECTION_TARGET_NOT_REGULAR', 'COMMAND_FAILED', 'COMMAND_TIMEOUT', 'CONTROL_MISMATCH', 'DESCENDANTS_REMAIN', 'HOST_TOOL_BOUND', 'HOST_TOOL_MISSING', 'METADATA_STOP', 'MOUNT_CREATION_UNPROVEN', 'MOUNT_OWNERSHIP_UNKNOWN', 'MOUNT_PREEXISTS', 'MOUNT_SCOPE', 'MOUNT_SOURCE_MISMATCH', 'MOUNT_STATE_UNKNOWN', 'NO_ARCHIVE', 'OUTPUT_BOUND', 'OUTPUT_DISCRIMINATOR', 'PARENT_DEADLINE', 'PARENT_ENVELOPE', 'PARENT_EVIDENCE_OVERSIZE', 'PERMIT_IDENTITY', 'PERMIT_TIMEOUT', 'PHASE', 'PREFLIGHT_ACTIVE_DEADLINE', 'QUALIFICATION_REQUIRED', 'RECORD_BOUND', 'SCRATCH_RETAINED', 'SLICE_ENVELOPE', 'STATUS_EXERCISE_INVALID', 'STATUS_EXERCISE_MISMATCH', 'STATUS_FALLBACK_BOUND', 'STATUS_FALLBACK_LOG_BOUND', 'STATUS_FALLBACK_MALFORMED', 'STATUS_FALLBACK_OVERSIZE', 'STATUS_FILE_OVERSIZE', 'STATUS_MALFORMED', 'STATUS_MISSING', 'STORAGE_NOT_EMPTY', 'TRANSPORT_RECORD_BOUND', 'UNCLASSIFIED_FAILURE', 'UNIT_CESSATION_UNPROVEN', 'UNIT_CLEANUP_INCOMPLETE', 'UNIT_CLIENT_EXITED_BEFORE_PERMIT', 'UNIT_CLIENT_TIMEOUT', 'UNIT_CREATION_UNRESOLVED', 'UNIT_DEADLINE', 'UNIT_DEADLINE_FORMAT', 'UNIT_IDENTITY_LOST', 'UNIT_POLICY', 'UNIT_RESULT_UNPROVEN', 'UNIT_SLICE_UNPROVEN', 'UNIT_START_FAILED', 'UNIT_START_TIMEOUT', 'UNIT_TERMINAL_UNPROVEN', 'UPLOAD_AUDIT_BOUND', 'UPLOAD_AUDIT_DUPLICATE', 'UPLOAD_AUDIT_INVALID', 'UPLOAD_COPY_INVALID', 'UPLOAD_COPY_SETUP_FAILURE', 'UPLOAD_PROVIDER_FAILURE', 'UPLOAD_STATUS_MISMATCH', 'UPLOAD_STOP', 'WORKER_STATUS'))
PARENT_CODES = PARENT_CODES | frozenset(('MOUNT_PROVENANCE_BOUND','MOUNT_PROVENANCE_UNKNOWN','MOUNT_SOURCE_UNKNOWN',
    'UNIT_CLIENT_UNREAPED','UNIT_INVENTORY_BOUND','UNIT_INVENTORY_INCOMPLETE','UNIT_INVENTORY_LAUNCHER',
    'UNIT_INVENTORY_LEASE','UNIT_INVENTORY_PARENT','UNIT_INVENTORY_UNAVAILABLE','UNIT_INVENTORY_FAILED',
    'UPLOAD_OBJECT_CHANGED','CLEANUP_SECONDARY','UNIT_RESET_FAILURE'))
PARENT_STAGES = frozenset(('run','upload','collection','cleanup','finalization','control'))
PARENT_OPERATIONS = frozenset(('run','upload','copy','finish','cease','disposition_or_package',
    'transport_collection','archive_ready_revalidation','copy.enumerate','copy.validate','copy.open',
    'mount.identity','mount.provenance','worker.status_write','upload.worker_setup','upload.copy','unit.cleanup','umount',
    'descriptor_close','client_kill','client_wait','inventory_client','inventory_close','status_recovery','object_record'))

def record_failure(state, error, stage, operation):
    code=getattr(error,'code',None);kind=type(error).__name__
    candidate=getattr(error,'operation',operation)
    valid_code=isinstance(code,str) and code in PARENT_CODES
    valid_kind=kind in SAFE_KINDS
    valid_stage=isinstance(stage,str) and stage in PARENT_STAGES
    valid_operation=isinstance(candidate,str) and candidate in PARENT_OPERATIONS
    record={'stop':code if valid_code else 'UNCLASSIFIED_FAILURE',
            'stage':stage if valid_stage else 'unknown',
            'operation':candidate if valid_operation else 'unknown',
            'kind':kind if valid_kind else 'Exception','detail_omitted':True,
            'classification_incomplete':not all((valid_code,valid_kind,valid_stage,valid_operation))}
    for name in ('errno','status','exit_code','busy_message','observed','limit'):
        value=getattr(error,name,None)
        if type(value) is bool and name=='busy_message': record[name]=value
        elif type(value) is int and (-4096<=value<=4096 if name in ('errno','status','exit_code') else 0<=value<=2**63-1): record[name]=value
        elif value is not None: record['classification_incomplete']=True
    member=getattr(error,'member',None)
    if isinstance(member,str) and 0<len(member)<=128 and member.isascii() and all(c.isalnum() or c in '._-' for c in member):
        record['member']=member
    origin_kind=getattr(error,'origin_kind',None)
    if isinstance(origin_kind,str) and origin_kind in SAFE_KINDS: record['origin_kind']=origin_kind
    if getattr(error,'origin_incomplete',False): record['classification_incomplete']=True
    if getattr(error,'secondary_omitted',0):
        record['secondary_omitted']=error.secondary_omitted;record['classification_incomplete']=True
    causes=getattr(error,'safe_causes',[])
    if isinstance(causes,list):
        projected=[]
        for cause in causes[:8]:
            if not isinstance(cause,dict): record['classification_incomplete']=True;continue
            c=cause.get('stop');o=cause.get('operation');k=cause.get('kind');eno=cause.get('errno');exit_code=cause.get('exit_code')
            projected.append({'stop':c if isinstance(c,str) and c in PARENT_CODES else 'UNCLASSIFIED_FAILURE',
                'operation':o if isinstance(o,str) and o in PARENT_OPERATIONS else 'unknown',
                'kind':k if isinstance(k,str) and k in SAFE_KINDS else 'Exception',
                'errno':eno if type(eno) is int and -4096<=eno<=4096 else None,
                'exit_code':exit_code if type(exit_code) is int and -4096<=exit_code<=4096 else None,
                'incomplete':True})
        if len(causes)>8: record['classification_incomplete']=True
        if projected: record['causes']=projected
    if state['primary'] is None: state['primary']=record
    else: state['secondary'].append(record)
    extra=getattr(error,'secondary_errors',[])
    if not isinstance(extra,list): extra=[];record['classification_incomplete']=True
    if len(extra)>64: record['classification_incomplete']=True
    for item in extra[:64]:
        if not isinstance(item,dict):
            state['secondary'].append({'stop':'SECONDARY_MALFORMED','stage':'finalization','operation':'unknown','classification_incomplete':True})
            continue
        code=item.get('stop');stage2=item.get('stage');operation2=item.get('operation');kind2=item.get('kind')
        number=item.get('errno');status=item.get('exit_code')
        state['secondary'].append({'stop':code if isinstance(code,str) and code in PARENT_CODES else 'UNCLASSIFIED_FAILURE',
            'stage':stage2 if isinstance(stage2,str) and stage2 in PARENT_STAGES else 'finalization',
            'operation':operation2 if isinstance(operation2,str) and operation2 in PARENT_OPERATIONS else 'unknown',
            'kind':kind2 if isinstance(kind2,str) and kind2 in SAFE_KINDS else 'Exception',
            'errno':number if type(number) is int and -4096<=number<=4096 else None,
            'exit_code':status if type(status) is int and -4096<=status<=4096 else None,
            'classification_incomplete':True})
    state['incomplete']=True
    return record


def collect_checked(state, source=None, destination=None):
    # The same bounded operation and failure handler are used for production
    # evidence and inert controls. The caller establishes writer cessation.
    source=S/'work' if source is None else source
    destination=W/'out/collected' if destination is None else destination
    state.setdefault('members',[])
    try:
        destination.mkdir(exist_ok=True)
        members=sorted(source.glob('*-audit.json'))+[source/'out/inventory.txt',source/'out/toolchain.json',source/'child-pids.json']
        if PACKAGE=='x01': members += [source/'out/installed-tools.json',source/'out/installed-packages.tsv']
        # Successful frames are already stored in units.json. Preserve any
        # unread/failed status frame without doubling the normal member count.
        recorded={r['worker_status']['case'] for r in RECORDS if isinstance(r.get('worker_status'),dict) and 'case' in r['worker_status']}
        members += [path for path in sorted(source.glob('status-*.json')) if path.name[7:-5] not in recorded]
        if len(members)>32:
            error=ControlError('COLLECTION_MEMBERS','member limit')
            error.observed=len(members);error.limit=32;error.operation='copy.enumerate'
            raise error
        total=0
        for path in members:
            if not path.exists(): continue
            fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW);info=os.fstat(fd)
            try:
                total+=info.st_size
                if not stat.S_ISREG(info.st_mode) or info.st_size>8*MIB or total>8*MIB:
                    error=ControlError('COLLECTION_BOUND',path.name)
                    error.member=path.name;error.observed=total;error.limit=8*MIB;error.operation='copy.validate'
                    raise error
                target=destination/path.name
                try:
                    output=target.open('xb')
                except FileExistsError as original:
                    # Actual production collision path, including the inert
                    # directory obstruction. Never follow an existing symlink.
                    existing=os.open(target,os.O_RDONLY|os.O_NOFOLLOW)
                    existing_failure=None
                    try:
                        saved=os.fstat(existing)
                        if not stat.S_ISREG(saved.st_mode):
                            error=ControlError('COLLECTION_TARGET_NOT_REGULAR',path.name)
                            error.errno=original.errno;error.operation='copy.open';error.member=path.name
                            raise error from original
                        if saved.st_size!=info.st_size: raise ControlError('COLLECTION_CHANGED',path.name)
                        while True:
                            a=os.read(fd,65536);b=os.read(existing,65536)
                            if a!=b: raise ControlError('COLLECTION_CHANGED',path.name)
                            if not a: break
                    except BaseException as error:
                        existing_failure=error;raise
                    finally:
                        try: os.close(existing)
                        except BaseException as error:
                            if existing_failure is None: raise
                            attach_secondary(existing_failure,error,'descriptor_close')
                else:
                    with preserving_close(output):
                        sink=BoundedWriter(output.fileno(),info.st_size)
                        while True:
                            b=os.read(fd,65536)
                            if not b: break
                            sink.write(b)
                with target.open('rb') as saved:
                    h=hashlib.sha256();count=0
                    while True:
                        data=saved.read(65536)
                        if not data: break
                        count+=len(data)
                        if count>info.st_size: raise ControlError('COLLECTION_CHANGED',path.name)
                        h.update(data)
                    if count!=info.st_size: raise ControlError('COLLECTION_CHANGED',path.name)
                state['members'].append({'name':path.name,'bytes':count,'sha256':h.hexdigest()})
            finally: close_fd_preserving(fd)
        return True
    except BaseException as error:
        record_failure(state,error,'collection','copy')
        return False


def control_mismatch(code, detail, state):
    error=ControlError(code,detail)
    error.control_state=state
    raise error


# Mount identity is derived only for the exact owned target and its source.
# All mount creation intents are durable before invoking mount(8).
MOUNT_RECORD = Path('/mnt/b38-' + PACKAGE + '-mounts.json')

def mount_graph(path):
    path=Path(path)
    allowed=(W,S,S/'control-fixtures'/'mount',S/'work')
    if path not in allowed: raise ControlError('MOUNT_SCOPE','target outside owned paths')
    try:
        fd=os.open(path,os.O_PATH|os.O_NOFOLLOW|os.O_CLOEXEC)
        try:
            info=os.fstat(fd)
            if not stat.S_ISDIR(info.st_mode): raise ValueError('not directory')
            lines=audit.read('/proc/self/fdinfo/'+str(fd),4096).splitlines()
            mids=[int(x.split(':',1)[1].strip()) for x in lines if x.startswith('mnt_id:')]
            if len(mids)!=1 or mids[0]<=0: raise ValueError('effective id')
            effective=mids[0]
        finally: close_fd_preserving(fd)
        lines=audit.read('/proc/self/mountinfo',524288).splitlines()
        if len(lines)>512: raise ValueError('mount count')
        matches=[];exact=[]
        for line in lines:
            left,right=line.split(' - ',1);fields=left.split();tail=right.split()
            if len(fields)<6 or len(tail)<3: raise ValueError('mount fields')
            mid=int(fields[0]);point=audit.mount_path(fields[4])
            if mid!=effective and point!=str(path): continue
            row={'id':mid,'parent':int(fields[1]),'device':fields[2],
                 'root':audit.mount_path(fields[3]),'path':point,
                 'filesystem':tail[0],'source':audit.mount_path(tail[1])}
            if mid<=0 or row['parent']<=0: raise ValueError('invalid id')
            if mid==effective: matches.append(row)
            if point==str(path): exact.append(row)
        if len(matches)!=1 or len(exact)>1 or exact and exact[0]!=matches[0]:
            raise ValueError('ambiguous or stacked graph')
        if matches[0]['device']!=str(os.major(info.st_dev))+':'+str(os.minor(info.st_dev)):
            raise ValueError('device mismatch')
        return {'effective':matches[0],'exact':exact,'stat':{'dev':info.st_dev,'ino':info.st_ino}}
    except BaseException as cause:
        error=ControlError('MOUNT_STATE_UNKNOWN','mount identity unavailable or ambiguous')
        error.operation='mount.identity'
        number=getattr(cause,'errno',None)
        if type(number) is int and -4096<=number<=4096: error.errno=number
        error.origin_kind=type(cause).__name__ if type(cause).__name__ in SAFE_KINDS else 'Exception'
        error.origin_incomplete=(type(cause).__name__ not in SAFE_KINDS or
            (number is not None and not (type(number) is int and -4096<=number<=4096)))
        error.secondary_errors=getattr(cause,'secondary_errors',[])
        raise error from cause

def mount_record_read():
    try:
        fd=os.open(MOUNT_RECORD,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
        try:
            info=os.fstat(fd)
            if (not stat.S_ISREG(info.st_mode) or info.st_uid!=0 or info.st_gid!=0
                or stat.S_IMODE(info.st_mode)!=0o600 or info.st_nlink!=1 or info.st_size>16384):
                raise ValueError('sidecar identity')
            raw=os.read(fd,16385)
            if len(raw)!=info.st_size: raise ValueError('sidecar size')
        finally: close_fd_preserving(fd)
        record=json.loads(raw)
        if (not isinstance(record,dict) or set(record)!={'version','token','entries'}
            or record['version']!=1 or record['token']!=os.environ['GITHUB_RUN_ID']+':'+os.environ['GITHUB_SHA']
            or not isinstance(record['entries'],dict) or not set(record['entries'])<={'external','scratch','fixture'}):
            raise ValueError('sidecar authority')
        return record
    except BaseException as cause:
        error=ControlError('MOUNT_PROVENANCE_UNKNOWN','mount provenance unavailable')
        error.operation='mount.provenance'
        number=getattr(cause,'errno',None)
        if type(number) is int and -4096<=number<=4096: error.errno=number
        error.origin_kind=type(cause).__name__ if type(cause).__name__ in SAFE_KINDS else 'Exception'
        error.origin_incomplete=(type(cause).__name__ not in SAFE_KINDS or
            (number is not None and not (type(number) is int and -4096<=number<=4096)))
        error.secondary_errors=getattr(cause,'secondary_errors',[])
        raise error from cause

def mount_record_write(record):
    data=json.dumps(record,sort_keys=True,separators=(',',':')).encode()
    if len(data)>16384: raise ControlError('MOUNT_PROVENANCE_BOUND','record size')
    tmp=Path(str(MOUNT_RECORD)+'.tmp')
    fd=os.open(tmp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
    try:
        os.fchmod(fd,0o600)
        if os.fstat(fd).st_uid!=0 or os.fstat(fd).st_gid!=0: raise ValueError('sidecar owner')
        BoundedWriter(fd,16384).write(data);os.fsync(fd)
    finally: close_fd_preserving(fd)
    os.replace(tmp,MOUNT_RECORD)
    directory=os.open('/mnt',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)
    try: os.fsync(directory)
    finally: close_fd_preserving(directory)

def mount_source_stat(path,mode):
    info=os.stat(path,follow_symlinks=False)
    if mode=='loop' and not stat.S_ISREG(info.st_mode) or mode=='bind' and not stat.S_ISDIR(info.st_mode):
        raise ControlError('MOUNT_SOURCE_UNKNOWN','source type')
    return {'dev':info.st_dev,'ino':info.st_ino}

def mount_validate(name,entry,view):
    expected={'external':(W,Path('/mnt/b38-'+PACKAGE+'-external.ext4'),'loop'),
              'scratch':(S,IMAGE,'loop'),
              'fixture':(S/'control-fixtures'/'mount',S/'work','bind')}
    if name not in expected or not isinstance(entry,dict) or set(entry)!={'state','target','underlying','mount','source','source_stat','mode'}:
        raise ControlError('MOUNT_PROVENANCE_UNKNOWN','invalid entry')
    target,source,mode=expected[name]
    if entry['target']!=str(target) or entry['source']!=str(source) or entry['mode']!=mode:
        raise ControlError('MOUNT_PROVENANCE_UNKNOWN','unexpected path')
    row=entry['mount'];underlying=entry['underlying']
    row_keys={'id','parent','device','root','path','filesystem','source'}
    if (not isinstance(row,dict) or set(row)!=row_keys or not isinstance(underlying,dict)
        or set(underlying)!=row_keys or row['path']!=str(target) or row['parent']!=underlying['id']
        or any(type(r[k]) is not int or r[k]<=0 for r in (row,underlying) for k in ('id','parent'))
        or any(not isinstance(r[k],str) for r in (row,underlying) for k in ('device','root','path','filesystem','source'))
        or not isinstance(entry['source_stat'],dict) or set(entry['source_stat'])!={'dev','ino'}
        or any(type(entry['source_stat'][k]) is not int for k in ('dev','ino'))
        or entry['source_stat']['dev']<0 or entry['source_stat']['ino']<=0 or row['id']==underlying['id']):
        raise ControlError('MOUNT_PROVENANCE_UNKNOWN','invalid mount identity')
    if entry['state'] in ('removed','released'):
        if entry['state']=='released' and mode!='loop': raise ControlError('MOUNT_PROVENANCE_UNKNOWN','invalid released state')
        if view['exact'] or view['effective']!=underlying:
            raise ControlError('MOUNT_STATE_UNKNOWN','removed identity changed')
        return
    if mount_source_stat(source,mode)!=entry['source_stat']:
        raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','source changed')
    if entry['state']!='mounted' or view['exact']!=[entry['mount']] or view['effective']!=entry['mount']:
        raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','owned mount not established')
    row=entry['mount'];underlying=entry['underlying']
    if row['parent']!=underlying['id'] or row['id']==underlying['id']:
        raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','mount parent changed')
    if mode=='loop':
        if row['filesystem']!='ext4' or row['root']!='/': raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','loop filesystem')
        backing=audit.read('/sys/dev/block/'+row['device']+'/loop/backing_file',4096).strip()
        if backing!=str(source): raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','loop backing')
    else:
        source_view=mount_graph(source);base=source_view['effective']
        try: relative=source.relative_to(Path(base['path']))
        except ValueError as cause: raise ControlError('MOUNT_SOURCE_UNKNOWN','source graph') from cause
        expected_root=str(Path(base['root'])/relative)
        if (view['stat']!=entry['source_stat'] or row['device']!=base['device'] or
            row['filesystem']!=base['filesystem'] or row['root']!=expected_root):
            raise ControlError('MOUNT_SOURCE_UNKNOWN','bind source identity')

def mount_begin(name,target,source,mode):
    record=mount_record_read();previous=record['entries'].get(name)
    if previous and previous.get('state')!='removed': raise ControlError('MOUNT_PREEXISTS','ownership unresolved')
    view=mount_graph(target)
    if view['exact']: raise ControlError('MOUNT_PREEXISTS','target mounted')
    if previous: mount_validate(name,previous,view)
    record['entries'][name]={'state':'pending','target':str(target),'underlying':view['effective'],
       'mount':None,'source':str(source),'source_stat':mount_source_stat(source,mode),'mode':mode}
    mount_record_write(record)

def mount_commit(name):
    record=mount_record_read();entry=record['entries'][name]
    if entry.get('state')!='pending': raise ControlError('MOUNT_PROVENANCE_UNKNOWN','no intent')
    view=mount_graph(Path(entry['target']))
    if len(view['exact'])!=1: raise ControlError('MOUNT_CREATION_UNPROVEN','no exact mount')
    prospective=dict(entry,state='mounted',mount=view['exact'][0])
    mount_validate(name,prospective,view)
    # Re-read graph before making the proof durable. A changed mount is unknown.
    if mount_graph(Path(entry['target']))!=view: raise ControlError('MOUNT_STATE_UNKNOWN','mount changed')
    record['entries'][name]=prospective;mount_record_write(record)
    return prospective

def validate_fixture_history(record,scratch,visible):
    fixture=record['entries'].get('fixture');point=S/'control-fixtures'/'mount'
    if fixture=={'state':'not_requested'}:
        if PACKAGE!='x03' or PHASE!='admission' or visible and os.path.lexists(point):
            raise ControlError('MOUNT_PROVENANCE_UNKNOWN','fixture sentinel mismatch')
        return
    keys={'state','target','underlying','mount','source','source_stat','mode'}
    rows={'id','parent','device','root','path','filesystem','source'}
    if not isinstance(fixture,dict) or set(fixture)!=keys:
        raise ControlError('MOUNT_PROVENANCE_UNKNOWN','fixture record missing')
    row=fixture['mount'];parent=fixture['underlying'];saved=fixture['source_stat']
    if (fixture['state']!='removed' or fixture['mode']!='bind' or fixture['target']!=str(point)
        or fixture['source']!=str(S/'work') or not isinstance(row,dict) or set(row)!=rows
        or not isinstance(parent,dict) or set(parent)!=rows or parent!=scratch['mount']
        or any(type(item[k]) is not int or item[k]<=0 for item in (row,parent) for k in ('id','parent'))
        or any(not isinstance(item[k],str) for item in (row,parent) for k in ('device','root','path','filesystem','source'))
        or row['path']!=str(point) or row['parent']!=parent['id'] or row['id']==parent['id']
        or row['device']!=parent['device'] or row['filesystem']!=parent['filesystem']
        or row['root']!=str(Path(parent['root'])/'work') or not isinstance(saved,dict)
        or set(saved)!={'dev','ino'} or type(saved['dev']) is not int or saved['dev']<0
        or type(saved['ino']) is not int or saved['ino']<=0
        or str(os.major(saved['dev']))+':'+str(os.minor(saved['dev']))!=row['device']):
        raise ControlError('MOUNT_PROVENANCE_UNKNOWN','fixture identity')
    if visible: mount_validate('fixture',fixture,mount_graph(point))


def mount_remove(name):
    record=mount_record_read();entry=record['entries'].get(name)
    if not entry: raise ControlError('MOUNT_PROVENANCE_UNKNOWN','no owned mount record')
    target=Path(entry['target']);view=mount_graph(target)
    mount_validate(name,entry,view)
    if name=='scratch':
        # A hidden fixture target after scratch release is supported only by its
        # previously committed record, never by ENOENT or a missing record.
        validate_fixture_history(record,entry,entry['state']=='mounted')
    if entry['state'] in ('removed','released'): return
    code,data=bounded(['umount',str(target)],5,4096,env={'PATH':'/usr/bin:/bin','LC_ALL':'C'})
    if code:
        error=ControlError('SCRATCH_RETAINED','owned unmount failed')
        error.exit_code=code;error.busy_message=b'target is busy' in data
        raise error
    after=mount_graph(target)
    if after['exact'] or after['effective']!=entry['underlying']:
        raise ControlError('MOUNT_STATE_UNKNOWN','unmount not established')
    entry['state']='removed';mount_record_write(record)

def mount_removed(name):
    record=mount_record_read();entry=record['entries'].get(name)
    if not entry or entry.get('state')!='removed': return False
    mount_validate(name,entry,mount_graph(Path(entry['target'])))
    return True

def owned_mount_identity(path):
    name='fixture' if Path(path)==S/'control-fixtures'/'mount' else 'scratch'
    record=mount_record_read();entry=record['entries'].get(name)
    if not entry or entry.get('state')!='mounted': raise ControlError('MOUNT_OWNERSHIP_UNKNOWN','no mounted record')
    mount_validate(name,entry,mount_graph(Path(path)))
    return entry

def make_control_bind(point):
    mount_begin('fixture',point,S/'work','bind')
    command('mount','--bind',str(S/'work'),str(point))
    # On mismatch, do not register the observed mount or unmount it.
    return mount_commit('fixture')

def parent_controls():
    # Inputs and targets are private owned fixtures, and use the production
    # collection and cleanup handlers, including their actual failure path.
    base=S/'control-fixtures';base.mkdir()
    source=base/'input';source.mkdir()
    (source/'a-audit.json').write_bytes(b'alpha')
    (source/'z-audit.json').write_bytes(b'omega')
    good=W/'out/control-collection-good';bad=W/'out/control-collection-bad'
    state={'primary':None,'secondary':[],'incomplete':False}
    if not collect_checked(state,source,good): control_mismatch('COLLECTION_DISCRIMINATOR','positive handler',state)
    if sorted(x['name'] for x in state['members'])!=['a-audit.json','z-audit.json'] or sorted(p.name for p in good.iterdir())!=['a-audit.json','z-audit.json']:
        control_mismatch('COLLECTION_DISCRIMINATOR','member set',state)
    if (good/'a-audit.json').read_bytes()!=b'alpha' or (good/'z-audit.json').read_bytes()!=b'omega':
        control_mismatch('COLLECTION_DISCRIMINATOR','retained bytes',state)
    bad.mkdir();(bad/'z-audit.json').mkdir()
    sentinel={'stop':'CONTROL_PRIMARY','stage':'control','operation':'seed'}
    negative={'primary':sentinel.copy(),'secondary':[],'incomplete':False}
    if collect_checked(negative,source,bad): control_mismatch('COLLECTION_DISCRIMINATOR','negative accepted',negative)
    if negative['primary']!=sentinel or not negative['incomplete'] or len(negative['secondary'])!=1:
        control_mismatch('COLLECTION_DISCRIMINATOR','disposition',negative)
    failure=negative['secondary'][0]
    if failure.get('stop')!='COLLECTION_TARGET_NOT_REGULAR' or failure.get('operation')!='copy.open' or failure.get('errno')!=17:
        control_mismatch('COLLECTION_DISCRIMINATOR','wrong production error',negative)
    if [x['name'] for x in negative['members']]!=['a-audit.json'] or (bad/'a-audit.json').read_bytes()!=b'alpha':
        control_mismatch('COLLECTION_DISCRIMINATOR','partial evidence',negative)
    RECORDS.append({'case':'collection-controls','positive':state,'negative':negative})
    point=base/'mount';point.mkdir()
    positive_identity=make_control_bind(point)
    clean={'primary':None,'secondary':[],'incomplete':False}
    if not finish_checked(clean,point,None,()) or not mount_removed('fixture'): control_mismatch('CLEANUP_DISCRIMINATOR','positive',clean)
    if not finish_checked(clean,point,None,()) or not mount_removed('fixture'): control_mismatch('CLEANUP_DISCRIMINATOR','idempotent',clean)
    negative_identity=make_control_bind(point)
    try:
        child=subprocess.Popen([sys.executable,'-B','-c','import sys,time;print("READY",flush=True);time.sleep(30)'],
                               cwd=point,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,
                               close_fds=True,start_new_session=True,env=control_environment())
    except BaseException as original:
        cleanup={'primary':None,'secondary':[],'incomplete':False}
        if not finish_checked(cleanup,point,None,()):
            original.secondary_errors=[cleanup['primary'],*cleanup['secondary']]
        raise
    failed={'primary':sentinel.copy(),'secondary':[],'incomplete':False}
    control_failure=None;cleanup_errors=[];child_ceased=False
    try:
        poll=selectors.DefaultSelector();poll.register(child.stdout,selectors.EVENT_READ)
        try:
            if not poll.select(3) or os.read(child.stdout.fileno(),32)!=b'READY\n' or child.poll() is not None:
                control_mismatch('CLEANUP_DISCRIMINATOR','child startup',failed)
        finally: poll.close()
        if finish_checked(failed,point,None,()): control_mismatch('CLEANUP_DISCRIMINATOR','busy mount accepted',failed)
        if failed['primary']!=sentinel or not failed['incomplete'] or len(failed['secondary'])!=1:
            control_mismatch('CLEANUP_DISCRIMINATOR','failure disposition',failed)
        entry=failed['secondary'][0]
        if entry.get('stop')!='CLEANUP_INCOMPLETE' or entry.get('operation')!='umount' or entry.get('exit_code')!=32 or not entry.get('busy_message') or owned_mount_identity(point)!=negative_identity or child.poll() is not None:
            control_mismatch('CLEANUP_DISCRIMINATOR','wrong production failure',failed)
    except BaseException as error:
        control_failure=error
    finally:
        try:
            if child.poll() is None: os.killpg(child.pid,signal.SIGKILL)
        except BaseException as error:
            cleanup_errors.append({'stop':type(error).__name__,'stage':'control.cleanup','operation':'child.kill'})
        try:
            child.wait(timeout=3)
            try: os.killpg(child.pid,0)
            except ProcessLookupError: child_ceased=True
        except BaseException as error:
            cleanup_errors.append({'stop':type(error).__name__,'stage':'control.cleanup','operation':'child.wait'})
        try: child.stdout.close()
        except BaseException as error:
            cleanup_errors.append({'stop':type(error).__name__,'stage':'control.cleanup','operation':'child.close'})
        if not child_ceased:
            cleanup_errors.append({'stop':'CONTROL_CHILD_CESSATION_UNPROVEN','stage':'control.cleanup','operation':'child.cease'})
        elif not finish_checked(failed,point,None,()):
            cleanup_errors.append(failed['secondary'][-1])
    if cleanup_errors:
        if control_failure is None: control_failure=ControlError('CLEANUP_CONTROL_TEARDOWN','incomplete')
        control_failure.secondary_errors=getattr(control_failure,'secondary_errors',[])+cleanup_errors
        control_failure.cessation_unproven=not child_ceased
    if control_failure is not None: raise control_failure
    if not mount_removed('fixture') or not finish_checked(failed,point,None,()) or not mount_removed('fixture'):
        control_mismatch('CLEANUP_DISCRIMINATOR','repeated cleanup',failed)
    RECORDS.append({'case':'cleanup-controls','positive':clean,'negative':failed,'child_status':child.returncode,
                    'child_group_absent':True,'positive_mount':positive_identity,'negative_mount':negative_identity,
                    'unmounted_after_child_cessation':True,'repeated_cleanup':True})

def package():
    write(W/'out/units.json',RECORDS)
    files=sorted(p for p in (W/'out').rglob('*') if p.is_file())
    if len(files)>64 or sum(p.stat().st_size for p in files)>16*MIB: raise ControlError('ARCHIVE_BOUND','members/bytes')
    index={str(p.relative_to(W/'out')):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    write(W/'out/sha256.json',index);files.append(W/'out/sha256.json')
    with preserving_close(zipfile.ZipFile(W/'evidence.zip','x',compression=zipfile.ZIP_STORED)) as z:
        for p in files:
            if p.is_symlink() or len(str(p.relative_to(W/'out'))) >128: raise ControlError('ARCHIVE_MEMBER',p.name)
            z.write(p,str(p.relative_to(W/'out')))
    # Close the ZIP writer before explicitly sealing the primary read-only.
    primary=os.open(W/'evidence.zip',os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
    try:
        info=os.fstat(primary)
        if not stat.S_ISREG(info.st_mode) or info.st_uid!=0 or info.st_nlink!=1:
            raise ControlError('ARCHIVE_PRIMARY_INVALID','identity')
        os.fchmod(primary,0o400);info=os.fstat(primary)
        if stat.S_IMODE(info.st_mode)!=0o400: raise ControlError('ARCHIVE_PRIMARY_INVALID','mode')
    finally: close_fd_preserving(primary)
    print('B38 inner ZIP SHA256 '+hashlib.sha256((W/'evidence.zip').read_bytes()).hexdigest())

def prepare_upload_copy(marker):
    # The original remains root-owned and read-only.  The worker receives only
    # a newly created, distinct, no-follow copy in its already confined /task.
    source=os.open(W/'evidence.zip',os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
    try:
        original=os.fstat(source)
        if (not stat.S_ISREG(original.st_mode) or original.st_uid!=0 or original.st_nlink!=1
            or stat.S_IMODE(original.st_mode)!=0o400 or original.st_size!=marker['bytes']
            or not 0<original.st_size<=17*MIB):
            raise ControlError('UPLOAD_COPY_INVALID','primary identity')
        target=os.open(S/'work/evidence.zip',os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
        try:
            info=os.fstat(target)
            if (not stat.S_ISREG(info.st_mode) or info.st_nlink!=1 or info.st_dev!=os.stat(S/'work').st_dev
                or (info.st_dev,info.st_ino)==(original.st_dev,original.st_ino)):
                raise ControlError('UPLOAD_COPY_INVALID','copy identity')
            digest=hashlib.sha256();seen=0;sink=BoundedWriter(target,17*MIB)
            while True:
                part=os.read(source,65536)
                if not part: break
                seen+=len(part)
                if seen>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','copy bound')
                digest.update(part);sink.write(part)
            if seen!=marker['bytes'] or digest.hexdigest()!=marker['sha256']:
                raise ControlError('UPLOAD_COPY_INVALID','primary changed')
            os.lseek(target,0,os.SEEK_SET);copy_digest=hashlib.sha256();copied=0
            while True:
                part=os.read(target,65536)
                if not part: break
                copied+=len(part)
                if copied>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','copy verification bound')
                copy_digest.update(part)
            if copied!=seen or copy_digest.hexdigest()!=marker['sha256']:
                raise ControlError('UPLOAD_COPY_INVALID','copied bytes')
            os.fchown(target,65534,65534);os.fchmod(target,0o600)
            info=os.fstat(target);after=os.fstat(source)
            if (info.st_uid!=65534 or info.st_gid!=65534 or stat.S_IMODE(info.st_mode)!=0o600
                or info.st_nlink!=1 or info.st_size!=seen or info.st_dev!=os.stat(S/'work').st_dev
                or (after.st_dev,after.st_ino,after.st_size)!=(original.st_dev,original.st_ino,original.st_size)
                or stat.S_IMODE(after.st_mode)!=0o400):
                raise ControlError('UPLOAD_COPY_INVALID','copy verification')
            print('B38_UPLOAD_COPY '+json.dumps({'primary_sha256':marker['sha256'],
                'copy_sha256':copy_digest.hexdigest(),'bytes':seen,'uid':info.st_uid,'gid':info.st_gid,
                'mode':'0600'},sort_keys=True),flush=True)
            return {'dev':info.st_dev,'ino':info.st_ino,'primary_sha256':marker['sha256'],'primary_bytes':seen}
        finally: close_fd_preserving(target)
    finally: close_fd_preserving(source)

def upload_members(stream):
    # Check all retained data through the same open copy; ZIP_STORED bounds the
    # bytes read. Compare the complete original name/digest map after append.
    stream.seek(0)
    with preserving_close(zipfile.ZipFile(stream,'r')) as archive:
        entries=archive.infolist()
        if len(entries)>66: raise ControlError('UPLOAD_AUDIT_BOUND','member count')
        result={};total=0
        for item in entries:
            name=item.filename
            if (not name or len(name)>128 or name.startswith('/') or '..' in Path(name).parts
                or name in result or item.is_dir() or item.compress_type!=zipfile.ZIP_STORED
                or item.file_size<0 or item.compress_size!=item.file_size):
                raise ControlError('UPLOAD_COPY_INVALID','member identity')
            total+=item.file_size
            if total>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','expanded members')
            h=hashlib.sha256();seen=0
            with preserving_close(archive.open(item,'r')) as member:
                for part in iter(lambda:member.read(65536),b''):
                    seen+=len(part)
                    if seen>item.file_size: raise ControlError('UPLOAD_COPY_INVALID','member size')
                    h.update(part)
            if seen!=item.file_size: raise ControlError('UPLOAD_COPY_INVALID','member size')
            result[name]=h.hexdigest()
        return result

def worker(role, case, kw):
    operation='permit'
    try:
        allowed={'PATH','HOME','LANG','LC_ALL','PYTHONDONTWRITEBYTECODE','GITHUB_ACTIONS','GITHUB_RUN_ATTEMPT','GITHUB_SHA','B38_PHASE','GITHUB_SERVER_URL','GITHUB_REPOSITORY','GITHUB_RUN_ID','ACTIONS_RESULTS_URL','ACTIONS_RUNTIME_TOKEN'}
        selected={k:v for k,v in os.environ.items() if k in allowed}
        os.environ.clear();os.environ.update(selected)
        until=time.monotonic()+4
        while not Path('/task/permit').exists():
            if time.monotonic()>until: raise ControlError('PERMIT_TIMEOUT',case)
            time.sleep(.01)
        if Path('/task/permit').read_text()!=case: raise ControlError('PERMIT_IDENTITY',case)
        operation='audit'
        proof=audit.audit(**kw);write(Path('/task')/(case+'-audit.json'),proof)
        if role=='control':
            operation='control'
            result=__import__('controls').run_case(case,**kw)
            if case in ('audit-valid','mount-negative','audit-missing','audit-wrong','audit-core-negative'):
                name=case+'-control-audit.json'
                write(Path('/task')/name,result)
                result={'audit_file':name}
        elif role=='metadata':
            operation='metadata'
            result=__import__('metadata').run(bounded)
        elif role=='upload':
            # Only the separate worker-owned copy is writable in /task.
            artifact=Path('/task/evidence.zip');audit_file=Path('/task/upload-audit.json')
            operation='audit_read'
            audit_fd=os.open(audit_file,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
            try:
                audit_info=os.fstat(audit_fd)
                if not stat.S_ISREG(audit_info.st_mode) or audit_info.st_size>MIB:
                    raise ControlError('UPLOAD_AUDIT_INVALID','type or bound')
                with preserving_close(os.fdopen(audit_fd,'rb',closefd=False)) as stream: audit_data=stream.read(MIB+1)
                if len(audit_data)!=audit_info.st_size: raise ControlError('UPLOAD_AUDIT_INVALID','changed')
            finally: close_fd_preserving(audit_fd)
            operation='archive_open'
            fd=os.open(artifact,os.O_RDWR|os.O_NOFOLLOW|os.O_CLOEXEC)
            try:
                info=os.fstat(fd)
                if (os.getuid()!=65534 or os.getgid()!=65534 or not stat.S_ISREG(info.st_mode)
                    or info.st_uid!=65534 or info.st_gid!=65534 or stat.S_IMODE(info.st_mode)!=0o600
                    or info.st_nlink!=1 or info.st_dev!=os.stat('/task').st_dev):
                    raise ControlError('UPLOAD_COPY_INVALID','owner, mode or identity')
                if info.st_size+len(audit_data)+65536>17*MIB:
                    raise ControlError('UPLOAD_AUDIT_BOUND','archive plus audit')
                with preserving_close(os.fdopen(fd,'r+b',closefd=False)) as stream:
                    before_hash=hashlib.sha256();stream.seek(0);before_bytes=0
                    for part in iter(lambda:stream.read(65536),b''):
                        before_bytes+=len(part)
                        if before_bytes>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','initial archive')
                        before_hash.update(part)
                    if before_bytes!=info.st_size: raise ControlError('UPLOAD_COPY_INVALID','initial bytes')
                    original_members=upload_members(stream)
                    if 'upload-audit.json' in original_members: raise ControlError('UPLOAD_AUDIT_DUPLICATE','member')
                    operation='archive_append';stream.seek(0)
                    with preserving_close(zipfile.ZipFile(stream,'a',compression=zipfile.ZIP_STORED)) as archive:
                        archive.writestr('upload-audit.json',audit_data)
                    stream.flush();info=os.fstat(fd)
                    if info.st_size>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','final archive')
                    operation='archive_verify';members=upload_members(stream)
                    audit_digest=hashlib.sha256(audit_data).hexdigest()
                    if members.pop('upload-audit.json',None)!=audit_digest or members!=original_members:
                        raise ControlError('UPLOAD_COPY_INVALID','member verification')
                    stream.seek(0);h=hashlib.sha256();seen=0
                    for block in iter(lambda:stream.read(65536),b''):
                        seen+=len(block)
                        if seen>17*MIB: raise ControlError('UPLOAD_AUDIT_BOUND','hash bound')
                        h.update(block)
                    if seen!=info.st_size or h.hexdigest()==before_hash.hexdigest():
                        raise ControlError('UPLOAD_COPY_INVALID','final identity')
                operation='object_record'
                upload_object={'archive_sha256':h.hexdigest(),'bytes':seen,
                    'upload_audit_sha256':audit_digest,'dev':info.st_dev,'ino':info.st_ino,
                    'primary_sha256':before_hash.hexdigest(),'primary_bytes':before_bytes}
                # Provider output is discarded; only this fixed worker record
                # reaches the parent. The pinned vendor code is not claimed to
                # be hostile-code-isolated from the same-UID wrapper.
                print('B38_UPLOAD_OBJECT '+json.dumps(upload_object,sort_keys=True,separators=(',',':')),flush=True)
            finally: close_fd_preserving(fd)
            operation='adapter_prepare'
            provider_names=('PATH','HOME','LANG','LC_ALL','GITHUB_SERVER_URL','GITHUB_REPOSITORY',
                            'GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','ACTIONS_RESULTS_URL','ACTIONS_RUNTIME_TOKEN')
            provider_env={name:os.environ[name] for name in provider_names if name in os.environ}
            provider_env.update({'INPUT_NAME':'b38-'+PACKAGE+'-'+os.environ['GITHUB_SHA'],
                'INPUT_PATH':'/task/evidence.zip','INPUT_RETENTION-DAYS':'30',
                'INPUT_COMPRESSION-LEVEL':'0','INPUT_IF-NO-FILES-FOUND':'error','INPUT_OVERWRITE':'false','INPUT_INCLUDE-HIDDEN-FILES':'false',
                'GITHUB_OUTPUT':'/task/artifact-output','RUNNER_TEMP':'/task/tmp','TMPDIR':'/task/tmp'})
            Path('/task/artifact-output').touch(exist_ok=False)
            operation='provider_execute'
            # Do not mix untrusted provider text with the worker status channel.
            # The bounded artifact-output fields are separately projected.
            result=subprocess.run(['/node','/src/experiments/bed-038/'+PACKAGE+'/artifact/vendor.js'],
                stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,close_fds=True,env=provider_env)
            if result.returncode:
                error=ControlError('UPLOAD_PROVIDER_FAILURE','provider exit')
                error.status=result.returncode if -255<=result.returncode<=255 else None
                raise error
            return
        else: raise ControlError('PHASE',role)
    except Exception as error:
        first=safe_failure(error,operation)
        if role=='upload':
            try: print('B38_UPLOAD_FAILURE '+json.dumps(first,sort_keys=True,separators=(',',':')),flush=True)
            except Exception as status_error: status_write_fallback(case,first,status_error)
        else:
            try: write_status(case,first['code'],failure=first)
            except Exception as status_error: status_write_fallback(case,first,status_error)
        sys.exit(1)
    try: rejected=write_status(case,'OK',result=result)
    except Exception as status_error:
        first=safe_failure(status_error);first['code']='STATUS_WRITE_FAILED'
        status_write_fallback(case,first,status_error)
        sys.exit(1)
    if rejected: sys.exit(1)

def run():
    group=Path('/sys/fs/cgroup')/Path('/proc/self/cgroup').read_text().split('0:',1)[1].strip().lstrip('/')
    actual={n:(group/n).read_text().strip() for n in ('memory.max','memory.swap.max','cpu.max')}
    if actual['memory.max']!=str(MEMORY) or actual['memory.swap.max']!='0' or len(CPUS)!=2:
        raise ControlError('PARENT_ENVELOPE',actual)
    quota,period=map(int,actual['cpu.max'].split())
    if not 0<quota<=2*period: raise ControlError('PARENT_ENVELOPE',actual)
    ancestor={n:(group.parent/n).read_text().strip() for n in ('memory.max','memory.swap.max','cpu.max')}
    if ancestor!=actual: raise ControlError('SLICE_ENVELOPE',ancestor)
    identity={}
    for name in ('python3','systemd-run','systemctl','unshare','setpriv','mount','umount','mkfs.ext4','fallocate','busctl','prlimit'):
        located=__import__('shutil').which(name)
        if located is None:
            error=ControlError('HOST_TOOL_MISSING',name);error.member=name;raise error
        path=Path(located).resolve()
        if path.stat().st_size>32*MIB:
            error=ControlError('HOST_TOOL_BOUND',name);error.member=name
            error.observed=path.stat().st_size;error.limit=32*MIB;raise error
        identity[name]={'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
    for name in ('launch.py','audit.py','controls.py','isolate.sh','launch-manifest.json','metadata.py','artifact/action.yml','artifact/bridge.js','artifact/supervisor.js'):
        path=Path(__file__).parent/name;identity[name]={'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
    # Bind bootstrap and teardown policy too. Exclude only the reviewed later
    # phase/qualification/deadline settings; every other workflow byte is bound.
    import re
    workflow=(W/'source/.github/workflows'/('bed-038-'+PACKAGE+'.yml')).read_text()
    workflow=re.sub(r'(?m)^    if: \$\{\{ (?:true|false) \}\}$','    if: PHASE_GUARD',workflow)
    workflow=re.sub(r'(?m)^      B38_PHASE: (?:disabled|preflight|admission)$','      B38_PHASE: PHASE',workflow)
    workflow=re.sub(r'(?m)^      B38_PREFLIGHT: .*$', '      B38_PREFLIGHT: QUALIFICATION',workflow)
    workflow=re.sub(r'(?m)^    timeout-minutes: (?:10|20)$','    timeout-minutes: PHASE_DEADLINE',workflow)
    workflow=workflow.replace('--kill-after=5s 295s systemd-run','PARENT_OUTER systemd-run').replace('--kill-after=5s 775s systemd-run','PARENT_OUTER systemd-run')
    workflow=workflow.replace('RuntimeMaxSec=285','RuntimeMaxSec=PHASE').replace('RuntimeMaxSec=765','RuntimeMaxSec=PHASE')
    identity['workflow.yml']={'normalized_sha256':hashlib.sha256(workflow.encode()).hexdigest()}
    mechanism=hashlib.sha256(json.dumps({n:identity[n] for n in identity if n.endswith(('.py','.sh','.json','.yml','.js'))},sort_keys=True).encode()).hexdigest()
    runtime=hashlib.sha256(json.dumps({'host':{k:v for k,v in identity.items() if k in ('python3','systemd-run','systemctl','unshare','setpriv','mount','umount','mkfs.ext4','fallocate','busctl','prlimit')},'image':os.environ.get('ImageVersion'),'kernel':os.uname().release},sort_keys=True).encode()).hexdigest()
    parent=props('b38-'+PACKAGE+'-parent.service')
    if duration(parent['RuntimeMaxUSec'])!=(765 if PHASE=='admission' else 285) or parent['KillMode']!='control-group' or parent['SendSIGKILL']!='yes': raise ControlError('PARENT_DEADLINE',parent)
    write(W/'out/runtime-identities.json',{'tools':identity,'parent':actual,'service':parent,'cpus':CPUS,'mechanism':mechanism,'runtime':runtime})
    if PHASE!='preflight':
        raise ControlError('PHASE','disabled or unauthorized')
    if os.path.lexists(IMAGE) or os.path.lexists(S): raise ControlError('STORAGE_NOT_EMPTY',str(S))
    command('fallocate','-l',str(TOTAL-128*MIB),str(IMAGE));command('mkfs.ext4','-q','-m','1',str(IMAGE))
    S.mkdir();mount_begin('scratch',S,IMAGE,'loop')
    command('mount','-o','loop,nodev,nosuid',str(IMAGE),str(S));mount_commit('scratch')
    for name in ['work','root','injected']: (S/name).mkdir()
    for name in ['usr','lib','lib64','etc','src','sys','proc','dev','run','tmp','task','surprise','var','var/lib','var/lib/dpkg']:
        (S/'root'/name).mkdir()
    (S/'root/bin').symlink_to('usr/bin');(S/'root/node').touch()
    for name in ('out','tmp'):
        (S/'work'/name).mkdir();os.chown(S/'work'/name,65534,65534)
    os.chown(S/'work',65534,65534)
    os.chown(S/'injected',65534,65534)
    if PHASE=='preflight':
        active_deadline=time.monotonic()+240
        for case,expected in EXPECTATIONS.items():
            if expected.get('executor')=='parent': continue
            if time.monotonic()+20>active_deadline: raise ControlError('PREFLIGHT_ACTIVE_DEADLINE',case)
            diagnostic=run_unit(case,timeout=3 if case=='child-timeout' else 8,injected=case=='mount-negative')
            if case=='output-bound' and (S/'work/control-output-bound.bin').stat().st_size!=4096: raise ControlError('OUTPUT_DISCRIMINATOR','size')
            if diagnostic!=expected['diagnostic']:
                found=next((r.get('worker_status') for r in reversed(RECORDS) if r.get('worker_status',{}).get('case')==case),None)
                failure=found.get('failure') if isinstance(found,dict) else None
                error=ControlError(failure['code'] if isinstance(failure,dict) else 'CONTROL_MISMATCH',case)
                error.worker_status=found;raise error
            if 'status_bytes' in expected or 'attempted_status_bytes' in expected:
                retained=next((r for r in reversed(RECORDS) if r.get('worker_status',{}).get('case')==case),None)
                if retained is None or ('status_bytes' in expected and retained.get('status_bytes')!=expected['status_bytes']) or ('attempted_status_bytes' in expected and retained['worker_status'].get('attempted_bytes')!=expected['attempted_status_bytes']):
                    raise ControlError('STATUS_EXERCISE_MISMATCH',case)
        if time.monotonic()+52>active_deadline: raise ControlError('PREFLIGHT_ACTIVE_DEADLINE','parent-controls')
        parent_controls()
        if time.monotonic()+52>active_deadline: raise ControlError('PREFLIGHT_ACTIVE_DEADLINE','metadata')
        if run_unit('metadata','metadata',timeout=40)!='OK':
            raise ControlError('METADATA_STOP','no build or probe authorized')

def finish(scratch=None, image=None, units=None):
    scratch=S if scratch is None else scratch
    units=UNITS if units is None else units
    problems=[];cause_secondaries=[]
    for unit in units:
        try:
            if unit not in COMPLETED and unit not in WITNESSES:
                raise ControlError('UNIT_CESSATION_UNPROVEN',unit)
            cease(unit,None,require_valid_termination=False)
        except BaseException as error:
            problems.append({'stop':getattr(error,'code','UNCLASSIFIED_FAILURE'),'operation':'unit.cleanup',
                             'kind':type(error).__name__,'errno':getattr(error,'errno',None)})
            cause_secondaries.extend(getattr(error,'secondary_errors',[]))
    if problems:
        error=ControlError('CLEANUP_INCOMPLETE',problems)
        error.safe_causes=problems
        error.secondary_errors=cause_secondaries
        error.operation=problems[0]['operation']
        raise error
    try:
        name='fixture' if scratch==S/'control-fixtures'/'mount' else 'scratch' if scratch==S else None
        if name is None: raise ControlError('MOUNT_SCOPE','unapproved cleanup target')
        mount_remove(name)
        if image is not None:
            record=mount_record_read();entry=record['entries'][name]
            if str(image)!=entry['source']: raise ControlError('MOUNT_SOURCE_UNKNOWN','image path')
            if entry['state']=='removed':
                # Missing/replaced image is not proof of our own unlink. A crash
                # between unlink and the durable released record is incomplete.
                if mount_source_stat(image,'loop')!=entry['source_stat']:
                    raise ControlError('MOUNT_SOURCE_UNKNOWN','image identity')
                image.unlink()
                entry['state']='released';mount_record_write(record)
            elif entry['state']!='released': raise ControlError('MOUNT_STATE_UNKNOWN','image release not recorded')
            elif os.path.lexists(image): raise ControlError('MOUNT_STATE_UNKNOWN','released image replaced')
    except BaseException as error:
        problems.append({'stop':getattr(error,'code',type(error).__name__),'operation':'umount',
                         'exit_code':getattr(error,'exit_code',None),'busy_message':getattr(error,'busy_message',False),
                         'kind':getattr(error,'origin_kind',type(error).__name__),'errno':getattr(error,'errno',None)})
        cause_secondaries.extend(getattr(error,'secondary_errors',[]))
    if problems:
        error=ControlError('CLEANUP_INCOMPLETE',problems)
        error.safe_causes=problems
        error.secondary_errors=cause_secondaries
        error.operation=problems[0]['operation']
        error.exit_code=problems[0].get('exit_code')
        error.busy_message=problems[0].get('busy_message')
        raise error


def finish_checked(state, scratch=None, image=None, units=None):
    try:
        finish(scratch,image,units)
        return True
    except BaseException as error:
        record_failure(state,error,'cleanup','finish')
        return False

if __name__=='__main__':
    if os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('GITHUB_RUN_ATTEMPT')!='1':
        raise SystemExit('ACTIONS_ONLY; reruns forbidden')
    PHASE=os.environ.get('B38_PHASE','disabled');CPUS=sorted(os.sched_getaffinity(0))[:2]
    HOST_NS={n:os.readlink('/proc/self/ns/'+n) for n in ('mnt','ipc','pid','net')}
    if len(sys.argv)>2: worker(sys.argv[1],sys.argv[2],json.loads(sys.argv[3]));sys.exit()
    mode=sys.argv[1]
    state={'primary':None,'secondary':[],'incomplete':False,'members':[],
           'cessation':'unproven','scope':'controlled units in this parent only'}
    transport={'provider_output_omitted':True,'provider_output_reason':'untrusted_output_not_recorded'}
    ready_verified=False

    def absorb(error, stage, operation):
        current=record_failure(state,error,stage,operation)
        # Only fixed, non-secret fields are admitted to the companion log.
        def safe_value(value, depth=0):
            if depth>4: return '<omitted>'
            if value is None or type(value) in (int,bool): return value
            if isinstance(value,str):
                if len(value)<=256 and value.isascii() and all(c.isalnum() or c in ' ._:/-@' for c in value): return value
                return '<omitted>'
            if isinstance(value,list): return [safe_value(v,depth+1) for v in value[:64]]
            if isinstance(value,dict):
                keys={'primary','secondary','incomplete','members','stop','stage','operation','kind','errno',
                      'exit_code','busy_message','member','name','bytes','sha256','code','case','diagnostic',
                      'original_diagnostic','attempted_bytes','status_write','observed','limit','failure','safe_origin',
                      'safe_secondary','detail_omitted','classification_incomplete','nested_fields_omitted','status','causes','origin_kind','precedence_unknown'}
                return {k:safe_value(v,depth+1) for k,v in value.items() if k in keys}
            return '<omitted>'
        setup_failure=getattr(error,'setup_failure',None)
        if isinstance(setup_failure,dict) and valid_failure(setup_failure): current['setup_failure']=safe_value(setup_failure)
        if getattr(error,'precedence_unknown',False): current['precedence_unknown']=True;state['incomplete']=True
        fallback=getattr(error,'worker_fallback',None)
        if isinstance(fallback,dict): current['worker_status_fallback']=safe_value(fallback)
        worker_status=getattr(error,'worker_status',None)
        if isinstance(worker_status,dict):
            current['worker_status']=safe_value(worker_status)
            current['worker_status_detail_omitted']='detail' in worker_status
        control_state=getattr(error,'control_state',None)
        if isinstance(control_state,dict): current['control_state']=safe_value(control_state)
        detail=getattr(error,'detail',None)
        if isinstance(detail,dict) and getattr(error,'code',None) in ('PARENT_ENVELOPE','SLICE_ENVELOPE','UNIT_POLICY','UNIT_DEADLINE','UNIT_TERMINAL_UNPROVEN','UNIT_RESULT_UNPROVEN','UNIT_IDENTITY_LOST'):
            allowed=set(PROPS)|{'memory.max','memory.swap.max','cpu.max','cpuset.cpus.effective','pids.max'}
            observed={}
            for k,v in detail.items():
                if k in allowed and (type(v) in (int,bool) or isinstance(v,str) and len(v)<=256 and v.isascii() and all(c.isalnum() or c in ' ._:/-@' for c in v)):
                    observed[k]=v
            current['observed_fields']=observed
            current['observed_fields_omitted']=len(detail)-len(observed)
        if getattr(error,'cessation_unproven',False):
            state['cessation']='unproven'
            state['cessation_unproven']=True

    def safe_value_for_unit(value):
        if isinstance(value,dict) and 'failure' in value and valid_failure(value['failure']):
            return {'case':value.get('case'),'diagnostic':value.get('diagnostic'),'failure':value['failure']}
        if isinstance(value,dict) and 'primary' in value and all(valid_failure(value.get(n)) for n in ('primary','status_write')):
            return {'case':value.get('case'),'primary':value['primary'],'status_write':value['status_write']}
        if isinstance(value,dict) and value.get('diagnostic') in ('OK','STATUS_OVERSIZE'):
            result={k:value[k] for k in ('case','diagnostic','original_diagnostic','attempted_bytes') if k in value}
            return result
        state['incomplete']=True
        return {'incomplete':True,'detail_omitted':True}

    def unit_projection():
        result=[]
        for item in RECORDS:
            if 'unit' not in item: continue
            row={k:item[k] for k in ('unit','cgroup_empty','cleanup_complete','slice_events') if k in item}
            for field in ('worker_status','worker_status_fallback'):
                if field in item: row[field]=safe_value_for_unit(item[field])
            for name in ('terminal','slice'):
                observed=item.get(name)
                if isinstance(observed,dict):
                    allowed=('Id','InvocationID','Result','ExecMainCode','ExecMainStatus','ActiveState','SubState','ControlGroup')
                    row[name]={k:observed[k] for k in allowed if k in observed}
            effective=item.get('effective')
            if isinstance(effective,dict):
                names=('RuntimeMaxUSec','TimeoutStartUSec','TimeoutStopUSec','KillMode','SendSIGKILL',
                       'Restart','KillSignal','FinalKillSignal','NoNewPrivileges','PrivateDevices','PrivateIPC',
                       'PrivateMounts','PrivateNetwork','ProtectSystem','RootDirectory',
                       'RestrictAddressFamilies','RestrictNamespaces','CapabilityBoundingSet',
                       'AmbientCapabilities','LimitCORE','LimitNOFILE','TasksMax','SystemCallFilter',
                       'ExitType','RemainAfterExit')
                row['effective']={k:effective[k] for k in names if k in effective and
                    isinstance(effective[k],str) and len(effective[k])<=4096}
                row['effective_omitted']=[k for k in names if k not in row['effective']]
                if row['effective_omitted']: state['incomplete']=True
            result.append(row)
        return result

    def archive_identity():
        path=W/'evidence.zip'
        fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
        try:
            info=os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or info.st_size>17*MIB: raise ControlError('NO_ARCHIVE','type or size')
            digest=hashlib.sha256();seen=0
            while True:
                part=os.read(fd,65536)
                if not part: break
                seen+=len(part)
                if seen>17*MIB: raise ControlError('NO_ARCHIVE','read bound')
                digest.update(part)
            if seen!=info.st_size: raise ControlError('NO_ARCHIVE','changed')
            return {'run_id':os.environ['GITHUB_RUN_ID'],'commit':os.environ['GITHUB_SHA'],
                    'attempt':os.environ['GITHUB_RUN_ATTEMPT'],'bytes':seen,'sha256':digest.hexdigest()}
        finally: close_fd_preserving(fd)

    def commit_ready():
        marker=W/'out/archive-ready.json';temporary=W/'out/archive-ready.tmp'
        if marker.exists() or marker.is_symlink(): raise ControlError('ARCHIVE_READY_EXISTS','marker')
        value=archive_identity();value['prior_cessation']='confirmed_for_controlled_units'
        write(temporary,value)
        os.replace(temporary,marker)

    def require_ready():
        path=W/'out/archive-ready.json'
        fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
        try:
            info=os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or info.st_size>4096: raise ControlError('ARCHIVE_READY_INVALID','type or size')
            data=os.read(fd,4097)
            if len(data)!=info.st_size: raise ControlError('ARCHIVE_READY_INVALID','changed')
        finally: close_fd_preserving(fd)
        try: marker=json.loads(data)
        except (ValueError,UnicodeDecodeError) as error: raise ControlError('ARCHIVE_READY_INVALID','JSON') from error
        expected=archive_identity();expected['prior_cessation']='confirmed_for_controlled_units'
        if marker!=expected: raise ControlError('ARCHIVE_READY_INVALID','identity or hash')
        return marker

    def emit(label, value):
        value=dict(value)
        if 'units' in value:
            value['units']=unit_projection()
            value['incomplete']=state['incomplete']
            value['unit_fields_omitted']='Full records are retained in bounded evidence files; validated failure status is projected.'
        data=json.dumps(value,sort_keys=True,ensure_ascii=True).encode()
        ceiling=256*1024 if label=='B38_PARENT_STATE' else 2*MIB
        if len(data)>ceiling:
            # Never report an omitted inventory as a complete record.
            state['incomplete']=True
            value={'primary':state['primary'],'secondary':state['secondary'],
                   'incomplete':True,'cessation':state['cessation'],
                   'diagnostic':'PARENT_EVIDENCE_OVERSIZE','original_bytes':len(data),
                   'omitted_unit_records':len(RECORDS),'omitted_members':max(0,len(state['members'])-32),
                   'partial_members':state['members'][:32]}
            data=json.dumps(value,sort_keys=True,ensure_ascii=True).encode()
            if len(data)>ceiling: raise ControlError('PARENT_EVIDENCE_OVERSIZE','summary')
        text=data.decode('ascii')
        for offset in range(0,len(text),4096):
            print(label+'_PART '+str(offset//4096+1)+' '+json.dumps(text[offset:offset+4096]))
        print(label+'_SHA256 '+hashlib.sha256(data).hexdigest()+' bytes='+str(len(data)),flush=True)

    try:
        if mode not in ('run','upload'): raise ControlError('PHASE',mode)
        lease=Path('/mnt/b38-'+PACKAGE+'-lease')
        expected=(os.environ['GITHUB_RUN_ID']+':'+os.environ['GITHUB_SHA']+'\nslice_start_attempted').encode('ascii')
        if len(expected)>128: raise ControlError('UNIT_INVENTORY_LEASE','ownership')
        try:
            lease_fd=os.open(lease,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
            try:
                info=os.fstat(lease_fd)
                if (not stat.S_ISREG(info.st_mode) or info.st_uid!=0 or info.st_nlink!=1
                    or stat.S_IMODE(info.st_mode)!=0o600 or info.st_size!=len(expected)
                    or os.read(lease_fd,129)!=expected):
                    raise ControlError('UNIT_INVENTORY_LEASE','ownership')
            finally: close_fd_preserving(lease_fd)
        except OSError as error:
            raise ControlError('UNIT_INVENTORY_LEASE','ownership') from error
        INVENTORY={'version':1,'package':PACKAGE,'run_id':os.environ['GITHUB_RUN_ID'],
                   'commit':os.environ['GITHUB_SHA'],'mode':mode,'workers':{},'parent_closed':False}
        inventory_parent_identity(mode)
        inventory_write()
        if mode=='run': run()
        elif mode=='upload':
            ready=require_ready()
            ready_verified=True
            try: UPLOAD_EXPECTED=prepare_upload_copy(ready)
            except Exception as cause:
                failure=safe_failure(cause,'archive_open')
                failure['stage']='upload_copy'
                if failure['code']=='WORKER_FAILURE': failure['code']='UPLOAD_COPY_SETUP_FAILURE'
                error=ControlError(failure['code'],'copy setup')
                error.operation='upload.copy';error.setup_failure=failure
                if failure['errno'] is not None: error.errno=failure['errno']
                raise error from cause
            require_ready()
            run_unit('upload','upload',timeout=60)
        else: raise ControlError('PHASE',mode)
    except BaseException as error:
        absorb(error,mode,'run' if mode=='run' else 'upload')
    finally:
        # A unit absent from our captured witness records is unknown, not ceased.
        for unit in UNITS:
            try:
                if unit not in COMPLETED and unit not in WITNESSES:
                    raise ControlError('UNIT_CESSATION_UNPROVEN',unit)
                cease(unit,None,require_valid_termination=False)
            except BaseException as error:
                state['cessation_unproven']=True
                absorb(error,'finalization','cease')
        if not state.get('cessation_unproven'):
            state['cessation']='confirmed_for_controlled_units'
        try:
            if INVENTORY is None: raise ControlError('UNIT_INVENTORY_UNAVAILABLE','parent')
            INVENTORY['parent_closed']=all(row['stage']=='cleaned' and row['cleanup_complete'] and
                  row['client_reaped'] and row['terminal'] and row['slice_events'] is not None
                  for row in INVENTORY['workers'].values())
            inventory_write()
            if not INVENTORY['parent_closed']:
                raise ControlError('UNIT_INVENTORY_INCOMPLETE','parent')
        except BaseException as error:
            state['cessation_unproven']=True;state['incomplete']=True
            state['cessation']='unproven'
            absorb(error,'finalization','inventory_close')
        if mode=='run':
            if state['cessation']=='confirmed_for_controlled_units':
                collect_checked(state)
            else:
                state['incomplete']=True
                state['collection']='skipped_unstable_sources'
            try:
                state['phase']=PHASE
                state['units']=RECORDS
                write(W/'out/disposition.json',state)
                if state['cessation']=='confirmed_for_controlled_units':
                    package()
                    commit_ready()
                else:
                    state['packaging']='skipped_unproven_cessation'
            except BaseException as error:
                absorb(error,'finalization','disposition_or_package')
            # This bounded companion survives a failed scratch path or uploader.
            emit('B38_PARENT_STATE',state)
        if mode=='upload':
            try:
                if state['cessation']=='confirmed_for_controlled_units':
                    data=UPLOAD_OBJECT
                    if not isinstance(data,dict):
                        state['incomplete']=True;transport['upload_object_status']='missing'
                    else:
                        transport['upload_object']=data
                        copy_fd=os.open(S/'work/evidence.zip',os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
                        try:
                            info=os.fstat(copy_fd)
                            if not stat.S_ISREG(info.st_mode) or (info.st_dev,info.st_ino)!=(data['dev'],data['ino']) or info.st_size!=data['bytes']:
                                raise ControlError('UPLOAD_OBJECT_CHANGED','identity')
                            digest=hashlib.sha256();count=0
                            while True:
                                block=os.read(copy_fd,65536)
                                if not block: break
                                count+=len(block)
                                if count>17*MIB: raise ControlError('UPLOAD_OBJECT_CHANGED','bound')
                                digest.update(block)
                            if count!=data['bytes'] or digest.hexdigest()!=data['archive_sha256']:
                                raise ControlError('UPLOAD_OBJECT_CHANGED','digest')
                        finally: close_fd_preserving(copy_fd)
                    output=S/'work/artifact-output'
                    if output.exists():
                        if output.is_symlink() or output.stat().st_size>65536: raise ControlError('TRANSPORT_RECORD_BOUND','artifact output')
                        raw=output.read_bytes();rows=raw.decode('utf-8').splitlines();safe={}
                        for i,line in enumerate(rows):
                            if '<<' not in line or i+2>=len(rows): continue
                            key,delimiter=line.split('<<',1);value=rows[i+1]
                            if rows[i+2]!=delimiter: continue
                            if key=='artifact-id' and value.isascii() and value.isdigit() and len(value)<=32: safe[key]=value
                            if key=='artifact-digest' and __import__('re').fullmatch(r'(?:sha256:)?[0-9a-f]{64}',value): safe[key]=value
                        transport['artifact_outputs']=safe
                        transport['artifact_output_sha256']=hashlib.sha256(raw).hexdigest()
                        transport['artifact_output_omissions']='Non-ID/digest output omitted from logs'
                        if 'artifact-id' not in safe or 'artifact-digest' not in safe: state['incomplete']=True
                    else: state['incomplete']=True;transport['artifact_output_status']='missing'
                else:
                    state['incomplete']=True
                    transport['collection']='skipped_unstable_sources'
                transport['units']=RECORDS
            except BaseException as error:
                absorb(error,'upload','transport_collection')
            # The upload parent is a new process: its local empty UNITS cannot
            # prove that a previous parent ceased. Revalidate the committed
            # prior-cessation marker and archive bytes before any scratch cleanup.
            if ready_verified and state['cessation']=='confirmed_for_controlled_units':
                try:
                    require_ready()
                    finish_checked(state,S,IMAGE,UNITS)
                except BaseException as error:
                    absorb(error,'cleanup','archive_ready_revalidation')
            else:
                state['incomplete']=True
                state['scratch_cleanup']='deferred_to_source_independent_teardown'
            transport.update(primary=state['primary'],secondary=state['secondary'],
                             cessation=state['cessation'],incomplete=state['incomplete'])
            emit('B38_TRANSPORT',transport)
    if state['primary'] or state['secondary'] or state.get('cessation_unproven') or state['incomplete']:
        raise SystemExit(1)
