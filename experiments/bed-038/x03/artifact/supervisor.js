'use strict';
// Runner-owned observer. The adapter itself still attaches to its own scope.
// This process is deliberately outside that scope so a scope-wide kill cannot
// erase the only direct-child wait observation.
const cp = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const packageName = path.basename(path.dirname(__dirname));
const output = process.env.GITHUB_OUTPUT;
const run = process.env.GITHUB_RUN_ID;
const commit = process.env.GITHUB_SHA;
const deadlineMs = 140000;
const killWaitMs = 3000;
const inventoryReader = `
import json,os,re,stat,sys
w,p,run,commit,kind=sys.argv[1:]
if p not in ('x01','x03') or not re.fullmatch('[0-9]{1,32}',run) or not re.fullmatch('[0-9a-f]{40}',commit): raise SystemExit(1)
lease='/mnt/b38-'+p+'-lease'
fd=os.open(lease,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC)
try:
 st=os.fstat(fd); data=os.read(fd,129)
 if not stat.S_ISREG(st.st_mode) or st.st_uid!=0 or st.st_nlink!=1 or st.st_mode&0o022 or data.decode()!=run+':'+commit+'\\nslice_start_attempted': raise SystemExit(1)
finally: os.close(fd)
d=os.open(w,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC)
try:
 if os.fstat(d).st_uid!=0: raise SystemExit(1)
 name='inventory-adapter.json' if kind=='adapter' else 'inventory-upload-parent.json'
 fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=d)
 try:
  st=os.fstat(fd)
  if not stat.S_ISREG(st.st_mode) or st.st_uid!=0 or st.st_nlink!=1 or stat.S_IMODE(st.st_mode)!=0o600 or st.st_size>16384: raise SystemExit(1)
  raw=os.read(fd,16385)
  if len(raw)!=st.st_size: raise SystemExit(1)
 finally: os.close(fd)
finally: os.close(d)
obj=json.loads(raw)
if not isinstance(obj,dict) or any(obj.get(k)!=v for k,v in [('version',1),('package',p),('run_id',run),('commit',commit)]): raise SystemExit(1)
print(json.dumps(obj,separators=(',',':')))
`;

// Keep inherited hard limits intact so the exec wrapper can restore the
// worker's inherited soft limits. The limits apply to this observer, not as a
// substitute for the adapter's independently checked cgroup envelope.
const limitRules = [
  ['cpu','Max cpu time','20'], ['as','Max address space',packageName === 'x03' ? '4294967296' : '6442450944'],
  ['nofile','Max open files','128'], ['nproc','Max processes','128'],
  ['core','Max core file size','0']
];
function limitSnapshot() {
  const lines = fs.readFileSync('/proc/self/limits','utf8').split('\n');
  return limitRules.map(([option,label,target]) => {
    const line = lines.find(x => x.startsWith(label + ' '));
    if (!line) throw new Error('limit');
    const m = /^\s*(unlimited|[0-9]+)\s+(unlimited|[0-9]+)(?:\s|$)/.exec(line.slice(label.length));
    if (!m) throw new Error('limit');
    const lower = (value, cap) => value === 'unlimited' || BigInt(value) > BigInt(cap) ? cap : value;
    return {option,soft:m[1],hard:m[2],restricted:lower(m[1],target)};
  });
}
function boundedHostTool(program, args) {
  cp.execFileSync('/usr/bin/timeout',['--kill-after=0.250s','3.000s',program,...args],
    {timeout:4000, maxBuffer:4096, stdio:'ignore', env:{PATH:'/usr/sbin:/usr/bin:/sbin:/bin'}});
}
function restrictObserver() {
  if (typeof process.getuid !== 'function' || process.getuid() === 0) throw new Error('uid');
  const limits = limitSnapshot();
  const setArgs = limits.map(x => '--'+x.option+'='+x.restricted+':'+x.hard);
  boundedHostTool('/usr/bin/prlimit',['--pid='+process.pid,...setArgs]);
  const observed = limitSnapshot();
  if (observed.some((x,i) => x.soft !== limits[i].restricted || x.hard !== limits[i].hard)) throw new Error('limit');
  const threads = fs.readdirSync('/proc/self/task');
  if (threads.length < 1 || threads.length > 128 || threads.some(x => !/^[0-9]+$/.test(x))) throw new Error('threads');
  for (const thread of threads) {
    const status = fs.readFileSync('/proc/self/task/'+thread+'/status','utf8').split('\n');
    const affinity = status.find(x => x.startsWith('Cpus_allowed_list:'));
    if (!affinity || !/^(?:Cpus_allowed_list:)\s*(?:0-1|0,1)\s*$/.test(affinity)) throw new Error('affinity');
  }
  return {restore:limits.map(x => '--'+x.option+'='+x.soft+':'+x.hard),
    observed:{limits:Object.fromEntries(observed.map(x => [x.option,{soft:x.soft,hard:x.hard}])),affinity:'0-1'}};
}

function startTicks(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 0) throw new Error('pid');
  const descriptor = fs.openSync('/proc/' + pid + '/stat', 'r');
  try {
    const buf = Buffer.alloc(4097);
    const n = fs.readSync(descriptor, buf, 0, buf.length, null);
    if (n > 4096) throw new Error('stat');
    const text = buf.subarray(0, n).toString('utf8');
    const end = text.lastIndexOf(')');
    const fields = text.slice(end + 2).trim().split(/\s+/);
    if (end < 0 || fields.length < 20 || !/^[0-9]{1,20}$/.test(fields[19]) || /^0+$/.test(fields[19])) throw new Error('stat');
    return fields[19];
  } finally { fs.closeSync(descriptor); }
}

// A fixed-size record goes to the runner's per-step output file. Every async
// operation is subject to one shared deadline, including open, write and close.
function publish(name, record, literal=false) {
  return new Promise(resolve => {
    let done = false;
    let fd = null;
    const finish = ok => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      if (fd !== null) { try { fs.close(fd, () => {}); } catch (_) {} }
      resolve(ok);
    };
    const timer = setTimeout(() => finish(false), 1500);
    if (typeof output !== 'string' || !output) { finish(false); return; }
    const encoded = literal ? String(record) : Buffer.from(JSON.stringify(record)).toString('base64');
    if (encoded.length > 8192) { finish(false); return; }
    const data = Buffer.from(name + '=' + encoded + '\n');
    fs.open(output, fs.constants.O_WRONLY | fs.constants.O_APPEND | fs.constants.O_NOFOLLOW, (error, handle) => {
      if (done) { if (!error) fs.close(handle, () => {}); return; }
      if (error) { finish(false); return; }
      fd = handle;
      fs.fstat(fd, (statError, st) => {
        if (done) return;
        if (statError || !st.isFile()) { finish(false); return; }
        fs.write(fd, data, 0, data.length, null, (writeError, written) => {
          if (done) return;
          if (writeError || written !== data.length) { finish(false); return; }
          const saved = fd; fd = null;
          fs.close(saved, closeError => finish(!closeError));
        });
      });
    });
  });
}

function manager(unit, properties) {
  const raw=cp.execFileSync('/usr/bin/timeout',['--kill-after=0.250s','1.500s',
    '/usr/bin/sudo','-n','/usr/bin/systemctl','show',unit,'--property='+properties.join(',')],
    {timeout:2000,maxBuffer:16384,encoding:'utf8',env:{PATH:'/usr/sbin:/usr/bin:/sbin:/bin'},stdio:['ignore','pipe','ignore']});
  const result={};
  for (const line of raw.trim().split('\n')) {
    const at=line.indexOf('=');
    if (at<1 || Object.prototype.hasOwnProperty.call(result,line.slice(0,at))) throw new Error('property');
    result[line.slice(0,at)]=line.slice(at+1);
  }
  if (properties.some(k=>typeof result[k]!=='string')) throw new Error('property');
  return result;
}
function asyncTool(program,args) {
  return new Promise((resolve,reject) => {
    cp.execFile('/usr/bin/timeout',['--kill-after=0.250s','1.500s',program,...args],
      {timeout:2000,maxBuffer:16384,encoding:'utf8',env:{PATH:'/usr/sbin:/usr/bin:/sbin:/bin'}},
      (error,stdout) => error ? reject(error) : resolve(stdout));
  });
}
async function asyncManager(unit,properties) {
  const raw=await asyncTool('/usr/bin/sudo',['-n','/usr/bin/systemctl','show',unit,'--property='+properties.join(',')]);
  const result={};
  for (const line of raw.trim().split('\n')) {
    const at=line.indexOf('=');
    if (at<1 || Object.prototype.hasOwnProperty.call(result,line.slice(0,at))) throw new Error('property');
    result[line.slice(0,at)]=line.slice(at+1);
  }
  if (properties.some(k=>typeof result[k]!=='string')) throw new Error('property');
  return result;
}
async function trustedInventory(kind) {
  if (!['adapter','upload'].includes(kind)) throw new Error('inventory');
  const text=await asyncTool('/usr/bin/sudo',['-n','/usr/bin/python3','-B','-c',inventoryReader,
    process.env.GITHUB_WORKSPACE || '',packageName,run || '',commit || '',kind]);
  return JSON.parse(text);
}
async function attachObserver(pid,ticks) {
  const scope='b38-'+packageName+'-observer.scope';
  const slice='b38-'+packageName+'-observer.slice';
  const baseline=await trustedInventory('adapter');
  const intent=baseline.observer_scope;
  const sliceIntent=baseline.owned_slices && baseline.owned_slices.observer;
  if (!intent || intent.id!==scope || intent.state!=='intent' || intent.invocation_id!==null ||
      intent.control_group!==null || intent.job_id!==null || !sliceIntent || sliceIntent.id!==slice ||
      sliceIntent.state!=='intent' || sliceIntent.invocation_id!==null || sliceIntent.control_group!==null) throw new Error('intent');
  boundedHostTool('/usr/bin/sudo',['-n','/usr/bin/busctl','--system','call','org.freedesktop.systemd1',
    '/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StartTransientUnit',
    'ssa(sv)a(sa(sv))',slice,'fail','1','StopWhenUnneeded','b','false','0']);
  const memory=packageName==='x03'?'4294967296':'6442450944';
  const raw=cp.execFileSync('/usr/bin/timeout',['--kill-after=0.250s','3.000s','/usr/bin/sudo','-n',
    '/usr/bin/busctl','--system','call','org.freedesktop.systemd1','/org/freedesktop/systemd1',
    'org.freedesktop.systemd1.Manager','StartTransientUnit','ssa(sv)a(sa(sv))',scope,'fail','12',
    'PIDs','au','1',String(pid),'Slice','s',slice,'MemoryMax','t',memory,'MemorySwapMax','t','0',
    'CPUQuotaPerSecUSec','t','2000000','AllowedCPUs','ay','1','3','TasksMax','t','128',
    'RuntimeMaxUSec','t','175000000','TimeoutStopUSec','t','2000000','KillMode','s','control-group',
    'SendSIGKILL','b','true','FinalKillSignal','i','9','0'],
    {timeout:4000,maxBuffer:4096,encoding:'utf8',env:{PATH:'/usr/sbin:/usr/bin:/sbin:/bin'},stdio:['ignore','pipe','ignore']});
  const job=/^o "\/org\/freedesktop\/systemd1\/job\/([0-9]{1,20})"\s*$/.exec(raw);
  if (!job) throw new Error('job');
  const deadline=Date.now()+8000; const pause=new Int32Array(new SharedArrayBuffer(4));
  while (Date.now()<deadline) {
    try {
      const unit=manager(scope,['Id','InvocationID','ControlGroup','Slice','ActiveState','RuntimeMaxUSec','TimeoutStopUSec','KillMode','SendSIGKILL','FinalKillSignal']);
      const parent=manager(slice,['Id','InvocationID','ControlGroup','ActiveState']);
      const self=boundedRead('/proc/self/cgroup').split('\n').find(x=>x.startsWith('0:'));
      if (unit.Id!==scope || unit.Slice!==slice || unit.ActiveState!=='active' || !/^[0-9a-f]{32}$/.test(unit.InvocationID) ||
          parent.Id!==slice || parent.ActiveState!=='active' || !/^[0-9a-f]{32}$/.test(parent.InvocationID) ||
          !parent.ControlGroup.endsWith('/b38-'+packageName+'.slice/'+slice) || unit.ControlGroup!==parent.ControlGroup+'/'+scope ||
          !self || self.slice(2)!==unit.ControlGroup || !['2min 55s','175s'].includes(unit.RuntimeMaxUSec) ||
          unit.TimeoutStopUSec!=='2s' || unit.KillMode!=='control-group' || unit.SendSIGKILL!=='yes' ||
          !['9','SIGKILL'].includes(unit.FinalKillSignal)) throw new Error('policy');
      const base='/sys/fs/cgroup'+unit.ControlGroup;
      const controls=Object.fromEntries(['memory.max','memory.swap.max','cpu.max','cpuset.cpus.effective','pids.max'].map(k=>[k,boundedRead(base+'/'+k).trim()]));
      const quota=controls['cpu.max'].split(' ').map(Number);
      if (controls['memory.max']!==memory || controls['memory.swap.max']!=='0' ||
          !['0-1','0,1'].includes(controls['cpuset.cpus.effective']) || controls['pids.max']!=='128' ||
          quota.length!==2 || !(quota[0]>0 && quota[1]>0 && quota[0]<=2*quota[1])) throw new Error('controls');
      return {version:1,package:packageName,run_id:run,commit,supervisor_pid:pid,supervisor_start_ticks:ticks,
        scope:{id:scope,invocation_id:unit.InvocationID,control_group:unit.ControlGroup,job_id:job[1],state:'observed'},
        slice:{id:slice,invocation_id:parent.InvocationID,control_group:parent.ControlGroup,state:'observed'},controls};
    } catch (_) { Atomics.wait(pause,0,0,50); }
  }
  throw new Error('observer');
}
function matchesSlice(row,seen,name) {
  return row && row.state==='observed' && row.id===name && /^[0-9a-f]{32}$/.test(row.invocation_id || '') &&
    seen.Id===name && seen.ActiveState==='active' && seen.InvocationID===row.invocation_id &&
    seen.ControlGroup===row.control_group && row.control_group.startsWith('/') &&
    !row.control_group.includes('..') && row.control_group.endsWith('/b38-'+packageName+'.slice/'+name);
}
function emptyCgroup(group) {
  const base='/sys/fs/cgroup'+group;
  const events=boundedRead(base+'/cgroup.events');
  const processes=boundedRead(base+'/cgroup.procs');
  return events.split('\n').filter(x=>x.startsWith('populated ')).join('\n')==='populated 0' && processes==='';
}
async function observeUpload(childDone,childRecord) {
  const unitName='b38-'+packageName+'-upload-parent.service';
  const sliceName='b38-'+packageName+'-upload-parent.slice';
  const keys=['Id','InvocationID','Slice','Result','ExecMainCode','ExecMainStatus','ActiveState','SubState'];
  const deadline=Date.now()+120000;
  let invocation=null, witnessed=false, unitSuccess=false, cleanup=false, outputOk=false, retainedUnit=null, retainedSlice=null;
  try {
    while (Date.now()<deadline && !childDone()) {
      let current=null;
      try { current=await asyncManager(unitName,[...keys,'LoadState','ExitType','RemainAfterExit']); }
      catch (_) { await new Promise(resolve=>setTimeout(resolve,100)); continue; }
      if (current.LoadState!=='loaded' || !/^[0-9a-f]{32}$/.test(current.InvocationID)) {
        await new Promise(resolve=>setTimeout(resolve,100)); continue;
      }
      if (current.Id!==unitName || current.Slice!==sliceName || current.ExitType!=='cgroup' || current.RemainAfterExit!=='yes') throw new Error('policy');
      if (invocation===null) invocation=current.InvocationID;
      if (current.InvocationID!==invocation) throw new Error('invocation');
      if (!(current.ActiveState==='active' && current.SubState==='exited') && current.ActiveState!=='failed') {
        if (current.ActiveState==='inactive') throw new Error('terminal');
        await new Promise(resolve=>setTimeout(resolve,100)); continue;
      }
      const unit=Object.fromEntries(keys.map(k=>[k,current[k]]));
      const terminal=await publish('upload_parent_terminal',unit);
      if (!terminal || !/^[a-z][a-z0-9-]{0,63}$/.test(unit.Result) ||
          !/^[1-3]$/.test(unit.ExecMainCode) || !/^[0-9]{1,3}$/.test(unit.ExecMainStatus) || Number(unit.ExecMainStatus)>255) throw new Error('terminal');
      unitSuccess=unit.Result==='success' && unit.ExecMainCode==='1' && unit.ExecMainStatus==='0' &&
        unit.ActiveState==='active' && unit.SubState==='exited';
      while (Date.now()<deadline) {
        const slice=await asyncManager(sliceName,['Id','ActiveState','ControlGroup','InvocationID']);
        const suffix='/b38-'+packageName+'.slice/'+sliceName;
        if (slice.Id!==sliceName || slice.ActiveState!=='active' || !slice.ControlGroup.startsWith('/') ||
            slice.ControlGroup.includes('..') || !slice.ControlGroup.endsWith(suffix)) throw new Error('slice');
        const base='/sys/fs/cgroup'+slice.ControlGroup;
        const events=boundedRead(base+'/cgroup.events');
        const processes=boundedRead(base+'/cgroup.procs');
        if (events.split('\n').filter(x=>x.startsWith('populated ')).join('\n')==='populated 0' && processes.trim()==='') {
          witnessed=await publish('upload_parent_witness',{unit,slice,events,processes});
          if (witnessed) { retainedUnit=unit; retainedSlice=slice; }
          break;
        }
        await new Promise(resolve=>setTimeout(resolve,100));
      }
      break;
    }
  } catch (_) {}
  // Stopping releases systemd-run --wait. Missing evidence is never repaired
  // by cleanup and is always reflected in the parent action failure.
  try {
    if (!witnessed || invocation===null || !retainedUnit || !retainedSlice) throw new Error('custody');
    const adapter=await trustedInventory('adapter');
    const upload=await trustedInventory('upload');
    const service=await asyncManager(unitName,keys);
    const slice=await asyncManager(sliceName,['Id','InvocationID','ControlGroup','ActiveState']);
    if (!adapter.scope || adapter.scope.id!=='b38-'+packageName+'-adapter.scope' || adapter.scope.state!=='observed' ||
        adapter.adapter_pid!==childRecord.pid || adapter.adapter_start_ticks!==childRecord.start_ticks ||
        !upload.unit || upload.unit.Id!==unitName || upload.unit.InvocationID!==invocation ||
        upload.unit.ControlGroup!==slice.ControlGroup+'/'+unitName ||
        service.Id!==unitName || service.InvocationID!==invocation || service.Slice!==sliceName ||
        !keys.every(k=>service[k]===retainedUnit[k]) ||
        !matchesSlice(adapter.owned_slices && adapter.owned_slices.upload,slice,sliceName) ||
        slice.InvocationID!==retainedSlice.InvocationID || slice.ControlGroup!==retainedSlice.ControlGroup ||
        !emptyCgroup(slice.ControlGroup)) throw new Error('custody');
    await asyncTool('/usr/bin/sudo',['-n','/usr/bin/systemctl','stop',unitName]);
    const fresh=await asyncManager(sliceName,['Id','InvocationID','ControlGroup','ActiveState']);
    if (!matchesSlice(adapter.owned_slices.upload,fresh,sliceName) ||
        fresh.InvocationID!==retainedSlice.InvocationID || fresh.ControlGroup!==retainedSlice.ControlGroup ||
        !emptyCgroup(fresh.ControlGroup)) throw new Error('custody');
    await asyncTool('/usr/bin/sudo',['-n','/usr/bin/systemctl','stop',sliceName]);
    cleanup=true;
  } catch (_) {}
  outputOk=await publish('upload_parent_cleanup',cleanup?'true':'false',true);
  return witnessed && unitSuccess && cleanup && outputOk;
}
function boundedRead(file) {
  const fd=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
  try {
    const buf=Buffer.alloc(4097); const n=fs.readSync(fd,buf,0,buf.length,null);
    if (n>4096) throw new Error('bound');
    return buf.subarray(0,n).toString('utf8');
  } finally { fs.closeSync(fd); }
}
function scopeSnapshot() {
  const scope='b38-'+packageName+'-adapter.scope';
  const slice='b38-'+packageName+'-adapter.slice';
  const hold='b38-'+packageName+'-hold.slice';
  const unit=manager(scope,['Id','InvocationID','Result','ActiveState','SubState','Slice']);
  const sliceProps=manager(slice,['Id','ActiveState','ControlGroup','InvocationID']);
  const holdProps=manager(hold,['Id','ActiveState','After','InvocationID','ControlGroup']);
  const suffix='/b38-'+packageName+'.slice/'+slice;
  if (unit.Id!==scope || !/^[0-9a-f]{32}$/.test(unit.InvocationID) ||
      !unit.Result || !['inactive','failed'].includes(unit.ActiveState) || unit.Slice!==slice ||
      sliceProps.Id!==slice || sliceProps.ActiveState!=='active' ||
      !sliceProps.ControlGroup.startsWith('/') || sliceProps.ControlGroup.includes('..') ||
      !sliceProps.ControlGroup.endsWith(suffix) || holdProps.Id!==hold ||
      holdProps.ActiveState!=='active' || !holdProps.After.split(/\s+/).includes(scope) ||
      !holdProps.After.split(/\s+/).includes('b38-'+packageName+'-observer.scope')) throw new Error('scope');
  const base='/sys/fs/cgroup'+sliceProps.ControlGroup;
  const events=boundedRead(base+'/cgroup.events');
  const processes=boundedRead(base+'/cgroup.procs');
  if (events.split('\n').filter(x=>x.startsWith('populated ')).join('\n')!=='populated 0' || processes.trim()!=='') throw new Error('populated');
  return {unit,slice:sliceProps,hold:holdProps,events,processes};
}
async function cleanupScope(witness,childRecord) {
  const adapter=await trustedInventory('adapter');
  const scope='b38-'+packageName+'-adapter.scope';
  const adapterName='b38-'+packageName+'-adapter.slice';
  const holdName='b38-'+packageName+'-hold.slice';
  if (!adapter.scope || adapter.scope.id!==scope || adapter.scope.state!=='observed' ||
      adapter.scope.invocation_id!==witness.unit.InvocationID || adapter.adapter_pid!==childRecord.pid ||
      adapter.adapter_start_ticks!==childRecord.start_ticks) throw new Error('custody');
  const scopeKeys=['Id','InvocationID','Result','ActiveState','SubState','Slice'];
  const scopeNow=await asyncManager(scope,scopeKeys);
  const adapterNow=await asyncManager(adapterName,['Id','InvocationID','ControlGroup','ActiveState']);
  const holdNow=await asyncManager(holdName,['Id','InvocationID','ControlGroup','ActiveState','After']);
  if (scopeNow.Id!==scope || scopeNow.InvocationID!==witness.unit.InvocationID ||
      !['inactive','failed'].includes(scopeNow.ActiveState) ||
      !scopeKeys.every(k=>scopeNow[k]===witness.unit[k]) ||
      !matchesSlice(adapter.owned_slices && adapter.owned_slices.adapter,adapterNow,adapterName) ||
      adapterNow.InvocationID!==witness.slice.InvocationID || adapterNow.ControlGroup!==witness.slice.ControlGroup ||
      !emptyCgroup(adapterNow.ControlGroup) ||
      !matchesSlice(adapter.owned_slices && adapter.owned_slices.hold,holdNow,holdName) ||
      !holdNow.After.split(/\s+/).includes(scope) ||
      !holdNow.After.split(/\s+/).includes('b38-'+packageName+'-observer.scope')) throw new Error('custody');
  await asyncTool('/usr/bin/sudo',['-n','/usr/bin/systemctl','stop',adapterName]);
  const fresh=await asyncManager(holdName,['Id','InvocationID','ControlGroup','ActiveState','After']);
  if (!matchesSlice(adapter.owned_slices.hold,fresh,holdName) ||
      !fresh.After.split(/\s+/).includes('b38-'+packageName+'-observer.scope')) throw new Error('custody');
}
async function settledScope() {
  const deadline=Date.now()+8000;
  const pause=new Int32Array(new SharedArrayBuffer(4));
  const scope='b38-'+packageName+'-adapter.scope';
  const slice='b38-'+packageName+'-adapter.slice';
  let failure=null, terminal=null;
  while (Date.now()<deadline) {
    try {
      if (terminal===null) {
        const value=manager(scope,['Id','InvocationID','Result','ActiveState','SubState','Slice']);
        if (value.Id!==scope || !/^[0-9a-f]{32}$/.test(value.InvocationID) || !value.Result ||
            !['inactive','failed'].includes(value.ActiveState) || value.Slice!==slice) throw new Error('terminal');
        // Retain the manager result even if a later cgroup or cleanup check fails.
        if (!await publish('adapter_scope_terminal',value)) throw new Error('output');
        terminal=value;
      }
      const witness=scopeSnapshot();
      if (JSON.stringify(witness.unit)!==JSON.stringify(terminal)) throw new Error('identity');
      return witness;
    } catch (error) { failure=error; }
    Atomics.wait(pause,0,0,Math.min(100,Math.max(1,deadline-Date.now())));
  }
  throw failure || new Error('scope');
}

async function main() {
  let supervisorTicks = null;
  try { supervisorTicks = startTicks(process.pid); } catch (_) {}
  const record = {version:1, package:packageName, run_id:run, commit,
    supervisor_pid:process.pid, supervisor_start_ticks:supervisorTicks,
    pid:null, start_ticks:null, status:null, signal:null,
    wait_completed:false, timeout:false, observer_resources:null};
  let child = null;
  let identityError = !['x01','x03'].includes(packageName) || !/^[0-9]{1,32}$/.test(run || '') ||
    !/^[0-9a-f]{40}$/.test(commit || '') || !supervisorTicks;
  let spawnError = false;
  let expiry = null;
  let killExpiry = null;
  let observerDone = false;
  let uploadObservation = null;
  try {
    if (identityError) throw new Error('identity');
    const observer=await attachObserver(process.pid,supervisorTicks);
    if (!await publish('observer_scope_identity',observer)) throw new Error('output');
    const limits = restrictObserver();
    record.observer_resources = limits.observed;
    const environment = {...process.env};
    environment.B38_OBSERVER_IDENTITY=JSON.stringify({scope:observer.scope,slice:observer.slice});
    // prlimit execs Node in place, retaining the child PID and restoring all
    // inherited soft limits before the adapter begins its existing prelude.
    child = cp.spawn('/usr/bin/prlimit', [...limits.restore, process.execPath, path.join(__dirname, 'bridge.js')],
      {env:environment, stdio:['ignore','inherit','inherit']});
    // Do not retain the provider credentials in the mutable JS environment.
    // The initial inherited process environment cannot be proven erased here.
    for (const key of ['ACTIONS_RUNTIME_TOKEN','ACTIONS_RESULTS_URL']) {
      delete environment[key]; delete process.env[key];
    }
    record.pid = Number.isSafeInteger(child.pid) && child.pid > 0 ? child.pid : null;
    try { record.start_ticks = startTicks(record.pid); } catch (_) { identityError = true; }
  } catch (_) { spawnError = true; }
  // Even a setup or spawn failure must drop mutable copies promptly.
  for (const key of ['ACTIONS_RUNTIME_TOKEN','ACTIONS_RESULTS_URL']) delete process.env[key];
  if (child) {
    uploadObservation=observeUpload(()=>observerDone,record);
    await new Promise(resolve => {
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true; observerDone=true; clearTimeout(expiry); clearTimeout(killExpiry); resolve();
      };
      child.once('error', () => { spawnError = true; if (record.pid === null) finish(); });
      child.once('exit', (status, signal) => {
        if (settled) return;
        const validStatus = Number.isInteger(status) && status >= 0 && status <= 255 && signal === null;
        const validSignal = status === null && typeof signal === 'string' &&
          Object.prototype.hasOwnProperty.call(os.constants.signals, signal);
        if (validStatus || validSignal) {
          record.status = status; record.signal = signal; record.wait_completed = true;
        }
        finish();
      });
      expiry = setTimeout(() => {
        record.timeout = true;
        try { child.kill('SIGKILL'); } catch (_) {}
        killExpiry = setTimeout(finish, killWaitMs);
      }, deadlineMs);
    });
  }
  const uploadSuccess=uploadObservation ? await uploadObservation : false;
  const published = await publish('adapter_witness',record);
  let scopePublished=false, scopeSuccess=false, cleanup=false, cleanupPublished=false, scopeWitness=null;
  try {
    const witness=await settledScope();
    scopeWitness=witness;
    // Retain terminal state and cessation before stopping either owned slice.
    scopePublished=await publish('adapter_scope_witness',witness);
    scopeSuccess=witness.unit.Result==='success' && witness.unit.ActiveState==='inactive' && witness.unit.SubState==='dead';
  } catch (_) {}
  try { if (!scopePublished || !scopeWitness) throw new Error('custody'); await cleanupScope(scopeWitness,record); cleanup=true; } catch (_) {}
  cleanupPublished=await publish('adapter_scope_cleanup',cleanup?'true':'false',true);
  const success = published && scopePublished && scopeSuccess && cleanup && cleanupPublished && uploadSuccess && !identityError && !spawnError && record.wait_completed &&
    !record.timeout && record.status === 0 && record.signal === null;
  if (!success) {
    try { process.stderr.write('B38_ADAPTER_SUPERVISOR_STOP {"incomplete":true}\n'); } catch (_) {}
    // Bound exit even if a failed pipe or unreaped child retains a handle.
    process.exit(1);
  }
}
main().catch(() => { try { process.stderr.write('B38_ADAPTER_SUPERVISOR_STOP {"incomplete":true}\n'); } catch (_) {} process.exit(1); });
