"""Differential test against original x86 palette loops (development only)."""
import struct,unittest,math
from pathlib import Path
from unicorn import Uc,UC_ARCH_X86,UC_MODE_16,UC_HOOK_CODE
from unicorn.x86_const import *
import test_wasm as core
ROOT=core.ROOT
NUM=65536*25175000
DEN=1193182*800*449

def original(entry,palette):
 raw=(ROOT/'analysis/PPK-unpacked.EXE').read_bytes();image=raw[struct.unpack_from('<H',raw,8)[0]*16:]
 u=Uc(UC_ARCH_X86,UC_MODE_16);u.mem_map(0,0x100000);u.mem_write(0,image);u.mem_write(0x50000,bytes(palette));u.mem_write(0x4fef8,struct.pack('<4H',0,0x6000,0,0x5000))
 for reg,value in [(UC_X86_REG_CS,0x327),(UC_X86_REG_IP,entry),(UC_X86_REG_SS,0x4000),(UC_X86_REG_SP,0xfef8),(UC_X86_REG_DS,0xf0a)]:u.reg_write(reg,value)
 samples=[];retraces=0
 def hook(uc,address,size,user):
  nonlocal retraces
  if address not in (0x2feb,0x340b):return
  ss=uc.reg_read(UC_X86_REG_SS)*16;sp=uc.reg_read(UC_X86_REG_SP)
  if address==0x2feb:retraces+=1
  else:
   off,seg=struct.unpack('<HH',bytes(uc.mem_read(ss+sp+4,4)));samples.append(bytes(uc.mem_read(seg*16+off,48)))
  ip,cs=struct.unpack('<HH',bytes(uc.mem_read(ss+sp,4)));uc.reg_write(UC_X86_REG_SP,(sp+4)&65535);uc.reg_write(UC_X86_REG_CS,cs);uc.reg_write(UC_X86_REG_IP,ip)
 u.hook_add(UC_HOOK_CODE,hook);u.emu_start(0x3270+entry,0x60000,count=3000000)
 return retraces,samples

class FadeTests(unittest.TestCase):
 setUpClass=classmethod(core.PortTests.setUpClass.__func__)
 setUp=core.PortTests.setUp
 call=core.PortTests.call
 state=core.PortTests.state
 put=core.PortTests.put
 ticks=core.PortTests.ticks
 finish_fade=core.PortTests.finish_fade
 def pixels(self):
  fb=self.call('framebuffer');return bytes(self.memory.read(self.store,fb,fb+256000))
 def palette(self,pixels):
  colors=list(dict.fromkeys(pixels[i:i+4] for i in range(0,len(pixels),4)));assert len(colors)<=16
  return colors,bytes((v*63+127)//255 for c in colors+[b'\0\0\0\xff']*(16-len(colors)) for v in c[:3])
 def expected(self,pixels,colors,palette):
  table={c:bytes(palette[i*3+j]*255//63 for j in range(3))+b'\xff' for i,c in enumerate(colors)}
  return b''.join(table[pixels[i:i+4]] for i in range(0,len(pixels),4))
 def test_original_palette_frames_and_readonly_rendering(self):
  stage=self.pixels();sc,sp=self.palette(stage);self.call('show_intro');title=self.pixels();tc,tp=self.palette(title)
  out_count,out=original(0x2e6,tp);in_count,inside=original(0x1d1,sp)
  self.assertEqual((out_count,len(out),in_count,len(inside)),(64,64,65,65))
  self.call('key',57,1);self.ticks(1);self.assertEqual(self.call('get_phase'),5)
  tick=0
  for frame in range(130):
   t=frame*DEN//NUM;fraction=math.ceil((frame*DEN-t*NUM)*1000/NUM)
   self.ticks(t-tick);tick=t
   digest=self.call('replay_hash');self.call('render_interpolated',fraction)
   self.assertEqual(self.call('transition_frame',fraction),frame)
   self.assertEqual(self.call('replay_hash'),digest)
   expected=title if frame==0 else self.expected(title,tc,out[frame-1]) if frame<=64 else self.expected(stage,sc,inside[frame-65])
   self.assertEqual(self.pixels(),expected,f'original palette mismatch at retrace {frame}')
   self.assertEqual((self.state(0x561),self.state(0x563)),(248,168))
  self.finish_fade();self.assertEqual(self.call('get_phase'),0)

 def test_clear_and_next_stage_retrace_duration(self):
  self.put(0x13fa,0);self.ticks(1)
  self.assertEqual(self.call('transition_kind'),2)
  self.ticks(16);self.assertEqual(self.call('get_phase'),5)
  self.ticks(1);self.assertEqual(self.call('get_phase'),4)
  self.ticks(1);self.call('key',57,1);self.ticks(1)
  self.assertEqual(self.call('transition_kind'),3)
  self.assertEqual(self.call('get_stage'),2)
  self.ticks(16);self.assertEqual(self.call('get_phase'),5)
  self.ticks(1);self.assertEqual(self.call('get_phase'),0)
