import init, { Compile, CompilerIntermediateProducts } from './circuitgame_lib.js';
import { isIntermediateProducts } from './typeGuards.js';
import { File, Gate, IntermediateProducts, Module } from './types.js';

function CompileAsTypescriptResult(code: string): IntermediateProducts {
    let result_from_rust: any = JSON.parse(CompilerIntermediateProducts(code));
    // 型チェックと変換を行う
    if (!isIntermediateProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust;
}

async function run() {
    await init();
    const input = `
// Example usage with comments
using nor:2->1;

// This is a NOT gate module
module not (x)->(a) {
    a: nor <- x x;
}

module or (x y)->(b) {
    a: nor <- x y;
    b: not <- a  ;
}

module and (x y)->(c) {
    a: not <- x  ;
    b: not <- y  ;
    c: nor <- a b;
}

module xor (x y)->(e) {
    a: not <- x  ;
    b: not <- y  ;
    c: nor <- a b;
    d: nor <- x y;
    e: nor <- c d;
}

module hAddr (x y)->(c s) {
    c: and <- x y;
    s: xor <- x y;
}

module fAdr (x y z)->(c s2) {
    c1 s1: hAddr <- x y  ;
    c2 s2: hAddr <- s1 z ;
    c    : or    <- c1 c2;
}

test not:1->1 {
    t -> f;
    f -> t;
}

test or:2->1 {
    t t -> t;
    t f -> t;
    f t -> t;
    f f -> f;
}

test and:2->1 {
    t t -> t;
    t f -> f;
    f t -> f;
    f f -> f;
}
    `;
    const input_without_space = `USEnor:2>1;DEFnot(x)>(a){a:nor<x,x;}DEFor(x,y)>(b){a:nor<x,y;b:not<a;}DEFand(x,y)>(c){a:not<x;b:not<y;c:nor<a,b;}DEFxor(x,y)>(e){a:not<x;b:not<y;c:nor<a,b;d:nor<x,y;e:nor<c,d;}DEFhAddr(x,y)>(c,s){c:and<x,y;s:xor<x,y;}DEFfAdr(x,y,z)>(c,s2){c1,s1:hAddr<x,y;c2,s2:hAddr<s1,z;c:or<c1,c2;}TESTnot:1>1{t>f;f>t;}TESTor:2>1{t,t>t;t,f>t;f,t>t;f,f>f;}TESTand:2>1{t,t>t;t,f>f;f,t>f;f,f>f;}`;
    {
        console.log("< Input >")
        const result = CompileAsTypescriptResult(input);
        console.log(result);
        console.log(result.module_dependency_sorted[0]);
        console.log(Compile(input, result.module_dependency_sorted[0]));
        visualizeCircuit(result.ast);
    }
}

run();
function createCircuitVisualizer(modules: Module[], container: HTMLElement) {
    const GATE_WIDTH = 60;
    const GATE_HEIGHT = 40;
    const START_X = 100;
    const START_Y = 120;
    const GATE_SPACING = 140;
    const VERTICAL_SPACING = 80;

    // 最大の幅と高さを計算
    const maxGates = Math.max(...modules.map(m => m.gates.length));
    const maxInputs = Math.max(...modules.map(m => m.inputs.length));
    const width = START_X * 2 + GATE_SPACING * (maxGates + 1);
    const height = START_Y * 2 + VERTICAL_SPACING * (maxInputs - 1);

    // SVG要素の作成
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", width.toString());
    svg.setAttribute("height", height.toString());
    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);

    // 接続線のパスを生成する関数
    function getConnectionPath(from: { x: number; y: number }, to: { x: number; y: number }): string {
        const midX = (from.x + to.x) / 2;
        return `M ${from.x} ${from.y} C ${midX} ${from.y}, ${midX} ${to.y}, ${to.x} ${to.y}`;
    }

    // ドラッグ状態を管理
    let isDragging = false;
    let selectedElement: SVGElement | null = null;
    let offset = { x: 0, y: 0 };
    let connections: SVGPathElement[] = [];

    // マウスイベントのハンドラを修正
    function startDrag(evt: MouseEvent, element: SVGElement) {
        isDragging = true;
        selectedElement = element;

        // SVGの座標系でのオフセットを計算
        const svgPoint = svg.createSVGPoint();
        const CTM = svg.getScreenCTM();
        if (!CTM) return;

        svgPoint.x = evt.clientX;
        svgPoint.y = evt.clientY;
        const transformedPoint = svgPoint.matrixTransform(CTM.inverse());

        // 現在の要素の位置を取得
        const transform = element.closest('g')?.getAttribute('transform');
        const currentTranslate = transform ? transform.match(/translate\(([^,]+),([^)]+)\)/) : null;
        const currentX = currentTranslate ? parseFloat(currentTranslate[1]) : 0;
        const currentY = currentTranslate ? parseFloat(currentTranslate[2]) : 0;

        offset = {
            x: transformedPoint.x - currentX,
            y: transformedPoint.y - currentY
        };
    }

    function drag(evt: MouseEvent) {
        if (!isDragging || !selectedElement) return;

        evt.preventDefault();
        const svgPoint = svg.createSVGPoint();
        const CTM = svg.getScreenCTM();
        if (!CTM) return;

        svgPoint.x = evt.clientX;
        svgPoint.y = evt.clientY;
        const transformedPoint = svgPoint.matrixTransform(CTM.inverse());

        moveElement(selectedElement,
            transformedPoint.x - offset.x,
            transformedPoint.y - offset.y
        );
        updateConnections();
    }

    function endDrag() {
        isDragging = false;
        selectedElement = null;
    }

    // マウス座標をSVG座標系に変換
    function getMousePosition(evt: MouseEvent) {
        const CTM = svg.getScreenCTM();
        if (!CTM) return { x: 0, y: 0 };
        return {
            x: (evt.clientX - CTM.e) / CTM.a,
            y: (evt.clientY - CTM.f) / CTM.d
        };
    }

    // 要素の移動関数を修正
    function moveElement(element: SVGElement, x: number, y: number) {
        const g = element.closest('g');
        if (!g) return;

        // ゲート全体を移動（補正値を削除）
        g.setAttribute('transform', `translate(${x},${y})`);
    }

    // 配線の接続点を計算する関数を修正
    function calculateConnectionPoints(
        fromElement: Element,
        toElement: Element,
        isInput: boolean
    ): { from: { x: number; y: number }, to: { x: number; y: number } } {
        const svgPoint = svg.createSVGPoint();
        const fromBox = fromElement.getBoundingClientRect();
        const toBox = toElement.getBoundingClientRect();
        const CTM = svg.getScreenCTM();

        if (!CTM) return { from: { x: 0, y: 0 }, to: { x: 0, y: 0 } };

        // ブラウザ座標をSVG座標に変換
        function clientToSVGPoint(x: number, y: number) {
            svgPoint.x = x;
            svgPoint.y = y;
            return svgPoint.matrixTransform(CTM.inverse());
        }

        // 接続点の座標を計算
        const fromPoint = clientToSVGPoint(
            isInput ? fromBox.x : fromBox.x + fromBox.width,
            fromBox.y + fromBox.height / 2
        );
        const toPoint = clientToSVGPoint(
            isInput ? toBox.x : toBox.x + toBox.width,
            toBox.y + toBox.height / 2
        );

        return {
            from: { x: fromPoint.x, y: fromPoint.y },
            to: { x: toPoint.x, y: toPoint.y }
        };
    }

    // 配線の更新関数を修正
    function updateConnections() {
        connections.forEach(path => {
            const fromElement = document.getElementById(path.dataset.from || '');
            const toElement = document.getElementById(path.dataset.to || '');
            if (!fromElement || !toElement) return;

            const isInput = path.dataset.from?.startsWith('input-') || false;
            const points = calculateConnectionPoints(fromElement, toElement, isInput);
            path.setAttribute('d', getConnectionPath(points.from, points.to));
        });
    }

    // ゲートの描画関数を修正
    function renderGate(pos: { x: number; y: number }, name: string, id: string, gate: Gate): SVGGElement {
        const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
        g.setAttribute('id', `gate-${id}`);
        g.setAttribute('transform', `translate(${pos.x},${pos.y})`);

        // ゲートの本体
        const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        rect.setAttribute("x", (-GATE_WIDTH / 2).toString());
        rect.setAttribute("y", (-GATE_HEIGHT / 2).toString());
        rect.setAttribute("width", GATE_WIDTH.toString());
        rect.setAttribute("height", GATE_HEIGHT.toString());
        rect.setAttribute("fill", "#e1f5fe");
        rect.setAttribute("stroke", "#333");
        rect.setAttribute("stroke-width", "1");
        rect.setAttribute("rx", "4");
        rect.setAttribute("cursor", "move");

        // 入力ポートの描画（左側）
        gate.inputs.forEach((_, index) => {
            const portSpacing = GATE_HEIGHT / (gate.inputs.length + 1);
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("id", `gate-${id}-input-${index}`);
            circle.setAttribute("cx", (-GATE_WIDTH / 2).toString());
            circle.setAttribute("cy", (-GATE_HEIGHT / 2 + portSpacing * (index + 1)).toString());
            circle.setAttribute("r", "3");
            circle.setAttribute("fill", "#4ade80");
            g.appendChild(circle);
        });

        // 出力ポートの描画（右側）
        gate.outputs.forEach((_, index) => {
            const portSpacing = GATE_HEIGHT / (gate.outputs.length + 1);
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("id", `gate-${id}-output-${index}`);
            circle.setAttribute("cx", (GATE_WIDTH / 2).toString());
            circle.setAttribute("cy", (-GATE_HEIGHT / 2 + portSpacing * (index + 1)).toString());
            circle.setAttribute("r", "3");
            circle.setAttribute("fill", "#f43f5e");
            g.appendChild(circle);
        });

        // ドラッグイベントの追加
        rect.addEventListener('mousedown', (evt) => startDrag(evt, g));

        const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
        text.setAttribute("text-anchor", "middle");
        text.setAttribute("dominant-baseline", "middle");
        text.textContent = name;

        g.appendChild(rect);
        g.appendChild(text);
        return g;
    }

    // SVGのイベントリスナーを追加
    svg.addEventListener('mousemove', drag);
    svg.addEventListener('mouseup', endDrag);
    svg.addEventListener('mouseleave', endDrag);

    // 配線作成関数を修正
    function createConnection(from: string, to: string, path: SVGPathElement) {
        path.dataset.from = from;
        path.dataset.to = to;
        path.setAttribute("stroke", "#666");
        path.setAttribute("stroke-width", "2");
        path.setAttribute("fill", "none");
        connections.push(path);

        const fromElement = document.getElementById(from);
        const toElement = document.getElementById(to);
        if (fromElement && toElement) {
            const isInput = from.startsWith('input-');
            const points = calculateConnectionPoints(fromElement, toElement, isInput);
            path.setAttribute('d', getConnectionPath(points.from, points.to));
        }

        svg.appendChild(path);
    }

    // モジュールの描画関数
    function renderModule(module: Module) {
        const g = document.createElementNS("http://www.w3.org/2000/svg", "g");

        // 入力ポートの配置を計算
        const inputPositions = module.inputs.map((_, i) => ({
            x: START_X,
            y: START_Y + i * VERTICAL_SPACING
        }));

        // ゲートの配置を計算（レベルごとに整理）
        const gatePositions = module.gates.map((_, i) => ({
            x: START_X + GATE_SPACING * (i + 1),
            y: START_Y + (VERTICAL_SPACING * i) / 2
        }));

        // 出力ポートの配置を計算
        const outputPositions = module.outputs.map((_, i) => ({
            x: START_X + GATE_SPACING * (module.gates.length + 1),
            y: START_Y + i * VERTICAL_SPACING
        }));

        // 入力ポートの描画
        inputPositions.forEach((pos, i) => {
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("id", `input-${module.inputs[i]}`);
            circle.setAttribute("cx", pos.x.toString());
            circle.setAttribute("cy", pos.y.toString());
            circle.setAttribute("r", "4");
            circle.setAttribute("fill", "#4ade80");

            const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
            text.setAttribute("x", (pos.x - 20).toString());
            text.setAttribute("y", pos.y.toString());
            text.setAttribute("dominant-baseline", "middle");
            text.textContent = module.inputs[i];

            g.appendChild(circle);
            g.appendChild(text);
        });

        // ゲートの描画
        module.gates.forEach((gate, i) => {
            const gateElement = renderGate(gatePositions[i], gate.module_name, `${gate.module_name}-${i}`, gate);
            g.appendChild(gateElement);

            // 配線の描画
            const connections = document.createElementNS("http://www.w3.org/2000/svg", "g");
            connections.setAttribute("stroke", "#666");
            connections.setAttribute("stroke-width", "2");
            connections.setAttribute("fill", "none");

            // 入力への配線
            gate.inputs.forEach((input, inputIndex) => {
                const inputPos = inputPositions[module.inputs.indexOf(input)];
                const sourceGate = module.gates.find(g => g.outputs.includes(input));
                if (inputPos) {
                    // モジュール入力からの配線
                    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
                    createConnection(
                        `input-${input}`,
                        `gate-${gate.module_name}-${i}-input-${inputIndex}`,
                        path
                    );
                } else if (sourceGate) {
                    // 他のゲートからの配線
                    const sourceIndex = module.gates.indexOf(sourceGate);
                    const outputIndex = sourceGate.outputs.indexOf(input);
                    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
                    createConnection(
                        `gate-${sourceGate.module_name}-${sourceIndex}-output-${outputIndex}`,
                        `gate-${gate.module_name}-${i}-input-${inputIndex}`,
                        path
                    );
                }
            });

            // 出力ポートへの配線
            gate.outputs.forEach((output, outputIndex) => {
                const outputPos = outputPositions[module.outputs.indexOf(output)];
                if (outputPos) {
                    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
                    createConnection(
                        `gate-${gate.module_name}-${i}-output-${outputIndex}`,
                        `output-${output}`,
                        path
                    );
                }
            });

            g.appendChild(connections);
        });

        // 出力ポートの描画
        outputPositions.forEach((pos, i) => {
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("id", `output-${module.outputs[i]}`);
            circle.setAttribute("cx", pos.x.toString());
            circle.setAttribute("cy", pos.y.toString());
            circle.setAttribute("r", "4");
            circle.setAttribute("fill", "#f43f5e");

            const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
            text.setAttribute("x", (pos.x + 20).toString());
            text.setAttribute("y", pos.y.toString());
            text.setAttribute("dominant-baseline", "middle");
            text.textContent = module.outputs[i];

            g.appendChild(circle);
            g.appendChild(text);
        });

        return g;
    }

    // モジュールの描画
    modules.forEach(module => {
        const moduleElement = renderModule(module);
        svg.appendChild(moduleElement);
    });

    // コンテナに追加
    container.appendChild(svg);
    updateConnections();
}

// 使用例
function visualizeCircuit(ast: File) {
    const modules = ast.components
        .filter(component => component.type === "Module")
        // Moduleタイプのコンポーネントだけを抽出
        .map(component => component as Module);

    const container = document.createElement('div');
    document.body.appendChild(container);
    createCircuitVisualizer(modules.slice(5, 6), container);
}
