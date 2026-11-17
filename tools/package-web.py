"""Stage browser runtime sources into dist; never include development inputs."""
from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[1]
shutil.copytree(root / 'web', root / 'dist', dirs_exist_ok=True)
