async function requestJson(path, options = {}) {
    const response = await fetch(path, options);
    const data = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(data.error || `HTTP ${response.status}`);
    return data;
}

function jsonOptions(body) {
    return {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    };
}

export function getState() {
    return requestJson('/api/state');
}

export function postAction(body) {
    return requestJson('/api/action', jsonOptions(body));
}

export function postNew(body) {
    return requestJson('/api/new', jsonOptions(body));
}

export function postUndo(body) {
    return requestJson('/api/undo', jsonOptions(body));
}
