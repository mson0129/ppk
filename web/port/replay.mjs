// Input movies, not video. One mask per original fixed simulation tick.
export const REPLAY_SCANS = Object.freeze([75,77,72,80,57,28]);
export const MAX_FRAMES = 200000;
export const MAX_FILE_BYTES = 12 * 1024 * 1024;
const MAX_WALL = 24 * 60 * 60 * 1000;
const integer = (n,min,max) => Number.isSafeInteger(n) && n >= min && n <= max;
// Bit 6 preserves a Space-up/re-press between ticks (the title release latch).
export function applyReplayMask(game,mask) {
    if(mask & 64) game.key(57,0);
    REPLAY_SCANS.forEach((scan,bit)=>game.key(scan,(mask>>bit)&1));
}
export function validateReplay(value,engine) {
    const bad = () => { throw new Error('올바른 너구리 리플레이 파일이 아닙니다.'); };
    if (!value || value.format !== 'ppk-replay' || value.version !== 1 ||
        (typeof value.engine!=='string' || !/^[a-f0-9]{64}$/.test(value.engine)) || !integer(value.stage,1,50) ||
        !integer(value.frames,1,MAX_FRAMES) || !integer(value.duration,0,MAX_WALL) ||
        typeof value.createdAt !== 'string' || value.createdAt.length > 64 ||
        !Array.isArray(value.times) || value.times.length !== value.frames ||
        !Array.isArray(value.inputs) || !value.inputs.length || value.inputs.length > value.frames ||
        !Array.isArray(value.hashes) || !value.hashes.length || value.hashes.length > value.frames) bad();
    let previous = -1;
    for (const time of value.times) { if (!integer(time,0,value.duration) || time < previous) bad(); previous=time; }
    previous=-1;
    for (const pair of value.inputs) {
        if (!Array.isArray(pair) || pair.length!==2 || !integer(pair[0],0,value.frames-1) || pair[0]<=previous || !integer(pair[1],0,127)) bad();
        previous=pair[0];
    }
    if (value.inputs[0][0]!==0) bad();
    previous=0;
    for (const pair of value.hashes) {
        if (!Array.isArray(pair) || pair.length!==2 || !integer(pair[0],1,value.frames) || pair[0]<=previous || !integer(pair[1],0,0xffffffff)) bad();
        previous=pair[0];
    }
    if (previous!==value.frames || !value.timer || !integer(value.timer.start,0,value.duration) ||
        !(value.timer.finish===null || integer(value.timer.finish,value.timer.start,value.duration)) ||
        !integer(value.timer.elapsed,0,value.duration+1) ||
        Math.abs(value.timer.elapsed-((value.timer.finish??value.duration)-value.timer.start))>1 ||
        typeof value.completed!=='boolean') bad();
    if (engine && value.engine!==engine) throw new Error('현재 게임 버전과 다른 리플레이입니다. 기록한 버전에서 재생해야 합니다.');
    // Retain only the format's data fields; imported JSON is never executed.
    return {format:'ppk-replay',version:1,engine:value.engine,stage:value.stage,createdAt:value.createdAt,
        frames:value.frames,duration:value.duration,times:[...value.times],inputs:value.inputs.map(p=>[...p]),
        hashes:value.hashes.map(p=>[...p]),timer:{start:value.timer.start,finish:value.timer.finish,elapsed:value.timer.elapsed},completed:value.completed};
}
export class ReplayRecorder {
    constructor(engine,stage,now) {
        this.engine=engine;this.stage=stage;this.origin=now;this.createdAt=new Date().toISOString();
        this.times=[];this.inputs=[];this.hashes=[];this.lastMask=-1;this.lastHash=0;
        this.started=null;this.finished=null;this.completed=false;this.full=false;
    }
    offset(now) { return Math.max(0,Math.min(MAX_WALL,Math.round(now-this.origin))); }
    start(now) { if(this.started===null)this.started=this.offset(now); }
    finish(now) { if(this.started!==null && this.finished===null){this.finished=this.offset(now);this.completed=true;} }
    push(mask,now,hash) {
        if(this.full)return false;
        const frame=this.times.length;
        if(frame>=MAX_FRAMES || now-this.origin>MAX_WALL){this.full=true;return false;}
        if(mask!==this.lastMask){this.inputs.push([frame,mask]);this.lastMask=mask;}
        this.times.push(Math.max(this.times.at(-1)??0,this.offset(now)));this.lastHash=hash>>>0;
        if(this.times.length%180===0)this.hashes.push([this.times.length,this.lastHash]);
        return true;
    }
    snapshot(now,elapsed) {
        if(!this.times.length || this.started===null)return null;
        const duration=this.full?this.times.at(-1):Math.max(this.times.at(-1),this.offset(now));
        if(duration<this.started)return null;
        const finish=this.full?null:this.finished;
        const hashes=this.hashes.map(p=>[...p]);
        if(hashes.at(-1)?.[0]!==this.times.length)hashes.push([this.times.length,this.lastHash]);
        return validateReplay({format:'ppk-replay',version:1,engine:this.engine,stage:this.stage,createdAt:this.createdAt,
            frames:this.times.length,duration,times:this.times,inputs:this.inputs,hashes,
            timer:{start:this.started,finish,elapsed:this.full||elapsed===undefined?(finish??duration)-this.started:elapsed},completed:this.completed&&!this.full});
    }
}
export class ReplayPlayback {
    constructor(data,engine) {this.data=validateReplay(data,engine);this.frame=0;this.wall=0;this.input=0;this.checkpoint=0;this.mask=0;this.rate=1;}
    get done(){return this.frame===this.data.frames && this.wall>=this.data.duration;}
    advance(delta,tick) {
        this.wall=Math.min(this.data.duration,this.wall+Math.max(0,delta)*this.rate);
        let budget=16;
        while(this.frame<this.data.frames && this.data.times[this.frame]<=this.wall && budget-->0){
            while(this.input<this.data.inputs.length && this.data.inputs[this.input][0]===this.frame)this.mask=this.data.inputs[this.input++][1];
            const actual=tick(this.mask,this.frame)>>>0;this.frame++;
            if(this.data.hashes[this.checkpoint]?.[0]===this.frame){
                if(actual!==this.data.hashes[this.checkpoint][1])throw new Error('리플레이 상태 검증에 실패했습니다. 재생을 중단했습니다.');
                this.checkpoint++;
            }
        }
    }
    get visibleWall(){return this.frame<this.data.frames && this.data.times[this.frame]<=this.wall ? this.data.times[Math.max(0,this.frame-1)] : this.wall;}
    elapsed(){if(this.done)return this.data.timer.elapsed;return Math.max(0,Math.min(this.visibleWall,this.data.timer.finish??this.data.duration)-this.data.timer.start);}
    fraction(stepMs){return this.done?1000:Math.min(1000,Math.max(0,(this.visibleWall-(this.data.times[this.frame-1]??0))/stepMs*1000));}
}
export class ReplayStore {
    async database(){
        if(this.db)return this.db;
        this.db=await new Promise((resolve,reject)=>{
            const request=indexedDB.open('ppk-replays',1);
            request.onupgradeneeded=()=>request.result.createObjectStore('movies');
            request.onsuccess=()=>resolve(request.result);request.onerror=()=>reject(request.error);
        });return this.db;
    }
    async load(){const db=await this.database();return new Promise((resolve,reject)=>{const request=db.transaction('movies').objectStore('movies').get('latest');request.onsuccess=()=>resolve(request.result??null);request.onerror=()=>reject(request.error);});}
    async save(data){const db=await this.database();return new Promise((resolve,reject)=>{const transaction=db.transaction('movies','readwrite');transaction.objectStore('movies').put(data,'latest');transaction.oncomplete=()=>resolve();transaction.onerror=()=>reject(transaction.error);transaction.onabort=()=>reject(transaction.error);});}
}
