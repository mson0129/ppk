"""Extract original data without shipping an EXE to the native player."""
from pathlib import Path
from zipfile import ZipFile
import hashlib,json
root=Path(__file__).resolve().parents[1]
with ZipFile(root/'original/ppk.zip') as z:
 files={n:z.read('ppk/'+n) for n in ['PPK.B16','STAGE.PPK','STAGE.PP1','STAGE.PP2','STAGE.PP3','STAGE.PP4']}
files['initial-data.bin']=(root/'assets/initial-data.bin').read_bytes()
with (root/'src/assets.generated.h').open('w') as f:
 for name,data in files.items():
  symbol='asset_'+name.lower().replace('.','_').replace('-','_')
  f.write(f'static const unsigned char {symbol}[{len(data)}] = {{\n')
  for i in range(0,len(data),32): f.write(','.join(str(b) for b in data[i:i+32])+',\n')
  f.write('};\n')
(root/'analysis/asset-hashes.json').write_text(json.dumps({n:hashlib.sha256(d).hexdigest() for n,d in files.items()},indent=2)+'\n')
