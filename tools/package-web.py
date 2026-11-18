"""Stage browser runtime sources into dist; never include development inputs."""
from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[1]
dist = root / 'dist'
shutil.copytree(
    root / 'web', dist, dirs_exist_ok=True,
    ignore=shutil.ignore_patterns('dos.htm', 'README.md', '.DS_Store', '*.jsdos'),
)
# Remove leftovers from builds that used to package the DOS reference.
for name in ('dos.htm', 'ppk.jsdos', 'README.md'):
    (dist / name).unlink(missing_ok=True)
# GitHub Pages and ordinary HTTP servers use index.html for a directory URL.
# Generate the alias from the canonical source so install metadata cannot drift.
shutil.copy2(root/'dist/index.htm', root/'dist/index.html')

if (root/'dist/port/ppk.wasm').is_file():
    import subprocess, sys
    subprocess.run([sys.executable,str(root/'tools/build-service-worker.py')],check=True)
