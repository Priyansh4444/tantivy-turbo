import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
r=json.loads((out/'tests-targeted/receipt.json').read_text())
assert r['returncode']==0 and r['source_unchanged']
assert (out/'tests-targeted/compiler.txt').read_bytes()==(out/'attempt-11/tests-targeted/compiler.txt').read_bytes()
assert '17 passed; 0 failed' in (out/'tests-targeted/stdout.log').read_text()
subprocess.run([sys.executable,'root-run-stage.py','build-final'],cwd=out,check=True)
assert (out/'build-final/compiler.txt').read_bytes()==(out/'attempt-11/build-final/compiler.txt').read_bytes()
for phase in ('verify','configured','pilot'):
 subprocess.run([sys.executable,'compare.py',phase,'--parent','parent-11.json'],cwd=out,check=True)
