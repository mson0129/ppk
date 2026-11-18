"""Check generated PWA metadata and byte-level offline package coherence."""
import hashlib,json,re,struct,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class PwaTests(unittest.TestCase):
 def test_manifest_and_platform_icons(self):
  manifest=json.loads((ROOT/'dist/app.webmanifest').read_text())
  self.assertEqual(manifest['display'],'standalone');self.assertEqual(manifest['start_url'],'./index.html');self.assertEqual(manifest['scope'],'./')
  for icon in manifest['icons']:
   data=(ROOT/'dist'/icon['src']).read_bytes();width,height=struct.unpack('>II',data[16:24]);self.assertEqual(icon['sizes'],f'{width}x{height}');self.assertTrue(data.startswith(b'\x89PNG'))
  apple=(ROOT/'dist/icons/apple-touch-icon.png').read_bytes();self.assertEqual(struct.unpack('>II',apple[16:24]),(180,180))
 def test_cache_hashes_cover_runtime_and_exclude_online_dos(self):
  worker=(ROOT/'dist/sw.js').read_text();assets=json.loads(re.search(r'const ASSETS = (.+);',worker)[1]);names={a['url'] for a in assets}
  self.assertTrue({'index.html','pwa.js','app.webmanifest','port/ppk.wasm','port/player.js','port/gamepad-input.mjs','port/speedrun-timer.mjs'}<=names)
  self.assertNotIn('dos.htm',names)
  self.assertNotIn('index.htm',names)
  self.assertFalse((ROOT/'dist/index.htm').exists())
  for asset in assets:self.assertEqual(asset['sha256'],hashlib.sha256((ROOT/'dist'/asset['url']).read_bytes()).hexdigest())
  for file in (ROOT/'dist/port').glob('*.mjs'):self.assertIn('port/'+file.name,names)
 def test_html_links_install_metadata(self):
  html=(ROOT/'dist/index.html').read_text();self.assertIn('rel="manifest"',html);self.assertIn('rel="apple-touch-icon"',html);self.assertIn('src="./pwa.js"',html)

 def test_directory_entry_preserves_install_identity_and_icons(self):
  from urllib.parse import urljoin
  text=(ROOT/'dist/index.html').read_text()
  self.assertIn('rel="canonical" href="./index.html"',text)
  self.assertIn("location.pathname.endsWith('/')",text)
  self.assertIn('apple-touch-icon.png?v=2',text)
  manifest=json.loads((ROOT/'dist/app.webmanifest').read_text())
  self.assertEqual(manifest['id'],'./index.html')
  for relative in ['./app.webmanifest','./icons/apple-touch-icon.png?v=2',manifest['start_url']]:
   self.assertEqual(urljoin('https://mson0129.github.io/ppk/',relative),urljoin('https://mson0129.github.io/ppk/index.html',relative))
