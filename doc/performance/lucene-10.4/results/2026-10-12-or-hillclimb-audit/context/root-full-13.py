import hashlib,json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
repo=out.parents[1]/'tantivy-or-hillclimb-oct07'
expected={'src/query/boolean_query/or_maxscore.rs':'1a1ce59b330b4a207d353385bf90cac96333cd4c6da49ed52a7c40cd5bb30804','src/query/term_query/term_scorer.rs':'21b05a2b6513223a81613ee0f40c4a31f6990a427bb0d4b60324154a55753cdb'}
for path,digest in expected.items():assert hashlib.sha256((repo/path).read_bytes()).hexdigest()==digest
for name in ('verify','configured-k0.9-b0.4','configured-k2-b1'):
 assert json.loads((out/name/'receipt.json').read_text())['pass']
assert json.loads((out/'pilot-final/receipt.json').read_text())['unchanged']
subprocess.run([sys.executable,'root-run-stage.py','formatting'],cwd=out,check=True)
subprocess.run([sys.executable,'stage.py','formatting-term','120','rustfmt','+1.99.0','--check','--edition','2021','../../../tantivy-or-hillclimb-oct07/src/query/term_query/term_scorer.rs'],cwd=out,check=True)
for phase in ('tests','clippy'):
 subprocess.run([sys.executable,'root-run-stage.py',phase],cwd=out,check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-11.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-11.json','--attempt','13'],cwd=out,check=True)
