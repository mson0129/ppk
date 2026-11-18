"""Prepare the local data files embedded by the Rust engine."""
from pathlib import Path
from zipfile import ZipFile

root = Path(__file__).resolve().parents[1]
archive_path = root / 'original/ppk.zip'
initial_data = root / 'assets/initial-data.bin'
missing = [p.relative_to(root).as_posix() for p in (archive_path, initial_data) if not p.is_file()]
if missing:
    raise SystemExit('Missing local build inputs: ' + ', '.join(missing) + '. See README.md.')

destination = root / 'rust/assets'
destination.mkdir(parents=True, exist_ok=True)
with ZipFile(archive_path) as archive:
    for name in ('PPK.B16', 'STAGE.PPK', 'STAGE.PP1', 'STAGE.PP2', 'STAGE.PP3', 'STAGE.PP4'):
        (destination / name).write_bytes(archive.read('ppk/' + name))
(destination / 'initial-data.bin').write_bytes(initial_data.read_bytes())
