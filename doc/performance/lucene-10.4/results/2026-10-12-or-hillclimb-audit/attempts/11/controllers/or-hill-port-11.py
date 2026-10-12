import subprocess,sys
from pathlib import Path
out=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07')
for script in ('port_measure.py','analyze_port.py'):
 subprocess.run([sys.executable,script],cwd=out,check=True)
subprocess.run([sys.executable,'port_group_audit.py','--folder',str(out)],cwd=out,check=True)
