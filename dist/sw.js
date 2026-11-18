/* Version and SHA-256 hashes are generated from the complete build. */
const VERSION = "29fd6d8ca3d16609041c";
const ASSETS = [{"url": "index.html", "sha256": "7f0e3201cb5f497cf216f72a7c7a91dc3f82ef1ab02fbd37383939e238f9247d"}, {"url": "pwa.js", "sha256": "57bdeb0eb61fd2173d8a1bfb44eace11b7bce463875d70ab8dd4823d836a5d06"}, {"url": "app.webmanifest", "sha256": "62e00c89250053ae0448a027fa2b6bf29453a1b63469c55e4723a16da9e31454"}, {"url": "port/gamepad-input.mjs", "sha256": "ec7eb7fbb6e8eab6fa172132502368ea1c772abc4607674d438d5050fe57440f"}, {"url": "port/player.js", "sha256": "b9be36549d68a55c3006a72b8865ed8b4f354224bf1ddc29afab576d1f1b9af8"}, {"url": "port/ppk.wasm", "sha256": "2c716e7b988cefa8385d1358d36a25dfeaa169ddb805c370ab41dcb554a8c25b"}, {"url": "port/replay.mjs", "sha256": "923d31566bab3da439b8e44e361bbcac5da5acc3536cf1e2756e8c44f3f8d22c"}, {"url": "port/speedrun-timer.mjs", "sha256": "de6758036e896bcefbf960ef44faf08b5303be5452cd47145137676a5353b3d4"}, {"url": "icons/apple-touch-icon.png", "sha256": "59405b141195fe4f6088881b71a92dbabab66ba9e6ab1d458c3baabd501355ce"}, {"url": "icons/icon-192.png", "sha256": "6257e33b59886d263965ffc26edd21076fd292a6706af1f0b75c5b7c4e4a496d"}, {"url": "icons/icon-512.png", "sha256": "f834327e01c7cb95733ad2169ffa3d18f561193e78accac518e2827bf2edebee"}, {"url": "icons/maskable-512.png", "sha256": "2523620c72aa7477c495426e4f57aeacbf338363d5d8a56dc529ddc32df43535"}];
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
    if (event.request.mode === 'navigate' && target === BASE) target = new URL('index.html', BASE).href;
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
