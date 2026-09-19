
const loaded = new Map();

export function import_and_start(url) {
    let started = loaded.get(url);
    if (!started) {
        started = (async () => {
            const mod = await import(url);
            await mod.default();
            mod.start();
        })();
        loaded.set(url, started);
    }
    return started;
}
