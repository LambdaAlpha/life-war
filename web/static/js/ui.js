import { initBoard, renderBoard, setBoardBusy } from './board.js';

let commandHandler;
let currentState = null;
let selectedColor = 'black';
let busy = false;

const elements = {};

export function init(onCommand) {
    commandHandler = onCommand;
    bindElements();
    initBoard(elements.board, point => {
        void commandHandler({ type: 'action', action: { type: 'place', ...point } });
    });

    elements.pass.addEventListener('click', () => {
        void commandHandler({ type: 'action', action: { type: 'pass' } });
    });
    elements.undo.addEventListener('click', () => void commandHandler({ type: 'undo' }));
    elements.newGame.addEventListener('click', () => openModal(elements.newModal));
    elements.rematch.addEventListener('click', () => openModal(elements.newModal));
    elements.rules.addEventListener('click', () => openModal(elements.rulesModal));
    elements.colorBlack.addEventListener('click', () => chooseColor('black'));
    elements.colorWhite.addEventListener('click', () => chooseColor('white'));
    elements.create.addEventListener('click', async () => {
        elements.modalError.textContent = '';
        const size = Number(elements.settingSize.value);
        const success = await commandHandler({ type: 'new_game', size });
        if (success) closeModals();
    });
    document.querySelectorAll('[data-close-modal]').forEach(button => {
        button.addEventListener('click', closeModals);
    });
    elements.overlay.addEventListener('click', event => {
        if (event.target === elements.overlay) closeModals();
    });
    window.addEventListener('keydown', event => {
        if (event.key === 'Escape') closeModals();
    });
}

export function render(state, result = null) {
    currentState = state;
    if (state.phase === 'layout') selectedColor = state.current_player;
    renderBoard(state, selectedColor, result?.changes || []);
    renderPhase(state);
    renderPopulation(state);
    renderHistory(state);
    renderResult(state);
    updateControls();
}

export function setBusy(value) {
    busy = value;
    document.body.classList.toggle('busy', value);
    setBoardBusy(value);
    updateControls();
}

export function setStatus(message, isError = false) {
    elements.status.textContent = message;
    elements.statusDot.classList.toggle('error', isError);
}

export function setModalError(message) {
    elements.modalError.textContent = message;
}

function bindElements() {
    const ids = [
        'board', 'btn-pass', 'btn-undo', 'btn-new', 'btn-rules', 'btn-rematch',
        'phase-index', 'phase-name', 'phase-hint', 'turn-card', 'turn-name',
        'black-count', 'white-count', 'black-track', 'white-track', 'generation',
        'pass-count', 'board-size', 'color-panel', 'color-black', 'color-white',
        'board-kicker', 'board-title', 'status', 'status-dot', 'result-banner',
        'result-title', 'result-detail', 'history-count', 'history-list',
        'modal-overlay', 'new-modal', 'rules-modal', 'setting-size', 'btn-create',
        'modal-error',
    ];
    for (const id of ids) elements[toCamelCase(id)] = document.getElementById(id);
    elements.pass = elements.btnPass;
    elements.undo = elements.btnUndo;
    elements.newGame = elements.btnNew;
    elements.rules = elements.btnRules;
    elements.rematch = elements.btnRematch;
    elements.create = elements.btnCreate;
    elements.overlay = elements.modalOverlay;
}

function toCamelCase(value) {
    return value.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

function chooseColor(color) {
    if (!currentState || currentState.phase !== 'action' || busy) return;
    selectedColor = color;
    elements.colorBlack.classList.toggle('active', color === 'black');
    elements.colorWhite.classList.toggle('active', color === 'white');
    renderBoard(currentState, selectedColor);
    setStatus(`本回合将投放${color === 'black' ? '黑色' : '白色'}细胞。`);
}

function renderPhase(state) {
    const layout = state.phase === 'layout';
    elements.phaseIndex.textContent = layout ? '01' : '02';
    elements.phaseName.textContent = layout ? '布局阶段' : '行动阶段';
    elements.phaseHint.textContent = layout
        ? '在己方半场或中线部署棋子'
        : '投放一个细胞，然后同步演化';
    elements.turnName.textContent = state.current_player === 'black' ? '黑方' : '白方';
    elements.turnCard.classList.toggle('black-turn', state.current_player === 'black');
    elements.turnCard.classList.toggle('white-turn', state.current_player === 'white');
    elements.colorPanel.classList.toggle('hidden', layout || state.status !== 'playing');
    elements.colorBlack.classList.toggle('active', selectedColor === 'black');
    elements.colorWhite.classList.toggle('active', selectedColor === 'white');
    elements.boardKicker.textContent = layout ? '布局区域' : '行动区域';
    elements.boardTitle.textContent = layout ? '建立你的初始生态' : '投放变量，观察生态战争';
    elements.pass.textContent = '停着';
    elements.pass.title = '双方连续停着后进入行动阶段';
    elements.pass.classList.toggle('hidden', !layout);
}

function renderPopulation(state) {
    elements.blackCount.textContent = state.black_count;
    elements.whiteCount.textContent = state.white_count;
    elements.generation.textContent = state.generation;
    elements.passCount.textContent = state.phase === 'layout'
        ? `${state.consecutive_passes} / 2`
        : '—';
    elements.boardSize.textContent = `${state.board.size} × ${state.board.size}`;
    const points = state.board.size * state.board.size;
    elements.blackTrack.style.width = populationWidth(state.black_count, points);
    elements.whiteTrack.style.width = populationWidth(state.white_count, points);
}

function populationWidth(count, total) {
    if (count === 0) return '0%';
    return `${Math.max(4, Math.min(100, count / total * 100))}%`;
}

function renderHistory(state) {
    elements.historyCount.textContent = state.history.length;
    elements.historyList.replaceChildren();
    if (state.history.length === 0) {
        const empty = document.createElement('div');
        empty.className = 'history-empty';
        empty.innerHTML = '<span>◇</span><p>第一步将在这里出现</p>';
        elements.historyList.append(empty);
        return;
    }

    for (const entry of state.history) {
        elements.historyList.append(createHistoryItem(entry));
    }
    elements.historyList.scrollTop = elements.historyList.scrollHeight;
}

function createHistoryItem(entry) {
    const item = document.createElement('div');
    item.className = 'history-item';

    const number = document.createElement('span');
    number.className = 'history-number';
    number.textContent = String(entry.number).padStart(2, '0');

    const main = document.createElement('div');
    main.className = 'history-main';
    const action = document.createElement('div');
    action.className = 'history-action';
    const stone = document.createElement('span');
    stone.className = `mini-stone ${entry.player}`;
    const actionText = document.createElement('span');
    actionText.textContent = describeHistoryAction(entry);
    action.append(stone, actionText);
    const meta = document.createElement('div');
    meta.className = 'history-meta';
    meta.textContent = entry.phase === 'layout'
        ? `${entry.player === 'black' ? '黑方' : '白方'}布局`
        : `演化至第 ${entry.generation} 代`;
    main.append(action, meta);

    const phase = document.createElement('span');
    phase.className = 'history-phase';
    phase.textContent = entry.phase === 'layout' ? '布局' : '行动';
    item.append(number, main, phase);
    return item;
}

function describeHistoryAction(entry) {
    if (entry.action.type === 'pass') return entry.phase === 'layout' ? '停着' : '跳过投放';
    const verb = entry.phase === 'layout' ? '部署' : '投放';
    const color = entry.action.color === 'black' ? '黑' : '白';
    return `${verb}${color} · ${entry.action.x + 1},${entry.action.y + 1}`;
}

function renderResult(state) {
    const finished = state.status !== 'playing';
    elements.resultBanner.classList.toggle('hidden', !finished);
    if (!finished) return;

    if (state.status === 'black_won') {
        elements.resultTitle.textContent = '黑方获胜';
        elements.resultDetail.textContent = `白方归零 · 黑方剩余 ${state.black_count} 个细胞`;
    } else if (state.status === 'white_won') {
        elements.resultTitle.textContent = '白方获胜';
        elements.resultDetail.textContent = `黑方归零 · 白方剩余 ${state.white_count} 个细胞`;
    } else {
        elements.resultTitle.textContent = '双方和棋';
        elements.resultDetail.textContent = '两个生态在同一代同时归零';
    }
}

function updateControls() {
    if (!elements.pass) return;
    const playing = currentState?.status === 'playing';
    elements.pass.disabled = busy || !playing;
    elements.undo.disabled = busy || !currentState?.can_undo;
    elements.newGame.disabled = busy;
    elements.rules.disabled = busy;
    elements.colorBlack.disabled = busy || !playing;
    elements.colorWhite.disabled = busy || !playing;
    elements.create.disabled = busy;
}

function openModal(modal) {
    if (modal === elements.newModal && currentState) {
        elements.settingSize.value = String(currentState.board.size);
        elements.modalError.textContent = '';
    }
    elements.newModal.classList.add('hidden');
    elements.rulesModal.classList.add('hidden');
    modal.classList.remove('hidden');
    elements.overlay.classList.remove('hidden');
}

function closeModals() {
    if (busy) return;
    elements.overlay.classList.add('hidden');
    elements.newModal.classList.add('hidden');
    elements.rulesModal.classList.add('hidden');
}
