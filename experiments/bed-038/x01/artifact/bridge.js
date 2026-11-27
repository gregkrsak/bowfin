'use strict';
const cp = require('node:child_process');
const fs = require('node:fs');
const crypto = require('node:crypto');
const packageName = 'x01';
const slice = 'b38-' + packageName + '.slice';
const adapterSlice = 'b38-' + packageName + '-adapter.slice';
const holdSlice = 'b38-' + packageName + '-hold.slice';
const uploadSlice = 'b38-' + packageName + '-upload-parent.slice';
const observerSlice = 'b38-' + packageName + '-observer.slice';
const observerScope = 'b38-' + packageName + '-observer.scope';
const scope = 'b38-' + packageName + '-adapter.scope';
const memory = 6442450944;
// Owned diagnostic codes are carried as data, never inferred from exception text.
class AdapterStop extends Error {
  constructor(code) { super(); this.safeCode = code; }
}
function stop(code) { throw new AdapterStop(code); }
function safeKind(error) {
  if (error instanceof AdapterStop) return 'AdapterStop';
  if (error instanceof TypeError) return 'TypeError';
  if (error instanceof RangeError) return 'RangeError';
  return 'Error';
}
function safeErrno(error) {
  return error != null && Number.isInteger(error.errno) && error.errno >= -4096 && error.errno <= 4096
    ? error.errno : null;
}
function read(path) {
  const descriptor = fs.openSync(path, 'r');
  let primary = null;
  try {
    const buffer = Buffer.alloc(4097);
    const size = fs.readSync(descriptor, buffer, 0, buffer.length, null);
    if (size > 4096) stop('ADAPTER_READ_BOUND');
    return buffer.subarray(0, size).toString().trim();
  } catch (error) { primary = error; throw error; }
  finally {
    try { fs.closeSync(descriptor); }
    catch (error) {
      if (!primary) throw error;
      // Preserve both failures even if logging would itself fail.
      primary.safeSecondary = {identity:'secondary',
        stage, operation:'read_close', code:'ADAPTER_CLOSE_FAILURE', kind:safeKind(error),
        errno:safeErrno(error), incomplete:true, qualification:'provisional'};
    }
  }
}
function control(arguments_, timeoutMs=4000) {
  const budget=Math.min(4000,Math.floor(timeoutMs));
  if (!Number.isFinite(budget) || budget<600) stop('ADAPTER_PRELUDE_TIMEOUT');
  return cp.execFileSync('/usr/bin/timeout', ['--kill-after=0.250s',
    ((budget-500)/1000).toFixed(3)+'s','/usr/bin/sudo',...arguments_], {
    timeout: budget, maxBuffer: 16384, encoding: 'utf8',
    env: {PATH:'/usr/sbin:/usr/bin:/sbin:/bin'},
    stdio: ['ignore', 'pipe', 'pipe']
  });
}
// Setup reporting has an explicit completion boundary for each output stream.
// A timeout treats the stream as unavailable; no raw stream error is reported.
function writeSetupRecord(stream, value) {
  return new Promise(resolve => {
    let settled=false;
    const finish=result=>{
      if (settled) return;
      settled=true; clearTimeout(timer); resolve(result);
    };
    const timer=setTimeout(()=>{
      finish({reason:'timeout',kind:'Error',errno:null});
      // Do not keep a failed output pipe alive indefinitely.
      try { stream.destroy(); } catch (_) {}
    },1000);
    try {
      stream.write(value, error=>finish(error
        ? {reason:'write_failure',kind:safeKind(error),errno:safeErrno(error)} : null));
    } catch (error) {
      finish({reason:'write_failure',kind:safeKind(error),errno:safeErrno(error)});
    }
  });
}
// Prevent an asynchronous stderr error from escaping the bounded fallback.
process.stderr.on('error',()=>{ process.exitCode=1; });
async function reportSetupFailure(failure) {
  process.exitCode=1;
  let record=JSON.stringify(failure);
  if (Buffer.byteLength(record,'utf8')>16384) {
    failure={...failure,observed:{},observations_omitted:true,incomplete:true};
    record=JSON.stringify(failure);
  }
  const primaryLine='B38_ADAPTER_SETUP_STOP '+record+'\n';
  const outputFailure=await writeSetupRecord(process.stdout,primaryLine);
  if (!outputFailure) { process.exit(1); return; } // detailed record completed
  const reporting={identity:'secondary',stage:'setup_reporting',operation:'stdout_write',
    code:'ADAPTER_SETUP_REPORT_FAILURE',kind:outputFailure.kind,errno:outputFailure.errno,
    reason:outputFailure.reason,incomplete:true,qualification:'provisional'};
  const fallback={...failure,secondary:[...failure.secondary,reporting],incomplete:true};
  // Its record carries both the first failure and the failed primary transport.
  await writeSetupRecord(process.stderr,'B38_ADAPTER_SETUP_STOP '+JSON.stringify(fallback)+'\n');
  // Both output attempts have completed or timed out. This also bounds exit
  // if a timed-out stream could not be destroyed.
  process.exit(1);
}

let setupFailure=null;
let stage = "scope";
const safeObservation = {};
let companionOutputError=null;
process.stdout.on('error', error=>{ if (!companionOutputError) companionOutputError=error; process.exitCode=1; });
function writeCompanion(value) {
  return new Promise(resolve=>{
    try { process.stdout.write(value,error=>resolve(error || companionOutputError)); }
    catch (error) { resolve(error); }
  });
}
async function relayChunks(chunks) {
  for (const chunk of chunks) { const error=await writeCompanion(chunk); if (error) return error; }
  return null;
}

// Fixed bounded root-owned provenance is pre-created by bootstrap on W.
// This writer is called only after this process has attached to its slice.
let observerIdentity=null;
try { observerIdentity=JSON.parse(process.env.B38_OBSERVER_IDENTITY || ''); } catch (_) {}
const adapterInventory = {version:1,package:packageName,run_id:process.env.GITHUB_RUN_ID,
  commit:process.env.GITHUB_SHA,
  scope:{id:scope,invocation_id:null,control_group:null,job_id:null,state:'intent'},
  observer_scope:observerIdentity && observerIdentity.scope,
  owned_slices:{adapter:{id:adapterSlice,invocation_id:null,control_group:null,state:'intent'},
    hold:{id:holdSlice,invocation_id:null,control_group:null,state:'intent'},
    upload:{id:uploadSlice,invocation_id:null,control_group:null,state:'intent'},
    observer:observerIdentity && observerIdentity.slice},
  adapter_pid:null,adapter_start_ticks:null,
  upload_client:{pid:null,start_ticks:null,status:null,signal:null,state:'not_started'},
  adapter_complete:false,incomplete:true};
const inventoryWriter = `
import json,os,pathlib,re,stat,sys
p,w,run,commit=sys.argv[1:]
if p not in ('x01','x03') or not re.fullmatch(r'[0-9]{1,32}',run) or not re.fullmatch(r'[0-9a-f]{40}',commit): raise SystemExit(1)
lease='/mnt/b38-'+p+'-lease'
lfd=os.open(lease,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
try:
 i=os.fstat(lfd); content=os.read(lfd,129)
 if not stat.S_ISREG(i.st_mode) or i.st_uid!=0 or i.st_nlink!=1 or i.st_mode&0o022 or content.decode()!=run+':'+commit+'\\nslice_start_attempted': raise SystemExit(1)
finally: os.close(lfd)
d=os.open(w,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC)
try:
 info=os.fstat(d)
 if not stat.S_ISDIR(info.st_mode) or info.st_uid!=0: raise SystemExit(1)
 old=os.open('inventory-adapter.json',os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=d)
 try:
  i=os.fstat(old)
  if not stat.S_ISREG(i.st_mode) or i.st_uid!=0 or i.st_nlink!=1 or stat.S_IMODE(i.st_mode)!=0o600 or i.st_size>16384: raise SystemExit(1)
  data=os.read(old,16385)
  if len(data)!=i.st_size: raise SystemExit(1)
  previous=json.loads(data)
 finally: os.close(old)
 raw=sys.stdin.buffer.read(16385)
 if len(raw)>16384: raise SystemExit(1)
 current=json.loads(raw)
 if not isinstance(previous,dict) or not isinstance(current,dict): raise SystemExit(1)
 if any(previous.get(k)!=v or current.get(k)!=v for k,v in [('version',1),('package',p),('run_id',run),('commit',commit)]): raise SystemExit(1)
 for role,suffix in (('adapter','adapter'),('hold','hold'),('upload','upload-parent'),('observer','observer')):
  name='b38-'+p+'-'+suffix+'.slice'
  for obj in (previous,current):
   rows=obj.get('owned_slices'); row=rows.get(role) if isinstance(rows,dict) else None
   if not isinstance(row,dict) or set(row)!={'id','invocation_id','control_group','state'} or row.get('id')!=name: raise SystemExit(1)
   if row['state']=='intent':
    if row['invocation_id'] is not None or row['control_group'] is not None: raise SystemExit(1)
   elif row['state']=='observed':
    if not isinstance(row['invocation_id'],str) or not re.fullmatch('[0-9a-f]{32}',row['invocation_id']): raise SystemExit(1)
    group=row['control_group']
    if not isinstance(group,str) or not group.startswith('/') or '..' in group or not group.endswith('/b38-'+p+'.slice/'+name): raise SystemExit(1)
   else: raise SystemExit(1)
  if previous['owned_slices'][role]['state']=='observed' and previous['owned_slices'][role]!=current['owned_slices'][role]: raise SystemExit(1)
 old=previous.get('observer_scope'); new=current.get('observer_scope'); name='b38-'+p+'-observer.scope'
 if not isinstance(old,dict) or not isinstance(new,dict) or set(old)!={'id','invocation_id','control_group','job_id','state'} or set(new)!=set(old) or old.get('id')!=name or new.get('id')!=name: raise SystemExit(1)
 if old['state']=='intent':
  if any(old[k] is not None for k in ('invocation_id','control_group','job_id')): raise SystemExit(1)
 elif old['state']=='observed':
  if old!=new: raise SystemExit(1)
 else: raise SystemExit(1)
 if (new['state']!='observed' or not isinstance(new['invocation_id'],str) or not re.fullmatch('[0-9a-f]{32}',new['invocation_id']) or
     not isinstance(new['job_id'],str) or not re.fullmatch('[0-9]{1,20}',new['job_id']) or
     not isinstance(new['control_group'],str) or not new['control_group'].endswith('/b38-'+p+'.slice/b38-'+p+'-observer.slice/'+name)): raise SystemExit(1)
 fd=os.open('.inventory-adapter.tmp',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600,dir_fd=d)
 try:
  os.fchmod(fd,0o600)
  view=memoryview(raw)
  while view:
   n=os.write(fd,view)
   if n<=0: raise SystemExit(1)
   view=view[n:]
  os.fsync(fd)
 finally: os.close(fd)
 os.replace('.inventory-adapter.tmp','inventory-adapter.json',src_dir_fd=d,dst_dir_fd=d)
 os.fsync(d)
finally: os.close(d)
`;
function storeInventory() {
  const payload=JSON.stringify(adapterInventory);
  if (Buffer.byteLength(payload)>16384) stop('ADAPTER_INVENTORY_BOUND');
  const r=cp.spawnSync('/usr/bin/timeout',['--kill-after=0.250s','3.500s',
    '/usr/bin/sudo','-n','/usr/bin/python3','-c',inventoryWriter,
    packageName,process.env.GITHUB_WORKSPACE,process.env.GITHUB_RUN_ID,process.env.GITHUB_SHA],
    {input:payload,timeout:4000,maxBuffer:4096,encoding:'utf8',env:{},stdio:['pipe','ignore','ignore']});
  if (r.error || r.status!==0) stop('ADAPTER_INVENTORY_FAILURE');
}
function pidStart(pid) {
  if (!Number.isSafeInteger(pid) || pid<1) stop('ADAPTER_CLIENT_IDENTITY');
  const value=read('/proc/'+String(pid)+'/stat');
  const pos=value.lastIndexOf(')');const fields=value.slice(pos+2).split(/\s+/);
  if (pos<0 || fields.length<20 || !/^[0-9]{1,20}$/.test(fields[19]) || /^0+$/.test(fields[19])) stop('ADAPTER_CLIENT_IDENTITY');
  return fields[19];
}
function observeOwnedSlice(name) {
  const deadline=process.hrtime.bigint()+4000000000n;
  const pause=new Int32Array(new SharedArrayBuffer(4));
  while (process.hrtime.bigint()<deadline) {
    const left=Number((deadline-process.hrtime.bigint())/1000000n);
    if (left<600) break;
    try {
      const raw=control(['/usr/bin/systemctl','show',name,'--property=Id,InvocationID,ControlGroup,ActiveState'],Math.min(left,800));
      const p=Object.fromEntries(raw.trim().split('\n').map(x=>x.split('=')));
      if (p.Id!==name || !/^[0-9a-f]{32}$/.test(p.InvocationID || '') ||
          !p.ControlGroup || !p.ControlGroup.startsWith('/') || p.ControlGroup.includes('..') ||
          !p.ControlGroup.endsWith('/'+slice+'/'+name)) stop('ADAPTER_TERMINATION');
      if (p.ActiveState==='active') return {id:name,invocation_id:p.InvocationID,control_group:p.ControlGroup,state:'observed'};
      if (p.ActiveState!=='activating') stop('ADAPTER_TERMINATION');
    } catch (error) { if (error instanceof AdapterStop && error.safeCode==='ADAPTER_TERMINATION') throw error; }
    Atomics.wait(pause,0,0,50);
  }
  stop('ADAPTER_TERMINATION');
}

// Finite resource-allocation prelude. No task data collection before attachment.
// StartTransientUnit/PIDs moves this existing Node process and all its threads.
try {
  if (!observerIdentity || !observerIdentity.scope || !observerIdentity.slice ||
      observerIdentity.scope.id!==observerScope || observerIdentity.slice.id!==observerSlice) stop('ADAPTER_TERMINATION');
  const scopeDeadline=process.hrtime.bigint()+4000000000n;
  function remainingScopeMs() {
    const ns=scopeDeadline-process.hrtime.bigint();
    return ns>0n ? Number(ns/1000000n) : 0;
  }
  // Keep an independently inspectable empty cgroup after the scope exits.
  control(['/usr/bin/busctl','--system','call','org.freedesktop.systemd1',
    '/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager',
    'StartTransientUnit','ssa(sv)a(sa(sv))',adapterSlice,'fail','1',
    'StopWhenUnneeded','b','false','0'],remainingScopeMs());
  const scopeJob=control(['/usr/bin/busctl', '--system', 'call', 'org.freedesktop.systemd1',
    '/org/freedesktop/systemd1', 'org.freedesktop.systemd1.Manager',
    'StartTransientUnit', 'ssa(sv)a(sa(sv))', scope, 'fail', '12',
    'PIDs', 'au', '1', String(process.pid), 'Slice', 's', adapterSlice,
    'MemoryMax', 't', String(memory), 'MemorySwapMax', 't', '0',
    'CPUQuotaPerSecUSec', 't', '2000000', 'AllowedCPUs', 'ay', '1', '3',
    'TasksMax', 't', '64', 'RuntimeMaxUSec', 't', '130000000',
    'TimeoutStopUSec', 't', '2000000', 'KillMode', 's', 'control-group',
    'SendSIGKILL', 'b', 'true', 'FinalKillSignal', 'i', '9', '0'],remainingScopeMs());
  const jobMatch=/^o \"\/org\/freedesktop\/systemd1\/job\/([0-9]+)\"\s*$/.exec(scopeJob);
  if (!jobMatch || jobMatch[1].length>20) stop('ADAPTER_CLIENT_IDENTITY');
  adapterInventory.scope.job_id=jobMatch[1];
  // A loaded unit's After= dependency retains the scope's terminal state.
  // The hold unit is outside the adapter slice and survives scope-wide kills.
  control(['/usr/bin/busctl','--system','call','org.freedesktop.systemd1',
    '/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager',
    'StartTransientUnit','ssa(sv)a(sa(sv))',holdSlice,'fail','2',
    'After','as','2',scope,observerScope,'StopWhenUnneeded','b','false','0'],remainingScopeMs());
  let holdReady=false;
  const holdPause=new Int32Array(new SharedArrayBuffer(4));
  while (remainingScopeMs()>=600) {
    try {
      const holdProperties=control(['/usr/bin/systemctl','show',holdSlice,
        '--property=Id,ActiveState,After'],Math.min(800,remainingScopeMs()));
      const hold=Object.fromEntries(holdProperties.trim().split('\n').map(x=>x.split('=')));
      if (hold.Id!==holdSlice || !String(hold.After || '').split(/\s+/).includes(scope) ||
          !String(hold.After || '').split(/\s+/).includes(observerScope)) stop('ADAPTER_TERMINATION');
      if (hold.ActiveState==='active') { holdReady=true; break; }
      if (hold.ActiveState!=='activating') stop('ADAPTER_TERMINATION');
    } catch (error) { if (error instanceof AdapterStop && error.safeCode==='ADAPTER_TERMINATION') throw error; }
    Atomics.wait(holdPause,0,0,Math.min(50,Math.max(1,remainingScopeMs())));
  }
  if (!holdReady) stop('ADAPTER_TERMINATION');
  stage = 'core_limit';
  if (remainingScopeMs()<600) stop('ADAPTER_PRELUDE_TIMEOUT');
  control(['/usr/bin/prlimit','--pid',String(process.pid),'--core=0:0'],remainingScopeMs());
  const corePattern=read('/proc/sys/kernel/core_pattern');
  const coreLine=read('/proc/self/limits').split('\n').find(x=>x.startsWith('Max core file size'));
  safeObservation.core_pipe = corePattern.startsWith('|');
  safeObservation.core_limit_zero = Boolean(coreLine && /^Max core file size\s+0\s+0\s+bytes$/.test(coreLine));
  if (!coreLine || !/^Max core file size\s+0\s+0\s+bytes$/.test(coreLine)) stop('ADAPTER_CORE_POLICY');
  stage = 'cgroup_identity';
  // StartTransientUnit returns a queued job; allow bounded creation latency.
  // Only actual attachment permits the following resource inspection or work.
  const pause=new Int32Array(new SharedArrayBuffer(4));
  let relative=null, sliceGroup=null, scopeLookupError=null;
  while (remainingScopeMs()>0) {
    let candidate='';
    if (remainingScopeMs()<600) break;
    try {
      candidate=control(['/usr/bin/systemctl','show',adapterSlice,'--property=ControlGroup','--value'],
        Math.min(800,remainingScopeMs())).trim();
    } catch (error) { scopeLookupError=error; }
    if (candidate && (!candidate.startsWith('/') || candidate.includes('..'))) stop('ADAPTER_CGROUP');
    const row=read('/proc/self/cgroup').split('\n').find(x=>x.startsWith('0:'));
    if (!row || row.includes('..')) stop('ADAPTER_CGROUP');
    if (candidate && row.slice(2)===candidate+'/'+scope) {sliceGroup=candidate;relative=row.slice(2);break;}
    Atomics.wait(pause,0,0,Math.min(50,Math.max(1,remainingScopeMs())));
  }
  if (relative===null) {
    const failure=new AdapterStop('ADAPTER_CGROUP');
    if (scopeLookupError) failure.safeSecondary={identity:'secondary',stage:'cgroup_identity',
      operation:'scope_lookup',code:'ADAPTER_SCOPE_LOOKUP_FAILURE',kind:safeKind(scopeLookupError),
      errno:safeErrno(scopeLookupError),incomplete:true,qualification:'provisional'};
    throw failure;
  }
  stage = 'resource_limits';
  const directory = '/sys/fs/cgroup' + relative;
  const observed = {};
  for (const name of ['memory.max', 'memory.swap.max', 'cpu.max', 'cpuset.cpus.effective', 'pids.max']) {
    observed[name] = read(directory + '/' + name);
  }
  for (const [name, value] of Object.entries(observed)) {
    if (/^(?:[0-9, .-]{1,128}|max)$/.test(value)) safeObservation[name] = value;
  }
  const quota = observed['cpu.max'].split(' ').map(Number);
  if (observed['memory.max'] !== String(memory) || observed['memory.swap.max'] !== '0' ||
      !['0-1', '0,1'].includes(observed['cpuset.cpus.effective']) ||
      observed['pids.max'] !== '64' || quota.length !== 2 ||
      !(quota[0] > 0 && quota[1] > 0 && quota[0] <= 2 * quota[1])) stop('ADAPTER_ENVELOPE');
  stage = 'termination_properties';
  const properties = control(['/usr/bin/systemctl', 'show', scope,
    '--property=Id,InvocationID,RuntimeMaxUSec,TimeoutStopUSec,KillMode,SendSIGKILL,FinalKillSignal,ControlGroup,Slice']);
  const effective = Object.fromEntries(properties.trim().split('\n').map(x => x.split('=')));
  if (!['2min 10s', '130s'].includes(effective.RuntimeMaxUSec) ||
      effective.TimeoutStopUSec !== '2s' || effective.KillMode !== 'control-group' ||
      effective.SendSIGKILL !== 'yes' || !['9', 'SIGKILL'].includes(effective.FinalKillSignal) ||
      effective.ControlGroup !== relative || effective.Slice !== adapterSlice || effective.Id !== scope ||
      !/^[0-9a-f]{32}$/.test(effective.InvocationID)) stop('ADAPTER_TERMINATION');
  stage='inventory_scope';
  adapterInventory.scope.invocation_id=effective.InvocationID;
  adapterInventory.scope.control_group=relative;adapterInventory.scope.state='observed';
  adapterInventory.adapter_pid=process.pid;adapterInventory.adapter_start_ticks=pidStart(process.pid);
  adapterInventory.owned_slices.adapter=observeOwnedSlice(adapterSlice);
  adapterInventory.owned_slices.hold=observeOwnedSlice(holdSlice);
  const observedObserver=observeOwnedSlice(observerSlice);
  if (JSON.stringify(observedObserver)!==JSON.stringify(adapterInventory.owned_slices.observer)) stop('ADAPTER_TERMINATION');
  const observerProps=control(['/usr/bin/systemctl','show',observerScope,'--property=Id,InvocationID,ControlGroup,Slice,ActiveState']);
  const observer=Object.fromEntries(observerProps.trim().split('\n').map(x=>x.split('=')));
  if (observer.Id!==observerScope || observer.InvocationID!==adapterInventory.observer_scope.invocation_id ||
      observer.ControlGroup!==adapterInventory.observer_scope.control_group || observer.Slice!==observerSlice ||
      observer.ActiveState!=='active') stop('ADAPTER_TERMINATION');
  storeInventory();
  stage = 'node_identity';
  const executable = fs.realpathSync(process.execPath);
  const size = fs.statSync(executable).size;
  if (size > 256 * 1024 * 1024) stop('ADAPTER_NODE_BOUND');
  const sha256 = crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex');
  // Emit only verified facts, not host paths, core-pattern text or provider data.
  console.log('B38_ADAPTER_CONTEXT ' + JSON.stringify({observed:safeObservation,
    termination_verified:true, cgroup_verified:true, bytes:size, sha256,
    node:/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(process.version) ? process.version : null}));
} catch (error) {
  // Only fixed, locally owned labels and bounded numeric facts cross this boundary.
  const allowedCodes = new Set(['ADAPTER_READ_BOUND', 'ADAPTER_CORE_POLICY',
    'ADAPTER_CGROUP', 'ADAPTER_ENVELOPE', 'ADAPTER_TERMINATION', 'ADAPTER_NODE_BOUND',
    'ADAPTER_INVENTORY_BOUND','ADAPTER_INVENTORY_FAILURE','ADAPTER_CLIENT_IDENTITY','ADAPTER_PRELUDE_TIMEOUT']);
  const operations = {scope:'attach_scope', core_limit:'verify_core_policy',
    cgroup_identity:'verify_cgroup', resource_limits:'verify_limits',
    termination_properties:'verify_termination', node_identity:'verify_node', inventory_scope:'inventory_scope'};
  const code = error instanceof AdapterStop && allowedCodes.has(error.safeCode)
    ? error.safeCode : 'ADAPTER_OPERATION_FAILURE';
  const status = error != null && Number.isInteger(error.status) && error.status >= 0 && error.status <= 255
    ? error.status : null;
  const failure = {identity:'primary', stage,
    operation:operations[stage], code, kind:safeKind(error), errno:safeErrno(error), status,
    observed:safeObservation, incomplete:true, qualification:'provisional',
    secondary:error && error.safeSecondary ? [error.safeSecondary] : []};
  setupFailure=failure;
}

if (setupFailure !== null) {
  // This branch cannot start the upload; completion or timeout precedes exit.
  process.exitCode=1;
  reportSetupFailure(setupFailure).catch(()=>{ process.exitCode=1; });
} else {
process.env.B38_NODE = process.execPath;
const environmentNames = ['GITHUB_ACTIONS', 'GITHUB_WORKSPACE', 'GITHUB_SHA',
  'GITHUB_RUN_ATTEMPT', 'GITHUB_RUN_ID', 'GITHUB_SERVER_URL', 'GITHUB_REPOSITORY',
  'B38_PHASE', 'B38_NODE', 'ACTIONS_RUNTIME_TOKEN', 'ACTIONS_RESULTS_URL'];
const source = require('node:path').resolve(__dirname, '../launch.py');
async function relayUpload() {
  let child=null, original=null, result=null, completion=null, relayed=false;
  try {
    stage='inventory_upload';adapterInventory.upload_client.state='intent';storeInventory();
    // Preserve a separately inspectable cgroup after the upload parent exits.
    control(['/usr/bin/busctl','--system','call','org.freedesktop.systemd1',
      '/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager',
      'StartTransientUnit','ssa(sv)a(sa(sv))',uploadSlice,'fail','1',
      'StopWhenUnneeded','b','false','0']);
    adapterInventory.owned_slices.upload=observeOwnedSlice(uploadSlice);
    storeInventory();
    // Only transport credentials and the execution identity needed by the
    // upload parent and worker are forwarded to this subprocess.
    const uploadEnvironment={PATH:'/usr/sbin:/usr/bin:/sbin:/bin'};
    for (const name of environmentNames) {
      if (Object.prototype.hasOwnProperty.call(process.env,name)) uploadEnvironment[name]=process.env[name];
    }
    child=cp.spawn('/usr/bin/sudo',[
      '--preserve-env=' + environmentNames.join(','), '/usr/bin/systemd-run',
      '--quiet','--pipe','--wait','--slice='+uploadSlice,'--unit=b38-'+packageName+'-upload-parent',
      '-p','ExitType=cgroup','-p','RemainAfterExit=yes',
      '-p','RuntimeMaxSec=110','-p','TimeoutStartSec=5','-p','TimeoutStopSec=2',
      '-p','KillMode=control-group','-p','LimitCORE=0','-p','CPUQuota=200%',
      '-p','AllowedCPUs=0,1','-p','MemoryMax='+String(memory),'-p','MemorySwapMax=0',
      ...environmentNames.map(name=>'--setenv='+name),'/usr/bin/python3','-B',source,'upload'
    ],{env:uploadEnvironment,stdio:['ignore','pipe','pipe']});
    let chunks=[],outBytes=0,errBytes=0,overflow=false,timedOut=false,spawnError=null,streamError=null,killError=null;
    function terminateClient() {
      try { if (!child.kill('SIGKILL') && !killError) killError=new AdapterStop('ADAPTER_CLIENT_KILL_FAILURE'); }
      catch (error) { if (!killError) killError=error; }
    }
    completion=new Promise(resolve=>{
      child.on('error',error=>{spawnError=error;});
      child.stdout.on('data',chunk=>{
        outBytes+=chunk.length;
        if (outBytes>4*1024*1024) {overflow=true;terminateClient();}
        else chunks.push(chunk);
      });
      child.stderr.on('data',chunk=>{
        errBytes+=chunk.length;
        if (errBytes>4*1024*1024) {overflow=true;terminateClient();}
      });
      child.stdout.on('error',error=>{if (!streamError) streamError=error;terminateClient();});
      child.stderr.on('error',error=>{if (!streamError) streamError=error;terminateClient();});
      const timer=setTimeout(()=>{timedOut=true;terminateClient();},120000);
      child.on('close',(status,signal)=>{clearTimeout(timer);resolve({status,signal,spawnError,streamError,killError,overflow,timedOut,chunks});});
    });
    if (!Number.isSafeInteger(child.pid) || child.pid<1) stop('ADAPTER_CLIENT_IDENTITY');
    adapterInventory.upload_client.pid=child.pid;
    adapterInventory.upload_client.state='running';
    try { adapterInventory.upload_client.start_ticks=pidStart(child.pid); }
    catch (error) {
      adapterInventory.upload_client.state='identity_unknown';
      try { storeInventory(); } catch (_) { /* Both failures remain incomplete. */ }
      throw error;
    }
    storeInventory();
    result=await completion;
    adapterInventory.upload_client.status=Number.isInteger(result.status) && result.status>=0 && result.status<=255 ? result.status : null;
    adapterInventory.upload_client.signal=result.signal===null ? null :
      (['SIGTERM','SIGKILL','SIGABRT','SIGSEGV','SIGPIPE','SIGINT'].includes(result.signal)?result.signal:'OTHER');
    adapterInventory.upload_client.state='exited';
    adapterInventory.incomplete=Boolean(result.spawnError || result.streamError || result.killError || result.overflow || result.timedOut || result.status!==0);
    // Completion records control flow only; actual scope/client settlement is
    // checked separately by source-independent final teardown.
    adapterInventory.adapter_complete=true;
    relayed=true;
    original=await relayChunks(result.chunks);
    if (original) adapterInventory.incomplete=true;
    try { storeInventory(); }
    catch (error) {
      if (!adapterInventory.incomplete) throw error;
      result.inventory_error=error;
    }
    if (adapterInventory.incomplete || original) {
      const primary=result.spawnError || result.streamError || original;
      const failed={identity:'secondary_or_unknown',stage:'transport',operation:'upload_parent',
        code:'ADAPTER_TRANSPORT_FAILURE',kind:primary?safeKind(primary):'Error',
        errno:primary?safeErrno(primary):null,status:adapterInventory.upload_client.status,
        incomplete:true,qualification:'provisional',
        output_overflow:result.overflow,timeout:result.timedOut,
        secondary:[...(result.inventory_error?[{identity:'secondary',stage:'transport',operation:'inventory_write',
          code:'ADAPTER_INVENTORY_FAILURE',kind:safeKind(result.inventory_error),
          errno:safeErrno(result.inventory_error),incomplete:true}]:[]),
          ...(result.killError?[{identity:'secondary',stage:'transport',operation:'client_kill',
            code:'ADAPTER_CLIENT_KILL_FAILURE',kind:safeKind(result.killError),errno:safeErrno(result.killError),incomplete:true}]:[])]};
      try { console.log('B38_ADAPTER_TRANSPORT_STOP '+JSON.stringify(failed)); }
      catch (_) { try { process.stderr.write('B38_ADAPTER_TRANSPORT_STOP '+JSON.stringify(failed)+'\n'); } catch (_) {} }
      process.exitCode=1;
    }
  } catch (error) {
    original=error;
    const secondary=[];
    if (original.safeSecondary) secondary.push(original.safeSecondary);
    try { if (child && child.exitCode===null) child.kill('SIGKILL'); }
    catch (killError) {
      secondary.push({identity:'secondary',stage:'transport',operation:'client_kill',
        code:'ADAPTER_CLIENT_KILL_FAILURE',kind:safeKind(killError),errno:safeErrno(killError),incomplete:true});
    }
    if (completion) {
      try {
        const settled=await completion;
        if (!relayed && settled.chunks.length) {
          const relayError=await relayChunks(settled.chunks);
          if (relayError) throw relayError;
        }
      } catch (relayError) {
        secondary.push({identity:'secondary',stage:'transport',operation:'companion_write',
          code:'ADAPTER_COMPANION_WRITE_FAILURE',kind:safeKind(relayError),errno:safeErrno(relayError),incomplete:true});
      }
    }
    adapterInventory.incomplete=true;
    try { storeInventory(); }
    catch (inventoryError) {
      secondary.push({identity:'secondary',stage:'transport',operation:'inventory_write',
        code:'ADAPTER_INVENTORY_FAILURE',kind:safeKind(inventoryError),errno:safeErrno(inventoryError),incomplete:true});
    }
    const failed={identity:'secondary_or_unknown',stage:'transport',operation:'adapter_inventory',
      code:error instanceof AdapterStop ? error.safeCode : 'ADAPTER_TRANSPORT_FAILURE',
      kind:safeKind(error),errno:safeErrno(error),incomplete:true,qualification:'provisional',
      secondary};
    try { console.log('B38_ADAPTER_TRANSPORT_STOP '+JSON.stringify(failed)); }
    catch (_) { try { process.stderr.write('B38_ADAPTER_TRANSPORT_STOP '+JSON.stringify(failed)+'\n'); } catch (_) {} }
    process.exitCode=1;
  }
}
relayUpload().catch(()=>{
  try { process.stderr.write('B38_ADAPTER_TRANSPORT_STOP {"code":"ADAPTER_UNHANDLED","incomplete":true}\n'); } catch (_) {}
  process.exitCode=1;
});
}
