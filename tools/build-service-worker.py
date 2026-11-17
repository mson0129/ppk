"""Version the complete offline runtime, including the actual compiled WASM."""
from pathlib import Path
import hashlib, json
root=Path(__file__).resolve().parents[1]
dist=root/'dist'
names=['index.htm','pwa.js','app.webmanifest']
names += [p.relative_to(dist).as_posix() for p in sorted((dist/'port').iterdir()) if p.suffix in {'.js','.mjs','.wasm'}]
names += [p.relative_to(dist).as_posix() for p in sorted((dist/'icons').glob('*.png'))]
assert 'port/ppk.wasm' in names
assert {'icons/icon-192.png','icons/icon-512.png','icons/maskable-512.png','icons/apple-touch-icon.png'} <= set(names)
assets=[{'url':name,'sha256':hashlib.sha256((dist/name).read_bytes()).hexdigest()} for name in names]
template=(root/'tools/service-worker.template.js').read_text()
version=hashlib.sha256((template+json.dumps(assets,sort_keys=True)).encode()).hexdigest()[:20]
(dist/'sw.js').write_text(template.replace('__VERSION__',json.dumps(version)).replace('__ASSETS__',json.dumps(assets,ensure_ascii=False)))
print('Offline package:',version,'files:',len(assets))
