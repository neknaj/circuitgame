import { Compile, VM } from './circuitgame.js';
import { elm as E, textelm as T, textelm } from './cdom.js';
import { IntermediateProducts, CompiledModule, CompiledGateInput } from './types.js';

let vm_id = null;
let selected = null;
let input = [];
let compiledModule: CompiledModule;

function init(elm: HTMLDivElement,product: IntermediateProducts,module_name: string = null) {
    if (module_name!=null&&product.module_dependency_sorted.includes(module_name)==false) { module_name = null; }
    if (selected!=null&&product.module_dependency_sorted.includes(selected)==false) { selected = null; }
    if (module_name == null) {
        if (selected == null) { selected = product.module_dependency_sorted[0]; }
        module_name = selected;
    }
    elm.innerHTML = "";
    console.log("module",module_name,selected);
    // module選択のドロップダウンメニューを追加
    elm.Add(E("select",{},product.module_dependency_sorted.map(
        (m,i) => {
            let option = E("option",{value:m},[textelm(m)]);
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
    console.log(Compile(product.source,module_name));
    const moduleType = product.module_type_list.filter(m=>m.name==module_name)[0];
    console.log(moduleType);
    elm.Add(E("h1",{},[textelm(module_name)]));
    elm.Add(E("h2",{},[textelm("input")]));
    elm.Add(E("div",{class:"input"},Array(moduleType.mtype.input_count).fill(0).map(
        (_,i) => E("span",{},[
            (() => {
                let inputElm = E("input",{type:"checkbox",id:"input"+i},[])
                if (input.length==moduleType.mtype.input_count&&input[i]==true) {
                    inputElm.setAttribute("checked","true");
                }
                return inputElm;
            })(),
            E("label",{for:"input"+i},[])
        ]),
    )));
    elm.Add(E("h2",{},[textelm("output")]));
    elm.Add(E("div",{class:"output"},Array(moduleType.mtype.output_count).fill(0).map(
        (_,i) => E("span",{},[
            E("input",{type:"checkbox",id:"output"+i,disabled:true},[]),
            E("label",{for:"output"+i},[])
        ]),
    )));
    elm.Add(E("h2",{},[T("tick")]));
    elm.Add(E("p",{id:"tick"},[]));
    vm_id = VM.init(Compile(product.source,module_name));
    {
        const myEvent = new CustomEvent("moduleChanged", {
            detail: { module_name, product: product },
            bubbles: true, // イベントが親要素に伝播する
            cancelable: true, // イベントをキャンセル可能にする
        });
        elm.dispatchEvent(myEvent);
    }
    compiledModule = product.expanded_modules[module_name];
}

function updateOutput() {
    if (vm_id==null) { return; }
    // console.log(vm_id);
    input = Array.from(document.querySelectorAll(".input input"))
                .map(e=>(e as HTMLInputElement).checked)
    input.forEach((v,i)=>VM.set(vm_id,i,v));
    VM.next(vm_id,1);
    (Array.from(document.querySelectorAll(".output input")) as HTMLInputElement[])
        .forEach((e,i)=>e.checked = VM.getOutput(vm_id)[i]==1?true:false);
    (document.querySelector("#tick") as HTMLParagraphElement).innerText = `${VM.getTick(vm_id)}`;
    // グラフの色を反映
    {
        // console.log(document.querySelectorAll("#graph .node.input"))
        document.querySelectorAll("#graph .node.input").forEach((node,i)=>{
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
        document.querySelectorAll("#graph .node.gate").forEach((node,i)=>{
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
        document.querySelectorAll("#graph .node.output").forEach((node,i)=>{
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
        const wires = compiledModule.gates.flat(1).concat(compiledModule.outputs.map(x=>({NorGate:x} as CompiledGateInput)));
        // console.log(wires)
        // console.log(document.querySelectorAll("#graph .edgePaths path"))
        document.querySelectorAll("#graph .edgePaths path").forEach((node,i)=>{
            let active = false;
            // console.log(wires[i])
            if ("NorGate" in wires[i]) {
                active = gate[wires[i].NorGate];
            }
            else {
                active = input[wires[i].Input];
            }
            if (active) {
                node.classList.add("active");
            }
            else {
                node.classList.remove("active");
            }
        });
    }
}
setInterval(updateOutput, 10);

export default init;