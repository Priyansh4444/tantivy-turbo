import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
r=json.loads((out/'tests-targeted/receipt.json').read_text());assert r['returncode']==0 and r['source_unchanged']
assert '32 passed; 0 failed' in (out/'tests-targeted/stdout.log').read_text()
for phase in ('tests-targeted-term','tests-targeted-postings'):
 command=json.loads((out/'attempt-13'/phase/'command.json').read_text())
 subprocess.run([sys.executable,'stage.py',phase,'1200',*command],cwd=out,check=True)
subprocess.run([sys.executable,'root-primary-19.py'],cwd=out,check=True)
