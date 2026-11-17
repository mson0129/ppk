const offlineState = document.getElementById('offline-state');
const installButton = document.getElementById('install-app');
const updateButton = document.getElementById('apply-update');
let installPrompt = null, registration = null, applyingUpdate = false;
function offlineLabel() {
    if (offlineState.dataset.ready === 'true') offlineState.textContent = navigator.onLine ? '오프라인 준비됨' : '오프라인 실행';
}
window.addEventListener('beforeinstallprompt', event => {
    event.preventDefault(); installPrompt = event; installButton.hidden = false;
});
installButton.addEventListener('click', async () => {
    if (!installPrompt) return;
    const prompt = installPrompt; installPrompt = null; installButton.hidden = true;
    await prompt.prompt(); await prompt.userChoice;
});
window.addEventListener('appinstalled', () => { installPrompt = null; installButton.hidden = true; });
window.addEventListener('online', offlineLabel);
window.addEventListener('offline', offlineLabel);
function checkOfflineReady() {
    const worker = registration?.active;
    if (!worker) return;
    const channel = new MessageChannel();
    channel.port1.onmessage = event => {
        channel.port1.close();
        if (event.data?.ready) {
            offlineState.dataset.ready = 'true'; offlineState.dataset.version = event.data.version;
            offlineLabel();
        } else {
            offlineState.dataset.ready = 'false'; offlineState.textContent = '오프라인 저장 실패';
        }
    };
    worker.postMessage({type:'OFFLINE_STATUS'}, [channel.port2]);
}
function showUpdate() { updateButton.hidden = !(registration?.waiting && navigator.serviceWorker.controller); }
updateButton.addEventListener('click', () => {
    if (!registration?.waiting) return;
    applyingUpdate = true; updateButton.disabled = true;
    registration.waiting.postMessage({type:'SKIP_WAITING'});
});
if ('serviceWorker' in navigator && window.isSecureContext) {
    offlineState.hidden = false; offlineState.textContent = '오프라인 저장 중';
    navigator.serviceWorker.addEventListener('controllerchange', () => {
        if (applyingUpdate) location.reload(); else { showUpdate(); checkOfflineReady(); }
    });
    try {
        registration = await navigator.serviceWorker.register(new URL('./sw.js', import.meta.url), {scope:'./',updateViaCache:'none'});
        showUpdate();
        registration.addEventListener('updatefound', () => {
            const installing = registration.installing;
            installing?.addEventListener('statechange', () => {
                if (installing.state === 'installed') { showUpdate(); checkOfflineReady(); }
                if (installing.state === 'redundant' && offlineState.dataset.ready !== 'true') offlineState.textContent = '오프라인 저장 실패';
            });
        });
        await navigator.serviceWorker.ready;
        checkOfflineReady();
    } catch (error) {
        offlineState.textContent = '오프라인 저장 실패';
        offlineState.title = '온라인 상태에서 페이지를 다시 불러오세요.';
        console.warn('Offline setup failed:', error);
    }
}
