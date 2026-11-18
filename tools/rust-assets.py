"""Extract data assets for Rust. No C compiler or executable translation is used."""
from pathlib import Path
from zipfile import ZipFile
root=Path(__file__).resolve().parents[1]
destination=root/'rust/assets';destination.mkdir(parents=True,exist_ok=True)
with ZipFile(root/'original/ppk.zip') as archive:
 for name in ['PPK.B16','STAGE.PPK','STAGE.PP1','STAGE.PP2','STAGE.PP3','STAGE.PP4']:
  (destination/name).write_bytes(archive.read('ppk/'+name))
(destination/'initial-data.bin').write_bytes((root/'assets/initial-data.bin').read_bytes())
