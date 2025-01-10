import init, { Compile, CompilerIntermediateProducts, Test, IntermediateProducts } from './circuitgame.js';
import { elm as E, textelm as T } from './cdom.js';
import VMinit, { tick, updateLogiAnaGraph } from './vm.js';

import ace from "ace-builds/src-noconflict/ace";
import { CustomMode, darkTheme } from "./editor.mode.js";

import mermaid from "mermaid";

import { initlayout } from "./layout.js";

async function fetchTextFile(url: string): Promise<string> {
    try {
        // fetchでリソースを取得
        const response = await fetch(url);

        // レスポンスが成功しているかを確認
        if (!response.ok) {
            throw new Error(`HTTP Error: ${response.status}`);
        }

        // テキストとしてレスポンスを取得
        const text = await response.text();
        return text;
    } catch (error) {
        console.error("テキストファイルの取得中にエラーが発生しました:", error);
        throw error;
    }
}

async function initEditor() {
    ace.define("ace/theme/custom_theme", ["require", "exports", "module", "ace/lib/dom"], darkTheme);
    // Aceエディタを初期化
    var editor = ace.edit("editor");
    editor.setTheme("ace/theme/custom_theme"); // テーマの設定
    editor.session.setMode(new CustomMode());
    editor.getSession().on('change',()=>{
        if ((document.querySelector("#autoCompile") as HTMLInputElement).checked) {
            update();
        }
    });
    editor.setValue(await fetchTextFile("./sample.ncg"));
    editor.moveCursorTo(0, 0);
    editor.setFontSize(15);
}

async function update() {
    const input = ace.edit("editor").getValue();
    const result = CompilerIntermediateProducts(input);
    console.log(result);
    // console.log(result.module_dependency_sorted[0]);
    const test_result = Test(input);
    console.log(test_result);
    // for (let name of Object.keys(test_result.test_result)) {
    //     console.log(`test: ${name}`);
    //     console.table(test_result.test_result[name]);
    // }
    // VM
    setErrMsg(result,test_result);
    VMinit(document.querySelector("#vm"),result);
}

function setErrMsg(compiler_products: IntermediateProducts,test_products: TestProducts) {
    { // リスト
        let elm = document.querySelector("#errMsgArea");
        elm.innerHTML = "";
        elm.Add(E("ul",{},compiler_products.errors.map(x=>E("li",{class:"error"},[T(x)]))));
        elm.Add(E("ul",{},compiler_products.warns.map(x=>E("li",{class:"warn"},[T(x)]))));
        elm.Add(E("ul",{},test_products.errors.map(x=>E("li",{class:"error"},[T(x)]))));
        elm.Add(E("ul",{},test_products.warns.map(x=>E("li",{class:"warn"},[T(x)]))));
    }
}


let graphModuleData: { module_name: string, product: IntermediateProducts };
document.addEventListener("moduleChanged", (event) => {
    console.log(event);
    if ("detail" in event) {
        graphModuleData = event.detail as { module_name: string, product: IntermediateProducts };
        if ((document.querySelector("#graph1_switch") as HTMLInputElement).checked) {
            upadteGraph(graphModuleData.product,graphModuleData.module_name);
        }
    }
});
async function upadteGraph(product: IntermediateProducts,module_name: string) {
    console.log(product.expanded_modules[module_name]);
    const expanded = product.expanded_modules[module_name];
    const wires = expanded.gates.map((g,i)=>g.map((v)=>{
        if ("NorGate" in v) {
            return `nor${v.NorGate} --> nor${i}`;
        }
        else {
            return `in${v.Input} --> nor${i}`;
        }
    }).join("\n")).join("\n")
    const output_wires = expanded.outputs.map((v,i)=>`nor${v} --> out${i}`).join("\n");
    // Graph
    {
        const graphDefinition = `
%%{init: {'theme':'dark'}}%%
graph TD

${expanded.inputs>0?"subgraph Inputs":""}
${new Array(expanded.inputs).fill(0).map((v,i)=>`in${i}(in ${i})`).join("\n")}
${expanded.inputs>0?"end":""}

${expanded.gates.length>0?`subgraph Gates[${module_name}]`:""}
${constructGraph(product,module_name)[0]}
${expanded.gates.length>0?"end":""}

${expanded.outputs.length>0?"subgraph Outputs":""}
${expanded.outputs.map((v,i)=>`out${i}(out ${i})`).join("\n")}
${expanded.outputs.length>0?"end":""}

${new Array(expanded.inputs).fill(0).map((v,i)=>`class in${i} input;`).join("\n")}
${expanded.gates.map((g,i)=>`class nor${i} gate;`).join("\n")}
${expanded.outputs.map((v,i)=>`class out${i} output;`).join("\n")}

${wires}
${output_wires}
        `;
        console.log(graphDefinition);
        const elm = document.querySelector("#graph1");
        // console.log(elm);
        const res = await mermaid.render("mermaidGraph", graphDefinition);
        // console.log(res)
        elm.innerHTML = res.svg;
    }
}

import { Module, TestProducts } from './types.js';
function constructGraph(product: IntermediateProducts,module_name: string,offset: number=0,subgraph=0): [string,number,number] {
    if (module_name=="nor") { return [`nor${offset}\n`,offset+1,subgraph]; }
    const modulesAST = module_name!="nor"?(product.ast.components.filter(x=>x.type=="Module"&&x.name==module_name)[0] as Module):{name:"nor",inputs:["x","y"],outputs:["a"],gates:[{inputs:["x","y"],outputs:["a"],module_name:"nor"}]};
    const gates = modulesAST.gates;
    console.log(gates,module_name)
    let result = "";
    for (let gate of gates) {
        if (gate.module_name=="nor") {
            result += `nor${offset}[nor]\n`
            offset+=1;
        }
        else {
            subgraph+=1;
            result += `subgraph ${subgraph}[${gate.module_name}]\n`;
            {
                const res = constructGraph(product,gate.module_name,offset,subgraph);
                result += res[0];
                offset = res[1];
                subgraph = res[2];
            }
            result += `end\n`;
        }
    }
    console.log(result)
    return [result,offset,subgraph];
}

async function run() {
    await init();
    initlayout(
        document.querySelector("#layoutroot"),
        ["h",[3,1],[
            ["h",[3,1],[
                ["v",[2,1],[
                    ["h",[1,3],[
                        ["v",[2,1],[
                            ["c","vmArea"],
                            ["c","vmCtrlArea"],
                        ]],
                        ["c","graph1Area"],
                    ]],
                    ["c","graph2Area"],
                ]],
                ["c","errMsgArea"],
            ]],
            ["c","editArea"],
        ]],
        {
            vmArea: ()=>{return E("div",{id:"vm"},[])},
            graph1Area: ()=>{return E("div",{id:"graph1"},[])},
            graph2Area: ()=>{return E("div",{id:"graph2"},[])},
            editArea: ()=>E("div",{id:"editor_area"},[
                E("div",{id:""},[
                    E("input",{type:"button",value:"compile"},[]).Listen("click",update),
                    E("span",{},[
                        E("input",{type:"checkbox",id:"autoCompile",checked:true},[]),
                        E("label",{for:"autoCompile"},[T("auto compile")]),
                    ]),
                ]),
                E("div",{id:"editor"},[]),
            ]),
            errMsgArea: ()=>{return E("div",{id:"errMsgArea"},[])},
            vmCtrlArea: ()=>E("div",{id:"vm_ctrl_area"},[
                E("div",{class:"prop"},[
                    E("input",{type:"checkbox",id:"graph1_switch",checked:true},[]).Listen("change",()=>{
                        if ((document.querySelector("#graph1_switch") as HTMLInputElement).checked) {
                            upadteGraph(graphModuleData.product,graphModuleData.module_name);
                        }
                        else {
                            document.querySelector("#graph1").innerHTML = "";
                            document.querySelector("#graph1").Add(E("p",{},[T("This graph is disabled.")]));
                        }
                    }),
                    E("label",{for:"graph1_switch"},[T("graph1")]),
                    E("input",{type:"checkbox",id:"graph2_switch",checked:true},[]).Listen("change",()=>{
                        if ((document.querySelector("#graph2_switch") as HTMLInputElement).checked) {
                            updateLogiAnaGraph();
                        }
                        else {
                            document.querySelector("#graph2").innerHTML = "";
                            document.querySelector("#graph2").Add(E("p",{},[T("This graph is disabled.")]));
                        }
                    }),
                    E("label",{for:"graph2_switch"},[T("graph2")]),
                ]),
                E("div",{class:"prop"},[
                    E("input",{type:"checkbox",id:"vmRun",checked:true},[]),
                    E("label",{for:"vmRun"},[T("run")]),
                    E("button",{},[T("tick")]).Listen("click",tick),
                    E("label",{for:"vmSpeed",id:"vmSpeed_label"},[]),
                    E("input",{type:"range",id:"vmSpeed",min:0,max:1000,step:1,value:0},[]).Listen("input",()=>{
                        (document.querySelector("#vmSpeed_label") as HTMLLabelElement).innerText = (document.querySelector("#vmSpeed") as HTMLInputElement).value;
                        updateLogiAnaGraph();
                    }),
                ]),
                E("div",{class:"prop"},[
                    E("label",{for:"digiAnaLastN",id:"digiAnaLastN_label"},[]),
                    E("input",{type:"range",id:"digiAnaLastN",min:20,max:2000,step:1,value:100},[]).Listen("input",()=>{
                        (document.querySelector("#digiAnaLastN_label") as HTMLLabelElement).innerText = (document.querySelector("#digiAnaLastN") as HTMLInputElement).value;
                        updateLogiAnaGraph();
                    }),
                ]),
            ]),
        }
    )
    {
        (document.querySelector("#vmSpeed_label") as HTMLLabelElement).innerText = (document.querySelector("#vmSpeed") as HTMLInputElement).value;
        (document.querySelector("#digiAnaLastN_label") as HTMLLabelElement).innerText = (document.querySelector("#digiAnaLastN") as HTMLInputElement).value;
    }
    {
        mermaid.initialize({
            maxEdges: 1000
        })
    }
    await initEditor();
    // update();
}

run();