import assert from 'node:assert/strict';
import test from 'node:test';
import { GamepadInput } from '../web/port/gamepad-input.mjs';
export const checks = [];
function check(name, fn) { checks.push([name,fn]); test(name,fn); }
const pad = (buttons=[], axes=[0,0]) => ({connected:true,mapping:'standard',buttons:Array.from({length:17},(_,i)=>({pressed:buttons.includes(i),value:buttons.includes(i)?1:0})),axes});
check('A and B independently generate Space and preserve short-tap edges',()=>{
 for(const button of [0,1]) { const input=new GamepadInput(); const first=input.update([pad([button])]);assert(first.keys.has(57));assert(first.pressed.has(57));assert(!input.update([pad([button])]).pressed.has(57));assert.equal(input.update([pad()]).keys.size,0); }
});
check('releasing A while B is held does not release Space or trigger a second press',()=>{
 const input=new GamepadInput();input.update([pad([0])]);assert(!input.update([pad([0,1])]).pressed.has(57));const held=input.update([pad([1])]);assert(held.keys.has(57));assert(!held.pressed.has(57));
});
check('D-pad maps all four directions and combines movement with jump',()=>{
 const input=new GamepadInput();for(const [button,scan] of [[12,72],[13,80],[14,75],[15,77]])assert(input.update([pad([button,0])]).keys.has(scan));assert.deepEqual([...input.update([pad([14,1])]).keys].sort(),[57,75]);
});
check('left stick deadzone filters drift and allows diagonal movement',()=>{
 const input=new GamepadInput();assert.equal(input.update([pad([],[0.29,-0.29])]).keys.size,0);assert.deepEqual([...input.update([pad([],[-0.8,-0.8])]).keys].sort(),[72,75]);
});
check('Start toggles only on a new press',()=>{
 const input=new GamepadInput();assert(input.update([pad([9])]).pausePressed);assert(!input.update([pad([9])]).pausePressed);input.update([pad()]);assert(input.update([pad([9])]).pausePressed);
});
check('disconnect, sparse slots and background disable release all keys safely',()=>{
 const input=new GamepadInput();input.update([null,pad([0,14])]);assert.equal(input.update([null]).keys.size,0);assert.equal(input.update([pad([0])],false).keys.size,0);assert.equal(input.update([{...pad([0]),connected:false}]).connected,0);
});
check('multiple controllers combine held keys and ignore XR controllers',()=>{
 const input=new GamepadInput();assert.deepEqual([...input.update([pad([14]),pad([1])]).keys].sort(),[57,75]);assert.equal(input.update([{...pad([0]),mapping:'xr-standard'}]).keys.size,0);
});
