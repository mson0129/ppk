"""Compare the frozen C engine and production Rust WASM, not just selected fields.
Build C first: WASI_SDK_PATH=... sh tools/build-c-reference.sh
Run: python tests/compare_c_rust.py
"""
from pathlib import Path
import hashlib,json,random,time
import wasmtime
ROOT=Path(__file__).resolve().parents[1]
class Engine:
 def __init__(self,path):
  self.store=wasmtime.Store();module=wasmtime.Module.from_file(self.store.engine,str(path))
  assert not module.imports
  self.exports=wasmtime.Instance(self.store,module,[]).exports(self.store);self.call('init')
  self.base=self.call('data_pointer');self.framebuffer=self.call('framebuffer')
 def call(self,name,*args):return self.exports[name](self.store,*args)
 def read(self,offset,size):return bytes(self.exports['memory'].read(self.store,self.base+offset,self.base+offset+size))
 def frame(self):return bytes(self.exports['memory'].read(self.store,self.framebuffer,self.framebuffer+256000))
 def inject(self,offset,value):self.exports['memory'].write(self.store,int(value&65535).to_bytes(2,'little'),self.base+0xf0a0+offset)
def assert_same(c,r,stage,tick):
 a,b=c.read(0xf0a0,0x1784),r.read(0xf0a0,0x1784)
 if a!=b:raise AssertionError((stage,tick,'state',[(hex(i),x,y)for i,(x,y)in enumerate(zip(a,b))if x!=y][:20]))
 assert c.read(0x20000,32000)==r.read(0x20000,32000),(stage,tick,'background')
 assert c.frame()==r.frame(),(stage,tick,'framebuffer')
 for name in ['get_phase','get_stage','run_finished','audio_frequency','audio_event','frame_count','transition_kind']:
  assert c.call(name)==r.call(name),(stage,tick,name)
def main():
 c=Engine(ROOT/'analysis/c-reference.wasm');r=Engine(ROOT/'dist/port/ppk.wasm');start=time.monotonic();checks=0
 for stage in range(1,51):
  for e in[c,r]:e.call('start_stage',stage);e.call('show_intro')
  rng=random.Random(stage);keys=[]
  for tick in range(450):
   if tick%7==0:keys=[k for k in[72,75,77,80,57]if rng.randrange(5)==0]
   if tick==2:keys=[57]
   for e in[c,r]:
    for scan in[72,75,77,80,57]:e.call('key',scan,int(scan in keys))
    e.call('tick');e.call('render_interpolated',tick*137%1001)
   assert_same(c,r,stage,tick);checks+=1
  if stage%5==0:print('PASS stages',stage,'ticks',checks,flush=True)
 # Force completion through every stage, including fades, bonuses and wrap.
 for stage in range(1,51):
  for e in[c,r]:e.call('start_stage',stage);e.inject(0x13fa,0)
  for tick in range(40):
   for e in[c,r]:
    e.call('key',57,int(tick>=20));e.call('tick');e.call('render_interpolated',tick*117%1001)
   assert_same(c,r,stage,tick);checks+=1
 result={'c_sha256':hashlib.sha256((ROOT/'analysis/c-reference.wasm').read_bytes()).hexdigest(),'rust_sha256':hashlib.sha256((ROOT/'dist/port/ppk.wasm').read_bytes()).hexdigest(),'stages':50,'tick_comparisons':checks,'compared':['complete 0x1784-byte legacy data image','32000 background-plane bytes','all 64000 RGBA pixels','phase, stage, RTA completion, sound and tick counter'],'seconds':round(time.monotonic()-start,3)}
 (ROOT/'analysis/rust-verification.json').write_text(json.dumps(result,indent=2)+'\n');print('PASS',checks,'tick comparisons',flush=True)
if __name__=='__main__':main()
