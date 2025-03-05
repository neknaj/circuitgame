import { Compile, VM } from './circuitgame.js';
import { elm as E, textelm as T } from './cdom.js';
import { IntermediateProducts, CompiledModule, CompiledGateInput, Module, Graphical } from './types.js';
import { constantOtherSymbol } from 'ace-builds/src-noconflict/mode-ruby_highlight_rules';

let vm_id = null;
let selected = null;
let compiledModule: CompiledModule;
let input: (0|1)[] = [];
let waveData: (0|1)[][] = []; // 0 or 1
let waveLabels = [];

function init(elm: HTMLDivElement,product: IntermediateProducts,module_name: string = null) {
    if (module_name!=null&&product.module_dependency_sorted.includes(module_name)==false) { module_name = null; }
    if (module_name == null) {
        if (selected == null) { selected = product.module_dependency_sorted[0]; }
        if (selected == null) { return; }
        module_name = selected;
    }
    if (module_name!=null&&product.module_dependency_sorted.includes(module_name)==false) { return; }
    selected = module_name;
    const binary = Compile(product.source,module_name);
    if (binary.length==0) { console.warn("compiling error") ;return; }
    vm_id = VM.init(binary);

    const modulesAST = module_name!="nor"?(product.ast.components.filter(x=>x.type=="Module"&&x.name==module_name)[0] as Module):{name:"nor",inputs:["x","y"],outputs:["a"],_sequential:[{inputs:["x","y"],outputs:["a"],module_name:"nor"}]};
    elm.innerHTML = "";
    console.log("module",module_name,selected);
    // module選択のドロップダウンメニューを追加
    elm.Add(E("select",{},product.module_type_list.map(x=>x.name).map(
        (m,i) => {
            let option = E("option",{value:m},[T(m)]);
            if (m==selected) { option.setAttribute("selected","true"); }
            return option;
        }
    )).Listen("change",e=>{
        if ("value" in e.target) {
            let new_module_name = e.target.value as string;
            if (new_module_name!=module_name) {
                selected = new_module_name;
                init(elm,product,new_module_name);
            }
        }
    }));
    //
    product.module_type_list
    const moduleType = product.module_type_list.filter(m=>m.name==module_name)[0];
    console.log(moduleType);
    if (input.length==moduleType.mtype.input_count) {
        input = new Array(moduleType.mtype.input_count).fill(0);
    }
    elm.Add(E("h2",{},[T(module_name)]));
    elm.Add(E("h3",{},[T("input")]));
    elm.Add(E("div",{class:"input"},Array(moduleType.mtype.input_count).fill(0).map(
        (_,i) => E("span",{},[
            (() => {
                let inputElm = E("input",{type:"checkbox",id:"input"+i},[])
                if (input[i]==1) {
                    inputElm.setAttribute("checked","true");
                }
                return inputElm;
            })(),
            E("label",{for:"input"+i},[T(modulesAST.inputs[i])])
        ]),
    )));
    console.log(modulesAST)
    elm.Add(E("h3",{},[T("output")]));
    elm.Add(E("div",{class:"output"},Array(moduleType.mtype.output_count).fill(0).map(
        (_,i) => E("span",{},[
            E("input",{type:"checkbox",id:"output"+i,disabled:true},[]),
            E("label",{for:"output"+i},[T(modulesAST.outputs[i])])
        ]),
    )));
    elm.Add(E("h3",{},[T("tick")]));
    elm.Add(E("span",{id:"tick"},[]));
    elm.Add(E("h3",{},[T("number of gate")]));
    elm.Add(E("span",{},[T(product.expanded_modules[module_name].gates_sequential.length)]));
    {
        const myEvent = new CustomEvent("moduleChanged", {
            detail: { module_name, product: product },
            bubbles: true, // イベントが親要素に伝播する
            cancelable: true, // イベントをキャンセル可能にする
        });
        elm.dispatchEvent(myEvent);
    }
    {
        compiledModule = product.expanded_modules[module_name];
        waveData = new Array(moduleType.mtype.input_count+moduleType.mtype.output_count).fill(0).map(x=>[]);
        waveLabels = [...modulesAST.inputs,...modulesAST.outputs];
    }
    CreateGraphdcalIO(product,module_name);
}

// 1tick進める
export function tick() {
    input = Array.from(document.querySelectorAll(".input input"))
                .map(e=>(e as HTMLInputElement).checked==true?1:0);
    input.forEach((v,i)=>VM.set(vm_id,i,v==1));
    VM.next(vm_id,1);
    (Array.from(document.querySelectorAll(".output input")) as HTMLInputElement[])
        .forEach((e,i)=>e.checked = VM.getOutput(vm_id)[i]==1?true:false);
    (document.querySelector("#tick") as HTMLParagraphElement).innerText = `${VM.getTick(vm_id)}`;
    // ロジアナグラフ
    {
        input.forEach((v,i)=>{waveData[i].push(v)})
        Array.from(VM.getOutput(vm_id)).map(x=>x==1?1:0).forEach((v,i)=>{waveData[i+input.length].push(v)})
    }
    if ((document.querySelector("#graph2_switch") as HTMLInputElement).checked) {
        updateLogiAnaGraph();
    }
    // グラフの色を反映
    if ((document.querySelector("#graph1_switch") as HTMLInputElement).checked) {
        changeGraphColors();
    }
    // graphical io を更新
    graphicalIO_update();
}

// 現在の入出力をコピー
export function copyIO() {
    const vm = document.querySelector("#vm");
    if (!vm) { return; }
    const inputs = Array.from(vm.querySelectorAll(".input input")).map(x=> (x as HTMLInputElement).checked);
    const outputs = Array.from(vm.querySelectorAll(".output input")).map(x=>(x as HTMLInputElement).checked);
    const text = `${inputs.map(x=>x?"t":"f").join(" ")} -> ${outputs.map(x=>x?"t":"f").join(" ")};`;
    navigator.clipboard.writeText(text).catch(err => {console.error('コピーに失敗しました: ' + err);});
}

export function updateLogiAnaGraph() {
    const elm = document.querySelector("#graph2");
    const bound = elm.getBoundingClientRect();
    const channelTypes = [...new Array(input.length).fill("input"),...new Array(VM.getOutput(vm_id).length).fill("output")];
    const graph = createLogicAnalyzerGraph(waveData, waveLabels, channelTypes, bound.width, bound.height, Number((document.querySelector("#digiAnaLastN") as HTMLInputElement).value));
    elm.innerHTML = "";
    elm.Add(graph);
}

function autoUpdate() {
    if (vm_id==null) { setTimeout(autoUpdate,100);return; }
    if (!(document.querySelector("#vmRun") as HTMLInputElement).checked) { setTimeout(autoUpdate,100);return; }
    tick();
    setTimeout(()=>{
        requestAnimationFrame(autoUpdate);
    },Number((document.querySelector("#vmSpeed") as HTMLInputElement).value));
}
autoUpdate();

export default init;


function changeGraphColors() {
    // グラフの色を反映
    {
        // console.log(document.querySelectorAll("#graph .node.input"))
        document.querySelectorAll("#graph1 .node.input").forEach((node,i)=>{
            // console.log("input",node,input[i],i,input)
            if (input[i]) {
                node.classList.add("active");
            }
            else {
                node.classList.remove("active");
            }
        });
    }
    {
        const gate = Array.from(VM.getGates(vm_id)).map(v=>v==1);
        document.querySelectorAll("#graph1 .node.gate").forEach((node,i)=>{
            if (gate[i]) {
                node.classList.add("active");
            }
            else {
                node.classList.remove("active");
            }
        });
    }
    {
        const gate = Array.from(VM.getOutput(vm_id)).map(v=>v==1);
        document.querySelectorAll("#graph1 .node.output").forEach((node,i)=>{
            if (gate[i]) {
                node.classList.add("active");
            }
            else {
                node.classList.remove("active");
            }
        });
    }
    // ワイヤーの色
    {
        const gate = Array.from(VM.getGates(vm_id)).map(v=>v==1);
        const wires = compiledModule.gates_sequential.flat(1).concat(compiledModule.outputs.map(x=>({NorGate:x} as CompiledGateInput)));
        // console.log(wires)
        // console.log(document.querySelectorAll("#graph .edgePaths path"))
        document.querySelectorAll("#graph1 .edgePaths path").forEach((node,i)=>{
            try {
                let active = false;
                console.log(wires)
                if ("NorGate" in wires[i]) {
                    active = gate[wires[i].NorGate];
                }
                else {
                    active = input[wires[i].Input]==1;
                }
                if (active) {
                    node.classList.add("active");
                }
                else {
                    node.classList.remove("active");
                }
            }
            catch (e) {
                console.error(e);
            }
        });
    }
}


function createLogicAnalyzerGraph(data: number[][], labels: string[], channelTypes: ("input"|"output")[], width: number, height: number, lastN: number) {
    lastN = Math.min(lastN,data[0].length);

    const svgNS = "http://www.w3.org/2000/svg";

    // Create SVG element
    const svg = document.createElementNS(svgNS, "svg");
    svg.setAttribute("width", width.toString());
    svg.setAttribute("height", height.toString());
    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
    svg.setAttribute("style", "border: 1px solid black;");

    // Parameters
    const numChannels = data.length;
    const channelHeight = (height - 20) / numChannels;
    const stepWidth = (width - 120) / lastN; // Leave space for labels on the left

    // Trim data to show only the last N steps
    const trimmedData = data.map(channel => channel.slice(-lastN));

    // Add channel labels
    labels.forEach((label, index) => {
        const text = document.createElementNS(svgNS, "text");
        text.setAttribute("x", "15");
        text.setAttribute("y", (30 + channelHeight * (index + 0.3)).toString());
        text.setAttribute("dominant-baseline", "middle");
        text.setAttribute("fill", "white");
        text.setAttribute("font-size", "22");
        text.textContent = label;
        svg.appendChild(text);
    });

    // Create waveforms for each channel
    function drawPath(color) {
        return (channelData, channelIndex) => {
            const path = document.createElementNS(svgNS, "path");
            let d = "";
            let currentY = 30 + channelHeight * (channelIndex + 0.3);

            channelData.forEach((value, index) => {
                const x = 100 + index * stepWidth;
                const y = currentY + (value ? -channelHeight / 8 : channelHeight / 8);
                if (index === 0) {
                    d += `M ${x} ${y} `; // Move to the first point
                } else {
                    d += `L ${x} ${y} `; // Line to next point
                }
                if (index < channelData.length - 1) {
                    // 立上がりと立下りが斜めになるようにする
                    const nextY = currentY + (channelData[index + 1] ? -channelHeight / 8 : channelHeight / 8);
                    d += `H ${x + stepWidth*0.2} `;
                    d += `L ${x + stepWidth} ${nextY} `;
                }
            });

            path.setAttribute("d", d.trim());
            path.setAttribute("fill", "none");
            path.setAttribute("stroke", color(channelIndex));
            path.setAttribute("stroke-width", "2");
            svg.appendChild(path);
        }
    }
    trimmedData.map(a=>a.map(x=>1-x)).forEach(drawPath(i=>channelTypes[i]=="input"?"#045":"#540"));
    trimmedData.forEach(drawPath(i=>channelTypes[i]=="input"?"#09f":"#f90"));

    // Add vertical lines and time labels
    const numSteps = trimmedData[0].length;
    const startIndex = data[0].length - lastN; // Adjust starting index for time labels
    for (let i = 0; i < numSteps; i++) {
        if ((startIndex + i) % Math.floor(lastN/20) === 0) {
            const x = 100 + i * stepWidth;

            // Vertical line
            const line = document.createElementNS(svgNS, "line");
            line.setAttribute("x1", x.toString());
            line.setAttribute("y1", "20");
            line.setAttribute("x2", x.toString());
            line.setAttribute("y2", height.toString());
            line.setAttribute("stroke", "gray");
            line.setAttribute("stroke-width", "1");
            line.setAttribute("stroke-dasharray", "4");
            svg.appendChild(line);

            // Time label
            const text = document.createElementNS(svgNS, "text");
            text.setAttribute("x", x.toString());
            text.setAttribute("y", "14");
            text.setAttribute("font-size", "13");
            text.setAttribute("text-anchor", "middle");
            text.setAttribute("fill", "gray");
            text.textContent = (startIndex + i).toString();
            svg.appendChild(text);
        }
    }

    return svg;
};


var graphicalIO_update: ()=>any = ()=>{};
function CreateGraphdcalIO(product: IntermediateProducts, module_name: string) {
    const graphical_ = product.ast.components.filter(component=>component.type=="Graphical"&&component.name==module_name);
    if (graphical_.length<1) {
        console.log("no graphical");
        return
    }
    const graphical = graphical_[0] as Graphical;
    console.log(graphical);
    let width = 0;
    let height = 0;
    if (graphical.size.type=="Size") {
        width = graphical.size.width;
        height = graphical.size.height;
    }
    type color = [number,number,number];
    let input_map: { [key:number]: [number, color,color]} = {}; // pixel -> index of input, on color, off color
    let output_map: { [key:number]: [number, color,color]} = {}; // pixel -> index of output, on color, off color
    let pixel_encoder = (x,y) => x+y*width;
    for (let pixel of graphical.pixels) {
        if (pixel.io_index.io_type=="input") {
            input_map[pixel_encoder(...pixel.coord)] = [pixel.io_index.index,pixel.color.on,pixel.color.off];
        }
        if (pixel.io_index.io_type=="output") {
            output_map[pixel_encoder(...pixel.coord)] = [pixel.io_index.index,pixel.color.on,pixel.color.off];
        }
    }
    console.log(input_map);
    console.log(output_map);
    document.querySelector("#graphicalIO")?.Replace([E("canvas",{width,height},[]).Proc((cnv: HTMLCanvasElement) => {
        const ctx = cnv.getContext('2d');
        ctx.fillStyle = "black";
        ctx.fillRect(0,0,width,height);

        // マウスクリックイベントを設定
        cnv.addEventListener('click', (event: MouseEvent) => {
            // CSSの拡大縮小を考慮したクリック位置を計算する
            const rect = cnv.getBoundingClientRect();
            const scaleX = cnv.width / rect.width;
            const scaleY = cnv.height / rect.height;
            const x = (event.clientX - rect.left) * scaleX;
            const y = (event.clientY - rect.top) * scaleY;
            console.log(`クリック座標: (${x}, ${y})`);
            { // inputを更新する
                const index = pixel_encoder(Math.floor(x),Math.floor(y));
                if (index in input_map) {
                    let input_index = input_map[index][0];
                    const input_elm = (Array.from(document.querySelectorAll(".input input"))[input_index] as HTMLInputElement);
                    console.log(index,input_index,input[input_index])
                    input_elm.checked = input[input_index]==1?false:true;
                    input[input_index] = input[input_index]==1?0:1;
                    console.log(index,input_index,input[input_index])
                }
            }
            graphicalIO_update();
        });

        graphicalIO_update = ()=>{
            const arr = new Uint8ClampedArray(width*height*4).fill(0);
            for (let index_ of Object.keys(input_map)) {
                const in_ = input_map[index_];
                let index = Number(index_);
                if (input[in_[0]]==1) {
                    arr[index*4+0] = in_[1][0];
                    arr[index*4+1] = in_[1][1];
                    arr[index*4+2] = in_[1][2];
                    arr[index*4+3] = 255;
                }
                else {
                    arr[index*4+0] = in_[2][0];
                    arr[index*4+1] = in_[2][1];
                    arr[index*4+2] = in_[2][2];
                    arr[index*4+3] = 255;
                }
            }
            const output = VM.getOutput(vm_id);
            for (let index_ of Object.keys(output_map)) {
                const out_ = output_map[index_];
                let index = Number(index_);
                if (output[out_[0]]==1) {
                    arr[index*4+0] = out_[1][0];
                    arr[index*4+1] = out_[1][1];
                    arr[index*4+2] = out_[1][2];
                    arr[index*4+3] = 255;
                }
                else {
                    arr[index*4+0] = out_[2][0];
                    arr[index*4+1] = out_[2][1];
                    arr[index*4+2] = out_[2][2];
                    arr[index*4+3] = 255;
                }
            }
            ctx.putImageData(new ImageData(arr,width,height),0,0);
        }
    })]);
}