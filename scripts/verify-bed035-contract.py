#!/usr/bin/env python3
"""Check completeness and referential integrity of the Stage I evidence ledger.

This is a manifest check, not execution evidence or a semantic proof. Cargo
identity reconciliation and exact numeric neighbors have separate CI checks.
"""
import json
import hashlib
from pathlib import Path

root = Path(__file__).resolve().parents[1]
p = json.loads((root / 'docs/engineering/bed-035-evidence.json').read_text())
k34 = json.loads((root / 'docs/engineering/bed-034-evidence.json').read_text())
pre = json.loads((root / 'docs/engineering/bed-034-predecessors.json').read_text())
# Exact live-file correspondence for paths retained inside historical evidence.
# This adapter checks existing source text only; it grants no execution authority.
evidence_paths = {
    'scripts/verify-ked034-budgets.py': 'scripts/verify-bed034-budgets.py',
    'scripts/verify-ked035-contract.py': 'scripts/verify-bed035-contract.py',
    'scripts/verify-ked035-tests.py': 'scripts/verify-bed035-tests.py',
}
assert len(set(evidence_paths.values())) == len(evidence_paths)
known = {x['id'] for x in pre['identities']} | set(k34['additions']) | set(p['additions'])
assert p['baseline_commit'] == '5f28977a42a7df1817e5cac46b3f6660a38435d3'
assert p['baseline_tree'] == '0b330328f3f213a26b6fc4af9f45ca06a98876b8'
assert len(known) == 2045 + len(p['additions'])
assert set(p['requirements']) == {str(i) for i in range(4, 19)}
assert set(p['semantic_obligations']) == {f'15.{i:02}' for i in range(1, 31)}
# The original evidence keys and the live ledger headings are distinct roles.
# Enumerate this fixed namespace; never normalize arbitrary K-prefixed tokens.
constraint_names = {
    'K35-P01': 'B35-P01',
    'K35-P02': 'B35-P02',
    'K35-P03': 'B35-P03',
    'K35-P04': 'B35-P04',
    'K35-P05': 'B35-P05',
    'K35-P06': 'B35-P06',
    'K35-P07': 'B35-P07',
    'K35-P08': 'B35-P08',
    'K35-P09': 'B35-P09',
    'K35-P10': 'B35-P10',
}
assert len(set(constraint_names.values())) == len(constraint_names)
assert set(p['constraints']) == set(constraint_names)
assert set(p['milestones']) == {f'M{i}' for i in range(10)}
for group in ['requirements', 'semantic_obligations', 'constraints', 'malformed_families']:
    for key, row in p[group].items():
        assert row['description'] and row['status'], (group, key)
        deferred = 'deferred Stage N' in row['status']
        script_only = key == '15.29'
        assert row['tests'] or deferred or script_only, (group, key)
        assert len(row['tests']) == len(set(row['tests'])), (group, key)
        assert set(row['tests']) <= known, (group, key, 'unknown test')
        assert row['code'] or deferred, (group, key)
        for path in row['code']:
            assert (root / evidence_paths.get(path, path)).is_file(), (group, key, path)
assert p['constraints']['K35-P10']['status'] == 'deferred Stage N'
for key in ['15.23', '15.24']:
    assert p['semantic_obligations'][key]['status'] == 'deferred Stage N'
assert set(p['classifications']) == set(p['additions'])
# Every new identity must name an actual test function, independently of Cargo.
# Cargo CI additionally checks its exact module/target identity and pass status.
text = '\n'.join(path.read_text() for path in (root / 'compiler').rglob('*.rs'))
for identity in p['additions']:
    assert 'fn ' + identity.rsplit('::', 1)[-1] + '(' in text, identity
assert p['m5_checkpoint']['commit'] == 'e31f6104d8222ee201bc392a3daf87bcb03734ab'
assert p['m5_checkpoint']['tree'] == '34466f3032475e6b03931f59e61e712fab3b54e2'
assert p['stage_n_deferred'] and p['limitations'] and p['public_api'] and p['boundedness']
ledger = (root / 'docs/engineering/BED-035-STAGE-I-LEDGER.md').read_text()
assert all(name in ledger for name in constraint_names.values())
assert all(f'M{i}' in ledger for i in range(10))
print('BED-035 map: frozen sections 4–18; 30 obligations; P01–P10; M0–M9; named malformed families')
print(f'2045 retained identities + {len(p["additions"])} classified additions; native evidence explicitly deferred')

# Preserve every reviewed Stage I control while classifying focused additions.
r = p['repair']
assert r['baseline_commit'] == '4922296a04b479915ad58001275e491af1287d58'
assert r['baseline_tree'] == '08712b0f327e65f91ef39db4a19bad9311debd0e'
assert r['reviewed_control_count'] == 48 and r['predecessor_migrations'] == {}
reviewed = [[t, p['classifications'][t]] for t in p['additions'][:48]]
expected_digest = '513b6decc1a21aafd30c0b54faa8cac1f58ec105aa7ebe078d8c7e90c5e41a6c'
assert hashlib.sha256(json.dumps(reviewed, separators=(',', ':')).encode()).hexdigest() == expected_digest
assert r['reviewed_controls_sha256'] == expected_digest
assert set(p['additions'][48:]) == set(r['tests'])
assert r['expected_per_profile'] == 2045 + len(p['additions'])
assert r['public_api_changes'] == []
assert set(r['findings']) == {'K35-I01', 'K35-I02', 'K35-I03', 'K35-I04'}
for finding in r['findings'].values():
    assert finding['tests'] and set(finding['tests']) <= set(r['tests'])
    code = '\n'.join((root / path).read_text() for path in finding['code'])
    assert all(symbol in code for symbol in finding['symbols'])
print(f'Focused repair: 48 reviewed controls retained, {len(r["tests"])} classified repair tests; zero migrations')

# Retain every original focused identity/classification; corrected assertions are
# audited by discriminators, not by this inventory digest.
f = r['focused_baseline']
assert f['commit'] == '23dd37ca32c0ec6c9df34fc3cbc44b0a7371060a'
assert f['tree'] == '6536a0a21ba266ef4a29880f0bfeac02bb08ee1b'
retained = [[t, p['classifications'][t]] for t in p['additions'][48:59]]
assert len(f['retained_repair_identities']) == 11
assert [t for t, _ in retained] == f['retained_repair_identities']
assert hashlib.sha256(json.dumps(retained, separators=(',', ':')).encode()).hexdigest() == '7584d2513b23d0c72110a532d50a1cc9bb69d70301bb74b4c09cb81e6df1b05e'
assert f['retained_repairs_sha256'] == '7584d2513b23d0c72110a532d50a1cc9bb69d70301bb74b4c09cb81e6df1b05e'
assert len(r['effective_rejection_paths']) == 7 and r['evidence_correction']
assert r['discriminator_limits']
