"""Stage browser runtime sources into dist; never include development inputs."""
from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[1]
shutil.copytree(root / 'web', root / 'dist', dirs_exist_ok=True)

if (root/'dist/port/ppk.wasm').is_file():
    import subprocess, sys
    subprocess.run([sys.executable,str(root/'tools/build-service-worker.py')],check=True)
