let canvas;
let context;
let placeHandler;
let state = null;
let selectedColor = 'black';
let busy = false;
let hoverPoint = null;
let geometry = null;
let flashes = [];
let frameRequest = 0;

export function initBoard(boardCanvas, onPlace) {
    canvas = boardCanvas;
    context = canvas.getContext('2d');
    placeHandler = onPlace;
    canvas.addEventListener('pointermove', handlePointerMove);
    canvas.addEventListener('pointerleave', () => {
        hoverPoint = null;
        draw();
    });
    canvas.addEventListener('click', handleClick);
    new ResizeObserver(() => draw()).observe(canvas);
}

export function renderBoard(nextState, color, changes = []) {
    state = nextState;
    selectedColor = color;
    const now = performance.now();
    flashes = changes.map(change => ({ ...change, started: now, expires: now + 720 }));
    draw(now);
}

export function setBoardBusy(value) {
    busy = value;
    draw();
}

function handlePointerMove(event) {
    const nextPoint = eventPoint(event);
    if (samePoint(nextPoint, hoverPoint)) return;
    hoverPoint = nextPoint;
    canvas.style.cursor = nextPoint && isLegal(nextPoint) ? 'pointer' : 'not-allowed';
    if (busy) canvas.style.cursor = 'wait';
    draw();
}

function handleClick(event) {
    const point = eventPoint(event);
    if (!point || !isLegal(point)) return;
    const color = state.phase === 'layout' ? state.current_player : selectedColor;
    placeHandler({ x: point.x, y: point.y, color });
}

function eventPoint(event) {
    if (!state || !geometry) return null;
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left - geometry.originX;
    const y = event.clientY - rect.top - geometry.originY;
    if (x < 0 || y < 0 || x >= geometry.side || y >= geometry.side) return null;
    return {
        x: Math.floor(x / geometry.cellSize),
        y: Math.floor(y / geometry.cellSize),
    };
}

function isLegal(point) {
    if (!state || busy || state.status !== 'playing') return false;
    const index = point.y * state.board.size + point.x;
    if (state.board.cells[index] !== 'empty') return false;
    if (state.phase === 'action') return true;
    const middle = Math.floor(state.board.size / 2);
    if (state.current_player === 'black') return point.y <= middle;
    return point.y >= middle;
}

function samePoint(left, right) {
    if (!left || !right) return left === right;
    return left.x === right.x && left.y === right.y;
}

function draw(now = performance.now()) {
    if (!canvas || !state) return;
    cancelAnimationFrame(frameRequest);
    resizeCanvas();
    context.clearRect(0, 0, geometry.width, geometry.height);
    drawBoardBackground();
    drawCells(now);
    drawTerritoryLabels();
    flashes = flashes.filter(flash => flash.expires > now);
    if (flashes.length > 0) {
        frameRequest = requestAnimationFrame(draw);
    }
}

function resizeCanvas() {
    const rect = canvas.getBoundingClientRect();
    const ratio = Math.min(window.devicePixelRatio || 1, 2);
    const pixelWidth = Math.max(1, Math.round(rect.width * ratio));
    const pixelHeight = Math.max(1, Math.round(rect.height * ratio));
    if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
        canvas.width = pixelWidth;
        canvas.height = pixelHeight;
    }
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    const padding = Math.max(8, Math.min(15, rect.width * .022));
    const side = Math.min(rect.width, rect.height) - padding * 2;
    geometry = {
        width: rect.width,
        height: rect.height,
        side,
        cellSize: side / state.board.size,
        originX: (rect.width - side) / 2,
        originY: (rect.height - side) / 2,
    };
}

function drawBoardBackground() {
    context.fillStyle = '#d6a45f';
    context.fillRect(0, 0, geometry.width, geometry.height);
}

function drawCells(now) {
    const size = state.board.size;
    const middle = Math.floor(size / 2);
    const lastAction = state.history.at(-1)?.action;

    for (let y = 0; y < size; y += 1) {
        for (let x = 0; x < size; x += 1) {
            const left = geometry.originX + x * geometry.cellSize;
            const top = geometry.originY + y * geometry.cellSize;
            const side = geometry.cellSize;
            drawCellBase(left, top, side, y, middle);

            const cell = state.board.cells[y * size + x];
            if (cell !== 'empty') drawStone(left, top, side, cell);

            if (lastAction?.type === 'place' && lastAction.x === x && lastAction.y === y) {
                drawLastMove(left, top, side);
            }
            if (hoverPoint?.x === x && hoverPoint?.y === y && isLegal(hoverPoint)) {
                const color = state.phase === 'layout' ? state.current_player : selectedColor;
                drawPreview(left, top, side, color);
            }
            const flash = flashes.find(item => item.x === x && item.y === y);
            if (flash) drawFlash(left, top, side, flash, now);
        }
    }
}

function drawCellBase(left, top, side, row, middle) {
    context.strokeStyle = row === middle && state.phase === 'layout'
        ? 'rgba(72,42,16,.82)'
        : 'rgba(72,42,16,.46)';
    context.lineWidth = row === middle && state.phase === 'layout' ? 1.25 : .8;
    context.strokeRect(left, top, side, side);
}

function drawStone(left, top, side, color) {
    const centerX = left + side / 2;
    const centerY = top + side / 2;
    const radius = side * .38;
    circlePath(centerX, centerY, radius);
    const gradient = context.createRadialGradient(
        centerX - radius * .34, centerY - radius * .38, radius * .08,
        centerX, centerY, radius,
    );
    if (color === 'black') {
        gradient.addColorStop(0, '#63676c');
        gradient.addColorStop(.24, '#24272b');
        gradient.addColorStop(.72, '#090a0c');
        gradient.addColorStop(1, '#020203');
        context.shadowColor = 'rgba(55,31,12,.72)';
    } else {
        gradient.addColorStop(0, '#ffffff');
        gradient.addColorStop(.38, '#f4f2e9');
        gradient.addColorStop(.78, '#d4d1c6');
        gradient.addColorStop(1, '#aaa99f');
        context.shadowColor = 'rgba(70,40,15,.5)';
    }
    context.shadowBlur = Math.min(8, side * .18);
    context.shadowOffsetY = Math.max(1, side * .055);
    context.fillStyle = gradient;
    context.fill();
    context.shadowBlur = 0;
    context.shadowOffsetY = 0;
    context.strokeStyle = color === 'black' ? 'rgba(0,0,0,.72)' : 'rgba(255,255,255,.72)';
    context.lineWidth = Math.max(.8, side * .025);
    context.stroke();
}

function drawPreview(left, top, side, color) {
    const centerX = left + side / 2;
    const centerY = top + side / 2;
    circlePath(centerX, centerY, side * .34);
    context.fillStyle = color === 'black' ? 'rgba(5,8,12,.56)' : 'rgba(244,244,238,.48)';
    context.fill();
    context.setLineDash([3, 3]);
    context.strokeStyle = 'rgba(67,44,18,.78)';
    context.lineWidth = 1.2;
    context.stroke();
    context.setLineDash([]);
}

function drawLastMove(left, top, side) {
    circlePath(left + side / 2, top + side / 2, side * .44);
    context.strokeStyle = 'rgba(205,245,86,.92)';
    context.lineWidth = Math.max(1.2, side * .04);
    context.stroke();
}

function drawFlash(left, top, side, flash, now) {
    const progress = Math.max(0, Math.min(1, (now - flash.started) / (flash.expires - flash.started)));
    const alpha = (1 - progress) * (.75 + Math.sin(progress * Math.PI * 3) * .18);
    const radius = side * (.46 - progress * .12);
    circlePath(left + side / 2, top + side / 2, radius);
    context.strokeStyle = flash.after === 'empty'
        ? `rgba(255,110,121,${alpha})`
        : `rgba(201,240,100,${alpha})`;
    context.lineWidth = Math.max(1.5, side * .06 * (1 - progress * .4));
    context.stroke();
}

function drawTerritoryLabels() {
    if (geometry.cellSize < 22) return;
    context.save();
    context.font = `600 ${Math.max(9, geometry.cellSize * .21)}px "Noto Sans SC", sans-serif`;
    context.textBaseline = 'middle';
    context.letterSpacing = '1px';
    if (state.phase === 'layout') {
        context.fillStyle = 'rgba(59,34,13,.7)';
        context.fillText('黑方区域', geometry.originX + 7, geometry.originY + geometry.cellSize / 2);
        context.fillStyle = 'rgba(59,34,13,.7)';
        const text = '白方区域';
        const width = context.measureText(text).width;
        context.fillText(text, geometry.originX + geometry.side - width - 7,
            geometry.originY + geometry.side - geometry.cellSize / 2);
    }
    context.restore();
}

function circlePath(x, y, radius) {
    context.beginPath();
    context.arc(x, y, radius, 0, Math.PI * 2);
}
