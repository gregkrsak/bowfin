# X03 static expectations — revision 1, before admission

These source/protocol derivations precede any experiment result. No interpreter
trace supplies an expected answer. This is not independent adversarial review.

| Root | Structural closure | Expected outcome | Predicted fuel |
| --- | --- | --- | ---: |
| compute | compute,seed,step | State(x=140,left=0) | 1675 |
| trap | trap,seed,step | RangeTrap computed=1002; caller x=11,left=64 | 18 |
| short | short,step | false; step never entered | 4 |
| wide_trap | wide_trap,wide_seed | RangeTrap computed=-9223372036854775809 | 8 |
| monitor | monitor,failed,read_code | Scenario table below | Not yet derived |
| b38_probe | b38_probe,b38_read,b38_relay | ProbePair(42,42), one Sensor released | Not yet derived |

Compute: first two iterations give 13 and 16; 62 subsequent iterations add 2.
Fuel hypothesis: initialization/preheader 9 + 2*24 + 62*26 + false header 4
+ return 2. BD-041 charges entries, statements, terminators and expression nodes;
continuation actions add no fuel. Synthetic Gotos still charge. Inspect actual
approved CFG before adopting this prediction. Limits 26/27 predict the first x
publication boundary; 31/32 the first left decrement; 1674/1675 final delivery.
The wide subtraction is mathematical and must trap before narrowing. Short
circuit excludes the RHS execution but not its structural callee admission.

Report tuple: observations,warnings,critical,peak,disposition,completed.

| Scenario | Source-derived expected result |
| --- | --- |
| Sample | Ok Report(2,0,0,42,0,2) |
| Warning | Ok Report(2,1,0,60,1,2) |
| Critical | Ok Report(2,1,1,90,2,2) |
| NoSensor | Failure phase=1,problem=1,read_status=0,completion=0; zero aggregates |
| OpenIoFailure | Same with problem=3 |
| NoSample | Failure(phase=2,problem=2,read_status=2,completion=1); observations=0,completed=1 |
| ReadIoFailure | Same with problem=3,read_status=3 |
| FinishIoFailure | Failure(3,3,0,2); observations=1,peak=42,completed=0 |
| FinishPartialIoFailure | Same application result; committed prefix=1 |
| combined read/finish failure | Failure(3,3,2,2); observations=completed=0; prefix=1 |
| release failure | Terminal tool stop; sole live Sensor and protected Error; no delivery |
| released/publication cut | Terminal limit; protected outcome; no live Sensor or delivery |
| threshold v then 42 | observations=completed=2; warnings=1 iff 50<=v<80; critical=1 iff v>=80; peak=max(v,42); disposition=2 if critical else warnings |

Threshold values: 0,49,50,79,80,4095. Classification tests critical before
warning, so counts are disjoint. Failed read accepts no observation but a
successful finish increments completed. Completion failure outranks a prior read
failure while preserving read_status. BED-034 protects either finish outcome
until fixed release succeeds. Terminal advance performs no work, three times.
BD-042 cleanup is not completion; host disposal is not source release.

Admission/inventory only is enabled in this revision. No interpreter comparison
is authorized by this manifest revision: exact independent fuel/depth/storage/
custody schedules must be bound to admitted IDs and committed first. A rejected
source stops the package unchanged. Inventory may not manufacture approval.
