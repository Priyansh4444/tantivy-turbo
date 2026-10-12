import subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
subprocess.run([sys.executable,'stage.py','formatting','120','rustfmt','+1.99.0','--check','--edition','2021','../../../tantivy-or-hillclimb-oct07/src/query/boolean_query/or_maxscore.rs'],cwd=out,check=True)
for phase in ('tests','clippy'):
    subprocess.run([sys.executable,'/tmp/or-hill-run.py',phase],check=True)
subprocess.run([sys.executable,'compare.py','measure','--parent','parent-06.json'],cwd=out,check=True)
subprocess.run([sys.executable,'analyze.py'],cwd=out,check=True)
subprocess.run([sys.executable,'measure_audit.py','--parent','parent-06.json','--attempt','11'],cwd=out,check=True)
