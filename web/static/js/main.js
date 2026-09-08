import { getState, postAction, postNew, postUndo } from './api.js';
import { init, render, setBusy, setModalError, setStatus } from './ui.js';

let currentState = null;
let requestInFlight = false;

async function main() {
    init(handleCommand);
    setBusy(true);
    try {
        currentState = await getState();
        render(currentState);
        setStatus(initialStatus(currentState));
    } catch (error) {
        setStatus(`无法连接本地服务：${error.message}`, true);
    } finally {
        setBusy(false);
    }
}

async function handleCommand(command) {
    if (!currentState || requestInFlight) return false;
    requestInFlight = true;
    setBusy(true);
    try {
        if (command.type === 'action') return await submitAction(command.action);
        if (command.type === 'undo') return await undo();
        if (command.type === 'new_game') return await createNewGame(command.size);
        return false;
    } catch (error) {
        const message = `请求失败：${error.message}`;
        setStatus(message, true);
        if (command.type === 'new_game') setModalError(message);
        return false;
    } finally {
        requestInFlight = false;
        setBusy(false);
    }
}

async function submitAction(action) {
    const response = await postAction({ revision: currentState.revision, action });
    currentState = response.state;
    render(currentState, response.result);
    if (response.error) {
        setStatus(response.error, true);
        return false;
    }
    setStatus(actionStatus(action, response.result, currentState));
    return true;
}

async function undo() {
    const response = await postUndo({ revision: currentState.revision });
    currentState = response.state;
    render(currentState);
    if (response.error) {
        setStatus(response.error, true);
        return false;
    }
    setStatus(`已撤销一步，回到${currentState.phase === 'layout' ? '布局阶段' : `第 ${currentState.generation} 代`}。`);
    return true;
}

async function createNewGame(size) {
    const response = await postNew({ revision: currentState.revision, size });
    currentState = response.state;
    render(currentState);
    if (response.error) {
        setModalError(response.error);
        setStatus(response.error, true);
        return false;
    }
    setStatus('新对局已创建。黑方先布局，可在上半场或中线落子。');
    return true;
}

function actionStatus(action, result, state) {
    if (state.status !== 'playing') return outcomeStatus(state);
    if (result.phase_before === 'layout' && result.phase_after === 'action') {
        return '双方连续停着，布局结束。黑方开始第一回合行动。';
    }
    if (result.phase_before === 'layout') {
        if (action.type === 'pass') return '已停着；对手若也停着，将进入行动阶段。';
        return `${state.current_player === 'black' ? '白方' : '黑方'}已完成部署，轮到${state.current_player === 'black' ? '黑方' : '白方'}。`;
    }
    const stats = result.evolution;
    const changed = result.changes.length;
    if (!stats || changed === 0) return `第 ${state.generation} 代完成，棋盘进入稳定状态。`;
    return `第 ${state.generation} 代：黑 +${stats.born_black}/−${stats.died_black}，白 +${stats.born_white}/−${stats.died_white}。`;
}

function outcomeStatus(state) {
    if (state.status === 'black_won') return `对局结束：白方归零，黑方获胜。`;
    if (state.status === 'white_won') return `对局结束：黑方归零，白方获胜。`;
    return '对局结束：双方同时归零，和棋。';
}

function initialStatus(state) {
    if (state.status !== 'playing') return outcomeStatus(state);
    if (state.phase === 'action') return `行动阶段第 ${state.generation + 1} 回合，${state.current_player === 'black' ? '黑方' : '白方'}行动。`;
    return `${state.current_player === 'black' ? '黑方' : '白方'}布局中。双方连续停着后开始演化。`;
}

void main();
