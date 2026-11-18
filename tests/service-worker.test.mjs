import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import fs from 'node:fs/promises';
import {webcrypto} from 'node:crypto';
const root = new URL('../',import.meta.url);
const source = await fs.readFile(new URL('dist/sw.js',root),'utf8');
const assets = JSON.parse(source.match(/const ASSETS = (.+);/)[1]);
const files = new Map(await Promise.all(assets.map(async a => [a.url,await fs.readFile(new URL('dist/'+a.url,root))])));
const base='https://example.test/ppk/';
export const checks=[];
function check(name,fn){checks.push([name,fn]);test(name,fn);}
function harness({missing=null,mismatch=null,store=new Map()}={}){
 const handlers={},state={offline:false,network:0,skipped:0,claimed:0};
 const caches={async open(name){if(!store.has(name))store.set(name,new Map());const entries=store.get(name);return {async put(key,value){entries.set(String(key),value.clone());},async match(key){return entries.get(String(key))?.clone();}};},async keys(){return [...store.keys()];},async delete(name){return store.delete(name);}};
 const self={registration:{scope:base},clients:{async claim(){state.claimed++;}},async skipWaiting(){state.skipped++;},addEventListener(name,fn){handlers[name]=fn;}};
 const fetch=async request=>{state.network++;if(state.offline)throw new Error('offline');const name=new URL(request.url).pathname.replace('/ppk/','');if(name===missing)return new Response('missing',{status:404});return new Response(name===mismatch?'corrupt':files.get(name));};
 vm.runInNewContext(source,{self,caches,fetch,crypto:webcrypto,Request,Response,URL,console});
 async function dispatch(name,extra={}){const work=[];handlers[name]({...extra,waitUntil:p=>work.push(p)});await Promise.all(work);}
 async function get(path,{mode='navigate',method='GET'}={}){let response;handlers.fetch({request:{url:new URL(path,base).href,method,mode},respondWith:p=>{response=p;}});return response?await response:undefined;}
 async function status(){let result;await dispatch('message',{data:{type:'OFFLINE_STATUS'},ports:[{postMessage:value=>{result=value;}}]});return result;}
 return {dispatch,get,status,state,store};
}
check('complete verified package becomes offline-ready without forced activation',async()=>{
 const h=harness();await h.dispatch('install');assert.equal((await h.status()).ready,true);assert.equal(h.state.skipped,0);assert.equal(h.state.network,assets.length);
});
check('offline navigation, query URLs and WASM return cached original bytes without network',async()=>{
 const h=harness();await h.dispatch('install');h.state.offline=true;const count=h.state.network;
 assert.equal(await (await h.get('index.html?launch=pwa')).text(),files.get('index.html').toString());
 assert.deepEqual(new Uint8Array(await (await h.get('port/ppk.wasm',{mode:'cors'})).arrayBuffer()),new Uint8Array(files.get('port/ppk.wasm')));
 assert.equal(await (await h.get('./')).text(),files.get('index.html').toString());
 assert.equal(await (await h.get('port/replay.mjs',{mode:'cors'})).text(),files.get('port/replay.mjs').toString());
 assert.deepEqual(new Uint8Array(await (await h.get('icons/apple-touch-icon.png?v=2',{mode:'cors'})).arrayBuffer()),new Uint8Array(files.get('icons/apple-touch-icon.png')));assert.equal(h.state.network,count);
});
check('other pages and non-GET requests are not intercepted',async()=>{
 const h=harness();assert.equal(await h.get('dos.htm'),undefined);assert.equal(await h.get('../index.html'),undefined);assert.equal(await h.get('index.html',{method:'POST'}),undefined);
});
check('404 during install removes partial new cache and preserves an existing version',async()=>{
 const store=new Map([['ppk-offline-%2Fppk%2F-old',new Map()]]);const h=harness({missing:'port/ppk.wasm',store});await assert.rejects(h.dispatch('install'));assert.deepEqual([...store.keys()],['ppk-offline-%2Fppk%2F-old']);
});
check('mixed or corrupt deployment fails integrity checks instead of activating',async()=>{
 const h=harness({mismatch:'port/player.js'});await assert.rejects(h.dispatch('install'));assert.equal(h.store.size,0);
});
check('activation cleans only this app scope; applying update is explicit',async()=>{
 const store=new Map([['ppk-offline-%2Fppk%2F-old',new Map()],['another-app',new Map()],['ppk-offline-%2Fother%2F-old',new Map()]]);const h=harness({store});await h.dispatch('install');await h.dispatch('activate');assert(!store.has('ppk-offline-%2Fppk%2F-old'));assert(store.has('another-app'));assert(store.has('ppk-offline-%2Fother%2F-old'));assert.equal(h.state.claimed,1);assert.equal(h.state.skipped,0);await h.dispatch('message',{data:{type:'SKIP_WAITING'}});assert.equal(h.state.skipped,1);
});
check('evicted WASM can be repaired only with the matching version',async()=>{
 const h=harness();await h.dispatch('install');const cache=[...h.store.values()][0];cache.delete(base+'port/ppk.wasm');h.state.offline=true;assert.equal((await h.status()).ready,false);await assert.rejects(h.get('port/ppk.wasm',{mode:'cors'}));h.state.offline=false;assert.equal((await h.get('port/ppk.wasm',{mode:'cors'})).ok,true);assert.equal((await h.status()).ready,true);
});
