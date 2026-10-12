import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
m=json.loads((out/'attempt-22-metrics.json').read_text())
assert m['groups']['OR_3PLUS']['candidate_win_both_beyond_group_controls']
assert not any(m['groups'][kind]['parent_win_both_beyond_group_controls'] for kind in ('ALL','TERM','AND','PHRASE','OR_2'))
for name in ('measure','verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
for script in ('port_measure.py','analyze_port.py'):
 subprocess.run([sys.executable,script],cwd=out,check=True)
subprocess.run([sys.executable,'port_group_audit.py','--folder',str(out)],cwd=out,check=True)
