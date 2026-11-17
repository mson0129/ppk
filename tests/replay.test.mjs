import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {ReplayRecorder,ReplayPlayback,ReplayArchive,validateReplay,applyReplayMask,MAX_FRAMES} from '../web/port/replay.mjs';
const bytes=await readFile(new URL('../dist/port/ppk.wasm',import.meta.url));
const engine=createHash('sha256').update(bytes).digest('hex');
const step=1000*65536/1193182;
const fresh=async(stage=1)=>{const {instance}=await WebAssembly.instantiate(bytes,{});instance.exports.start_stage(stage);instance.exports.show_intro();return instance.exports;};
async function recording(stage=1){
 const game=await fresh(stage),rec=new ReplayRecorder(engine,stage,1000),digests=[];let wall=1000;
 for(let i=0;i<720;i++){
  if(i===10)rec.start(wall+15.123);
  wall+=step+(i===200?4200:0)+(i===500?1700:0);
  const phase=game.get_phase();
  const mask=(i<10?0:i<30?17:i%60<20?1:i%60<40?2:0)|((i>=40&&i%31<4)||(phase===2&&i%20<5)||(phase===4&&i%20<5)?16:0);
  applyReplayMask(game,mask);game.tick();assert.equal(game.get_error(),0);
  digests.push(game.replay_hash()>>>0);rec.push(mask,wall,digests.at(-1));
  // Rendering and browser audio reads must not change a movie's logical state.
  game.render_interpolated(432);game.audio_frequency();assert.equal(game.replay_hash()>>>0,digests.at(-1));
 }
 const elapsed=Math.floor(wall-(1000+rec.started));
 return {data:rec.snapshot(wall,elapsed),digests,last:game.replay_hash()>>>0};
}
test('WASM replay reproduces every tick including held title Space, deaths and wall-clock gaps',async()=>{
 const {data,digests,last}=await recording(),game=await fresh(),movie=new ReplayPlayback(JSON.parse(JSON.stringify(data)),engine);
 let frame=0,iterations=0;
 while(!movie.done&&iterations++<10000)movie.advance(16.7,mask=>{applyReplayMask(game,mask);game.tick();const hash=game.replay_hash()>>>0;assert.equal(hash,digests[frame++]);return hash;});
 assert(movie.done);assert.equal(frame,data.frames);assert.equal(game.replay_hash()>>>0,last);assert.equal(movie.elapsed(),data.timer.elapsed);
});
test('2x and 4x playback preserve state, selected stage and final RTA',async()=>{
 const {data,last}=await recording(27);
 for(const rate of [2,4]){const game=await fresh(27),movie=new ReplayPlayback(data,engine);movie.rate=rate;let count=0;
  while(!movie.done&&count++<10000)movie.advance(16.7,mask=>{applyReplayMask(game,mask);game.tick();return game.replay_hash();});
  assert(movie.done);assert.equal(game.replay_hash()>>>0,last);assert.equal(movie.elapsed(),data.timer.elapsed);
 }
});
test('completed movie retains exact millisecond timer result independently of playback rate',()=>{
 const rec=new ReplayRecorder(engine,1,0.4);rec.start(100.8);rec.push(16,110.2,12);rec.finish(200.7);
 const data=rec.snapshot(300,99);assert.equal(data.timer.elapsed,99);
 const movie=new ReplayPlayback(data,engine);movie.advance(1000,()=>12);assert(movie.done);assert.equal(movie.elapsed(),99);
});
test('malformed, excessive, unordered and incompatible files are rejected',async()=>{
 const {data}=await recording();
 for(const edit of [v=>v.frames=MAX_FRAMES+1,v=>v.times[3]=-1,v=>v.times[2]=v.duration+1,v=>v.inputs[0][1]=128,v=>v.inputs[0][0]=1,v=>v.inputs.push(v.inputs[0]),v=>v.hashes.pop(),v=>v.timer.elapsed+=30,v=>v.timer.finish=-1]){
  const copy=structuredClone(data);edit(copy);assert.throws(()=>validateReplay(copy,engine));
 }
 assert.throws(()=>validateReplay(data,'a'.repeat(64)),/다른 리플레이/);
});
test('state mismatch stops playback instead of silently showing a different run',async()=>{
 const {data}=await recording(),movie=new ReplayPlayback(data,engine);let count=0;
 assert.throws(()=>{while(!movie.done&&count++<10000)movie.advance(1000,()=>0);},/검증/);
});
test('long gaps and adversarial same-time ticks are processed with bounded per-frame work',()=>{
 const rec=new ReplayRecorder(engine,1,0);rec.start(0);for(let i=0;i<100;i++)rec.push(0,0,7);
 const movie=new ReplayPlayback(rec.snapshot(10000),engine);let calls=0;movie.advance(10000,()=>{calls++;return 7;});assert.equal(calls,16);assert(!movie.done);
});

test('Space released and pressed again between two ticks replays the intentional jump',async()=>{
 const game=await fresh(),rec=new ReplayRecorder(engine,1,0);rec.start(0);
 applyReplayMask(game,16);game.tick();rec.push(16,1,game.replay_hash());
 let wall=1;while(game.get_phase()===5){applyReplayMask(game,16);game.tick();wall+=55;rec.push(16,wall,game.replay_hash());}
 game.key(57,0);game.key(57,1);game.tick();wall+=55;rec.push(16|64,wall,game.replay_hash());
 applyReplayMask(game,16);game.tick();wall+=55;rec.push(16,wall,game.replay_hash());
 assert.equal(game.debug_state(0x563),166);
 const original=game.replay_hash(),movie=new ReplayPlayback(rec.snapshot(wall),engine),replayed=await fresh();
 while(!movie.done)movie.advance(wall,mask=>{applyReplayMask(replayed,mask);replayed.tick();return replayed.replay_hash();});
 assert(movie.done);assert.equal(replayed.replay_hash(),original);assert.equal(replayed.debug_state(0x563),166);
});

test('recording limit produces a valid truncated movie even if completion happens later',()=>{
 const rec=new ReplayRecorder(engine,1,0);rec.start(0);
 for(let i=0;i<MAX_FRAMES;i++)rec.push(0,0,7);
 rec.finish(500);assert.equal(rec.push(0,500,7),false);
 const data=rec.snapshot(1000,500);assert.equal(data.frames,MAX_FRAMES);assert.equal(data.completed,false);assert.equal(data.timer.finish,null);validateReplay(data,engine);
 const late=new ReplayRecorder(engine,1,0);for(let i=0;i<MAX_FRAMES;i++)late.push(0,0,7);late.start(2);late.push(16,3,7);assert.equal(late.snapshot(3),null);
});

test('autosave uses a fresh post-tick timestamp rather than the earlier RAF timestamp',()=>{
 const rec=new ReplayRecorder(engine,1,0);rec.start(0);rec.push(0,5004,123);
 // This is the pre-fix failure: a 5000ms RAF timestamp with a 5004ms game tick.
 assert.throws(()=>rec.snapshot(5000,5000));
 const saved=[],errors=[];
 const archive=new ReplayArchive({clock:()=>5005,elapsed:now=>now,save:data=>saved.push(data),onError:e=>errors.push(e)});
 const movie=archive.capture(rec);assert.equal(movie.duration,5005);assert.equal(movie.timer.elapsed,5005);assert.equal(saved.length,1);assert.equal(errors.length,0);
});
test('snapshot or synchronous storage failures cannot escape autosaving and stop gameplay',()=>{
 let errors=0,ticks=0;
 const archive=new ReplayArchive({clock:()=>5005,elapsed:now=>now,save:()=>{throw Error('storage failure');},onError:()=>{errors++;}});
 for(const recorder of [{snapshot:()=>{throw Error('snapshot failure');}},{snapshot:()=>({})}]){
  assert.equal(archive.capture(recorder),null);
  ticks++; // The caller can finish the frame and schedule the next one.
 }
 assert.equal(ticks,2);assert.equal(errors,2);
});
