from pathlib import Path
from capstone import *
import sys,struct
b=(Path(__file__).resolve().parents[1]/'analysis/PPK-unpacked.EXE').read_bytes()
base=struct.unpack_from('<H',b,8)[0]*16
seg=int(sys.argv[1],16) if len(sys.argv)>1 else 0x327
start=int(sys.argv[2],16) if len(sys.argv)>2 else 0
end=int(sys.argv[3],16) if len(sys.argv)>3 else start+0x100
md=Cs(CS_ARCH_X86,CS_MODE_16)
for i in md.disasm(b[base+seg*16+start:base+seg*16+end],start):
 print(f'{seg:04x}:{i.address:04x} {i.mnemonic:8} {i.op_str}')
