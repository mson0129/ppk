"""Ahead-of-time translation of PPK's game routines to C control flow.
Only reachable game functions are translated. DOS/runtime/graphics calls are replaced.
There is no instruction decoder or instruction dispatch loop in the resulting build.
"""
from pathlib import Path
from collections import Counter
import struct
from capstone import *
from capstone.x86 import *
root=Path(__file__).resolve().parents[1]
raw=(root/'analysis/PPK-unpacked.EXE').read_bytes()
image=raw[struct.unpack_from('<H',raw,8)[0]*16:]
md=Cs(CS_ARCH_X86, CS_MODE_16); md.detail=True
roots=[0x4db,0x9e9,0xd1e,0x11c1,0x12f5,0x1429,0xd98,0x1cca,0x1db1,0x1e09,0x15a3,0x1e7a,0x1eeb,0x1f34,0x260d]
# Replaced functions: speaker queue, BIOS palette, fatal error.
replaced={0xc6,0x19b,0x1d1,0x2e6,0x2437}
funcs={}; pending=list(roots); counts=Counter(); external=set()
def ins(off):
 return next(md.disasm(image[0x3270+off:0x3270+off+16],off,count=1))
while pending:
 entry=pending.pop()
 if entry in funcs or entry in replaced: continue
 seen={}; todo=[entry]
 while todo:
  pc=todo.pop()
  while pc not in seen:
   i=ins(pc); seen[pc]=i; counts[i.mnemonic]+=1
   nxt=pc+i.size
   if i.mnemonic in ['retf','ret','iret']: break
   if i.mnemonic=='call':
    target=i.operands[0].imm
    if target not in funcs: pending.append(target)
   if i.mnemonic=='lcall': external.add((i.operands[0].imm,i.operands[1].imm))
   if i.mnemonic.startswith('j') or i.mnemonic=='loop':
    if i.operands[0].type==X86_OP_IMM: targets=[i.operands[0].imm]
    else:
     # Tile-character switch table, 22 entries.
     assert pc==0x557
     targets=list(struct.unpack_from('<22H',image,0x3270+0x9bd))
    todo.extend(targets)
    if i.mnemonic=='jmp': break
   pc=nxt
 funcs[entry]=seen
print('Functions:',len(funcs),'instructions:',sum(map(len,funcs.values())),'operations:',counts)
print('External calls:', sorted(external))
regs={'ax':'ax','bx':'bx','cx':'cx','dx':'dx','si':'si','di':'di','bp':'bp','sp':'sp','ds':'ds','es':'es','ss':'ss','cs':'cs'}
low={'al':'ax','bl':'bx','cl':'cx','dl':'dx'}; high={'ah':'ax','bh':'bx','ch':'cx','dh':'dx'}
def reg(r):
 n=md.reg_name(r)
 if n in regs: return n
 if n in low: return f'({low[n]} & 255)'
 if n in high: return f'({high[n]} >> 8)'
 raise ValueError(n)
def addr(o):
 m=o.mem; terms=[str(m.disp)]
 for r in [m.base,m.index]:
  if r: terms.append(reg(r))
 off='(uint16_t)('+'+'.join(terms)+')'
 sg=reg(m.segment) if m.segment else ('ss' if m.base in [X86_REG_BP,X86_REG_SP] else 'ds')
 return f'(({sg} << 4) + {off})'
def val(o):
 if o.type==X86_OP_REG: return reg(o.reg)
 if o.type==X86_OP_IMM: return str(o.imm)
 if o.type==X86_OP_MEM: return f'rd{o.size*8}({addr(o)})'
 raise ValueError(o)
def put(o,v):
 if o.type==X86_OP_MEM: return f'wr{o.size*8}({addr(o)}, {v});'
 n=md.reg_name(o.reg)
 if n in low: return f'{low[n]}=({low[n]} & 0xff00) | (({v}) & 255);'
 if n in high: return f'{high[n]}=({high[n]} & 255) | ((({v}) & 255)<<8);'
 return f'{n}=(uint16_t)({v});'
def emit(i):
 m=i.mnemonic; o=i.operands; v=[val(x) for x in o] if m not in ['les','lds','lea'] else []
 if m=='mov': return put(o[0],v[1])
 if m=='lea':
  a=addr(o[1]); return put(o[0],a.split(' + ',1)[1][:-1])
 if m in ['les','lds']:
  a=addr(o[1]); sg='es' if m=='les' else 'ds'
  return f'a={a}; '+put(o[0],'rd16(a)')+f' {sg}=rd16(a+2);'
 if m=='push': return f'push16({v[0]});'
 if m=='pop': return put(o[0],'pop16()')
 if m in ['add','sub','cmp','and','or','xor','test','adc','sbb']:
  op={'add':'+','sub':'-','cmp':'-','and':'&','or':'|','xor':'^','test':'&','adc':'+','sbb':'-'}[m]
  extra=' + cf' if m=='adc' else ' - cf' if m=='sbb' else ''
  t=f'a={v[0]}; b={v[1]}; t=a {op} b{extra}; '
  flags=f'flags_{"add" if m in ["add","adc"] else "sub" if m in ["sub","cmp","sbb"] else "logic"}(a,b,t,{o[0].size*8}); '
  return t+flags+('' if m in ['cmp','test'] else put(o[0],'t'))
 if m in ['inc','dec']:
  return f'a={v[0]}; b=1; t=a {"+" if m=="inc" else "-"} 1; saved_cf=cf; flags_{"add" if m=="inc" else "sub"}(a,b,t,{o[0].size*8}); cf=saved_cf; '+put(o[0],'t')
 if m=='neg': return f'a=0; b={v[0]}; t=0-b; flags_sub(a,b,t,{o[0].size*8}); '+put(o[0],'t')
 if m=='not': return put(o[0],f'~({v[0]})')
 if m in ['shl','shr','sar']:
  return f't=shift_{m}({v[0]},{v[1]},{o[0].size*8}); '+put(o[0],'t')
 if m in ['imul','mul']:
  assert len(o)==1
  if m=='imul': return f't=(uint32_t)((int32_t)(int16_t)ax*(int16_t)({v[0]})); ax=t; dx=t>>16; cf=of=((int32_t)t!=(int16_t)ax);'
  return f't=(uint32_t)ax*({v[0]}); ax=t; dx=t>>16; cf=of=(dx!=0);'
 if m in ['idiv','div']:
  if m=='idiv': return f'sdiv32((int32_t)(((uint32_t)dx<<16)|ax), (int16_t)({v[0]}));'
  return f't=((uint32_t)dx<<16)|ax; a={v[0]}; ax=t/a; dx=t%a;'
 if m in ['cdq','cwd']: return 'dx=(ax & 0x8000)?0xffff:0;'
 if m=='cbw': return 'ax=(uint16_t)(int16_t)(int8_t)(ax & 255);'
 if m=='xchg': return f't={v[0]}; '+put(o[0],v[1])+put(o[1],'t')
 if m=='call':
  target=o[0].imm
  return f'push16({i.address+i.size}); '+(f'replace_game(0x{target:x}); sp+=4;' if target in replaced else f'game_{target:04x}();')
 if m=='lcall':
  sg,ip=o[0].imm,o[1].imm
  return f'platform_call(0x{sg:x},0x{ip:x});'
 if m in ['retf','ret']:
  return f'sp+={4 if m=="retf" else 2}; return;'
 if m in ['cli','sti','nop','cld']: return ';'
 if m=='in': return put(o[0],'0')
 if m=='out': return f'speaker_port({v[0]}, {v[1]});'
 cond={'je':'zf','jne':'!zf','jb':'cf','jae':'!cf','jbe':'(cf||zf)','ja':'(!cf&&!zf)','jl':'(sf!=of)','jge':'(sf==of)','jle':'(zf||(sf!=of))','jg':'(!zf&&(sf==of))','js':'sf','jns':'!sf','jcxz':'cx==0'}
 if m in cond: return f'if ({cond[m]}) goto L{o[0].imm:04x};'
 if m=='loop': return f'cx--; if(cx) goto L{o[0].imm:04x};'
 if m=='jmp':
  if o[0].type==X86_OP_IMM: return f'goto L{o[0].imm:04x};'
  targets=struct.unpack_from('<22H',image,0x3270+0x9bd)
  return 'switch (bx>>1) {'+''.join(f'case {n}: goto L{t:04x};' for n,t in enumerate(targets))+'default: fail(0x557); return;}'
 raise ValueError(f'{i.address:x} {m} {i.op_str}')
lines=['/* Generated by tools/translate.py. Original PPK game control flow, not a CPU interpreter. */']
for entry in sorted(funcs): lines.append(f'static void game_{entry:04x}(void);')
for entry,body in sorted(funcs.items()):
 lines.append(f'static void game_{entry:04x}(void) {{\n uint32_t a=0,b=0,t=0; int saved_cf=0;')
 for pc,i in sorted(body.items()):
  lines.append(f'L{pc:04x}: /* {i.mnemonic} {i.op_str} */')
  if pc==0xcce: lines.append('wr16(DATA+0x1596,rd16(DATA+0x1596)+1); /* Browser tick replaces the BIOS timer wait. */')
  lines.append(emit(i))
 lines.append('}')
(root/'src/game.generated.c').write_text('\n'.join(lines)+'\n')
# Keep just the initialized game data; executable code is never loaded at runtime.
(root/'assets/initial-data.bin').write_bytes(image[0xf0a0:0xf0a0+0xc44])
with (root/'analysis/routines.tsv').open('w') as f:
 for entry,body in sorted(funcs.items()): f.write(f'0327:{entry:04x}\t{len(body)} instructions\n')
