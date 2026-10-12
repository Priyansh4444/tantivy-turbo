"""Replay preparation commands from accepted11; metric controllers stay frozen."""
import json,subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
phase=sys.argv[1]
timeout={'tests-targeted':1200,'build-final':1800,'tests':3600,'clippy':1800,'formatting':120}[phase]
command=json.loads((out/'attempt-11'/phase/'command.json').read_text())
subprocess.run([sys.executable,'stage.py',phase,str(timeout),*command],cwd=out,check=True)
