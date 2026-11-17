"""Rebuild ppk.jsdos from the original ppk.zip and DOSBox configuration."""
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

root = Path(__file__).resolve().parents[1]
names = ('PPK.EXE', 'PPK.B16', 'STAGE.PPK', 'STAGE.PP1', 'STAGE.PP2', 'STAGE.PP3', 'STAGE.PP4')
with ZipFile(root / 'original/ppk.zip') as source, ZipFile(root / 'dist/ppk.jsdos', 'w', ZIP_DEFLATED) as bundle:
    bundle.writestr('.jsdos/dosbox.conf', (root / 'dos/dosbox.conf').read_bytes())
    for name in names:
        bundle.writestr(name, source.read('ppk/' + name))
print('Created', root / 'dist/ppk.jsdos')
