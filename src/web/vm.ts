import { Compile, VM } from './circuitgame.js';
import { elm as E, textelm as T, textelm } from './cdom.js';
import { IntermediateProducts } from './types.js';

let vm_id = 0;
let selected = null;
let input = [];

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
    vm_id = VM.init(Compile(product.source,module_name));
    setInterval(updateOutput, 10);
}

function updateOutput() {
    console.log(vm_id);
    input = Array.from(document.querySelectorAll(".input input"))
                .map(e=>(e as HTMLInputElement).checked)
    input.forEach((v,i)=>VM.set(vm_id,i,v));
    VM.next(vm_id,1);
    (Array.from(document.querySelectorAll(".output input")) as HTMLInputElement[])
        .forEach((e,i)=>e.checked = VM.getOutput(vm_id)[i]==1?true:false);
}

export default init;