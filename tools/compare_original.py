"""Differential test: execute original x86 ONLY as an offline test oracle.
The browser build has no Unicorn, x86 interpreter, DOS or executable dependency.
Build test library: clang -O1 -Wno-parentheses-equality -DTEST_ORACLE -dynamiclib src/port.c -o /tmp/ppk-port.dylib
"""
from pathlib import Path
import ctypes, struct, random, sys
from unicorn import *
from unicorn.x86_const import *
from zipfile import ZipFile
root=Path(__file__).resolve().parents[1]
lib=ctypes.CDLL(sys.argv[1] if len(sys.argv)>1 else '/private/tmp/ppk-port.dylib')
lib.test_memory.restype=ctypes.c_void_p
lib.init();ptr=lib.test_memory(); DATA=0xf0a0
raw=(root/'analysis/PPK-unpacked.EXE').read_bytes();image=raw[struct.unpack_from('<H',raw,8)[0]*16:]
u=Uc(UC_ARCH_X86,UC_MODE_16);u.mem_map(0,0x60000)
# External graphics functions affect no gameplay state and are native platform calls.
skip={0x2fe0+i for i in [0xb,0x22,0x46,0x5f,0x179]}
skip|={0x3270+i for i in [0x19b,0x1d1,0x2e6]}
def ret_far():
 sp=u.reg_read(UC_X86_REG_SP);ss=u.reg_read(UC_X86_REG_SS);ip,cs=struct.unpack('<HH',u.mem_read(ss*16+sp,4));u.reg_write(UC_X86_REG_SP,(sp+4)&65535);u.reg_write(UC_X86_REG_CS,cs);u.reg_write(UC_X86_REG_IP,ip)
with ZipFile(root/'original/ppk.zip') as archive:
 title_file=archive.read('ppk/STAGE.PP1')
file_position=0
def hook(uc,address,size,data):
 global file_position
 if address in [0x8e0,0xa80,0x1b5f,0x54c]:
  sp=u.reg_read(UC_X86_REG_SP);args=struct.unpack('<6H',u.mem_read(0x40000+sp+4,12))
  if address==0x8e0:file_position=0;u.reg_write(UC_X86_REG_AX,1);u.reg_write(UC_X86_REG_DX,0)
  elif address==0xa80:
   count=args[2]*args[3];u.mem_write(args[1]*16+args[0],title_file[file_position:file_position+count]);file_position+=count;u.reg_write(UC_X86_REG_AX,args[3])
  elif address==0x1b5f:u.reg_write(UC_X86_REG_AX,title_file[file_position]);file_position+=1
  ret_far();return

 if address==0x38000:u.emu_stop();return
 if address in skip:ret_far();return
 if address==0x14c7:
  sp=u.reg_read(UC_X86_REG_SP);a=struct.unpack('<6H',u.mem_read(0x40000+sp+4,12));dst=a[1]*16+a[0];fmt=bytes(u.mem_read(a[3]*16+a[2],40)).split(b'\0')[0].decode();n=a[4];text=(fmt%n).encode()+b'\0';u.mem_write(dst,text);ret_far()
u.hook_add(UC_HOOK_CODE,hook)
def original_call(address,snapshot):
 u.mem_write(0,snapshot);u.mem_write(0,image[:0x6020]);
 for r,v in [(UC_X86_REG_CS,0x327),(UC_X86_REG_DS,0xf0a),(UC_X86_REG_ES,0),(UC_X86_REG_SS,0x4000),(UC_X86_REG_SP,0xfffa),(UC_X86_REG_BP,0),(UC_X86_REG_EFLAGS,2)]:u.reg_write(r,v)
 u.mem_write(0x4fffa,struct.pack('<HH',0,0x3800))
 u.emu_start(0x3270+address,0x38001,count=5000000)
 assert u.reg_read(UC_X86_REG_CS)==0x3800,hex(u.reg_read(UC_X86_REG_IP))
 return bytes(u.mem_read(DATA,0x1784))
# Independently decode the intro using the original file-loader routine.
lib.init();snapshot=ctypes.string_at(ptr,0x50000)
original_call(0x2477,snapshot)
expected_title=bytes(u.mem_read(0x20000,32000))
lib.show_intro()
assert ctypes.string_at(ptr+0x20000,32000)==expected_title
print('PASS: original intro loader and native RLE decoder produce identical 32,000-byte image')
# Verify full game data, except stack/external audio transport. No position tolerances.
routines=[0x11c1,0x12f5,0x1429,0xd98,0x1cca,0x1db1,0x1e09,0x15a3]
rng=random.Random(1992)
checks=0
for scenario in range(50):
 lib.start_stage(scenario+1)
 for frame in range(80):
  for scan in [72,75,77,80,57]:lib.key(scan,int(rng.random()<(.25 if scan!=57 else .12)))
  for address in routines:
   before=ctypes.string_at(ptr,0x50000)
   expected=original_call(address,before)
   lib.test_routine(address)
   actual=ctypes.string_at(ptr+DATA,0x1784)
   expected_bg=bytes(u.mem_read(0x20000,32000))
   actual_bg=ctypes.string_at(ptr+0x20000,32000)
   if actual_bg != expected_bg:
    d=[i for i,(a,b) in enumerate(zip(actual_bg,expected_bg)) if a!=b]
    print('GRAPHICS FAIL',scenario,frame,hex(address),[(hex(i),actual_bg[i],expected_bg[i]) for i in d[:10]])
    raise SystemExit(1)
   diffs=[i for i,(a,b) in enumerate(zip(actual,expected)) if a!=b]
   if diffs:
    print('FAIL scenario',scenario,'frame',frame,'routine',hex(address),'offsets',[(hex(i),actual[i],expected[i]) for i in diffs[:15]])
    raise SystemExit(1)
   checks+=1
  # Main loop: preserve original stage-death reset so every scenario keeps playing.
  if ctypes.c_uint16.from_address(ptr+DATA+0x585).value:
   lib.start_stage(scenario+1)
 print('stage',scenario+1,'ok')
print('PASS:',checks,'original-routine comparisons; exact game-data and background matches')
