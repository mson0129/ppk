/* Version and SHA-256 hashes are generated from the complete build. */
const VERSION = __VERSION__;
const ASSETS = __ASSETS__;
const BASE = self.registration.scope;
const PREFIX = 'ppk-offline-' + encodeURIComponent(new URL(BASE).pathname) + '-';
const CACHE = PREFIX + VERSION;
const byUrl = new Map(ASSETS.map(asset => [new URL(asset.url, BASE).href, asset]));
async function verifiedFetch(asset) {
    const response = await fetch(new Request(new URL(asset.url, BASE), {cache:'reload', credentials:'same-origin'}));
    if (!response.ok || response.type === 'opaque') throw new Error('Offline asset unavailable: '+asset.url);
    const digest = await crypto.subtle.digest('SHA-256', await response.clone().arrayBuffer());
    const actual = Array.from(new Uint8Array(digest), n => n.toString(16).padStart(2,'0')).join('');
    if (actual !== asset.sha256) throw new Error('Offline asset version mismatch: '+asset.url);
    return response;
}
self.addEventListener('install', event => {
    event.waitUntil((async () => {
        const cache = await caches.open(CACHE);
        const results = await Promise.allSettled(ASSETS.map(async asset => {
            await cache.put(new URL(asset.url, BASE).href, await verifiedFetch(asset));
        }));
        if (results.some(result => result.status === 'rejected')) {
            await caches.delete(CACHE);
            throw new Error('Incomplete offline package; keeping the previous version.');
        }
        // Do not replace an active game; an update waits for user consent or closed tabs.
    })());
});
self.addEventListener('activate', event => {
    event.waitUntil((async () => {
        for (const name of await caches.keys()) if (name.startsWith(PREFIX) && name !== CACHE) await caches.delete(name);
        await self.clients.claim();
    })());
});
self.addEventListener('fetch', event => {
    if (event.request.method !== 'GET') return;
    const url = new URL(event.request.url); url.search = ''; url.hash = '';
    let target = url.href;
    if (event.request.mode === 'navigate' && target === BASE) target = new URL('index.htm', BASE).href;
    const asset = byUrl.get(target);
    if (!asset) return; // Other pages and the online-only DOS reference are untouched.
    event.respondWith((async () => {
        const cache = await caches.open(CACHE);
        const cached = await cache.match(target);
        if (cached) return cached;
        // Repair an evicted file online, but reject a mismatched/newer version.
        const response = await verifiedFetch(asset);
        await cache.put(target, response.clone());
        return response;
    })());
});
self.addEventListener('message', event => {
    if (event.data?.type === 'SKIP_WAITING') { event.waitUntil(self.skipWaiting()); return; }
    if (event.data?.type === 'OFFLINE_STATUS') event.waitUntil((async () => {
        const cache = await caches.open(CACHE);
        const matches = await Promise.all(ASSETS.map(asset => cache.match(new URL(asset.url, BASE).href)));
        event.ports[0]?.postMessage({ready:matches.every(Boolean), version:VERSION, files:ASSETS.length});
    })());
});
