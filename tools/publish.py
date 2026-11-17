"""Copy a verified build to a local website checkout, without network publication."""
from pathlib import Path
import shutil
import sys
root = Path(__file__).resolve().parents[1]
if len(sys.argv) != 2:
    raise SystemExit('Usage: python3 tools/publish.py /path/to/site/raccoon')
destination = Path(sys.argv[1]).expanduser().resolve()
if destination == root or root in destination.parents:
    raise SystemExit('Choose a separate website directory.')
if not (root / 'dist/port/ppk.wasm').is_file():
    raise SystemExit('Run build.sh first.')
import subprocess
subprocess.run([sys.executable,str(root/'tools/build-service-worker.py')],check=True)
shutil.copytree(root / 'dist', destination, dirs_exist_ok=True)
print('Updated local runtime files:', destination)
