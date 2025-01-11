import init, { Compile, NCG_Test, CompilerIntermediateProducts, IntermediateProducts } from './circuitgame.js';
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

let socket = null;
let editor;
let retryCount = 0;
function initWebSocket(path) {
    const reconnectInterval = 3000; // 再接続間隔（ミリ秒）
    const maxRetries = 20; // 最大再接続回数
    console.log(path);
    socket = new WebSocket(path);
    socket.onmessage = function(event) {
        const message = event.data as string;
        console.log(message)
        if (message.startsWith("file:")) {
            if (message.slice(5).length<1) {
                socket.send("get file");
                return;
            }
            editor.setValue(`# received from ${path}\n# ${new Date()}\n\n${message.slice(5)}`);
            editor.moveCursorTo(0, 0);
        }
    };
    socket.onopen = function() {
        socket.send("get file");
    };

    socket.onerror = (error) => {
        console.error("WebSocket error:", error);
    };

    socket.onclose = (event) => {
        console.warn("WebSocket connection closed:", event);

        // 再接続を試みる
        if (retryCount < maxRetries) {
            retryCount++;
            console.log(`Retrying connection (${retryCount}/${maxRetries}) in ${reconnectInterval / 1000} seconds...`);
            setTimeout(initWebSocket, reconnectInterval, path);
        } else {
            console.error("Maximum retry attempts reached. Connection failed.");
        }
    };
}

async function initEditor() {
    ace.define("ace/theme/custom_theme", ["require", "exports", "module", "ace/lib/dom"], darkTheme);
    // Aceエディタを初期化
    editor = ace.edit("editor");
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
    const test_result  = NCG_Test(input);
    console.log(test_result);
    // for (let name of Object.keys(test_result.test_result)) {
    //     console.log(`test: ${name}`);
    //     console.table(test_result.test_result[name]);
    // }
    // VM
    setErrMsg(result,test_result);
    setModuleInfo(result);
    VMinit(document.querySelector("#vm"),result);
}

function setErrMsg(compiler_products: IntermediateProducts,test_products: TestProducts) {
    // 普通にリストで表示
    document.querySelector("#errMsgArea").Replace([
        E("ul",{},compiler_products.errors.map(x=>E("li",{class:"error"},[T(x)]))),
        E("ul",{},compiler_products.warns.map(x=>E("li",{class:"warn"},[T(x)]))),
        E("ul",{},test_products.errors.map(x=>E("li",{class:"error"},[T(x)]))),
        E("ul",{},test_products.warns.map(x=>E("li",{class:"warn"},[T(x)]))),
    ]);
    // テスト結果の表
    document.querySelector("#testResult").Replace(Object.keys(test_products.test_result).map(
        name=>{
            const accept = !test_products.test_result[name].some(x=>!x.accept);
            let detail = E("details",{class:[accept?"accept":"failed"]},[
                E("summary",{},[
                    T(name)
                ]),
                E("table",{},[
                    E("thead",{},[E("tr",{},[
                        E("th",{},[T("accept")]),
                        E("th",{},[T("input")]),
                        E("th",{},[T("output")]),
                        E("th",{},[T("expect")]),
                    ])]),
                    E("tbody",{},
                        test_products.test_result[name].map(x=>E("tr",{class:x.accept?"accept":"failed"},[
                            E("td",{},[T(x.accept?"true":"false")]),
                            E("td",{class:"boolean"},[T(x.input.length>0?x.input.map(x=>x?"t":"f").join(" "):"-")]),
                            E("td",{class:"boolean"},[T(x.output.length>0?x.output.map(x=>x?"t":"f").join(" "):"-")]),
                            E("td",{class:"boolean"},[T(x.expect.length>0?x.expect.map(x=>x?"t":"f").join(" "):"-")]),
                        ]))
                    ),
                ])
            ]);
            // エラーあるやつだけ詳細表示をデフォルトopen
            if (!accept) { detail.addProp({open:true}); }
            return detail;
        }
    ));
}

function setModuleInfo(product: IntermediateProducts) {
    console.log("modules",product.ast.components.filter(x=>x.type=="Module").map(x=>x.name));
    document.querySelector("#moduleInfo").Replace([E("table",{},[
        E("thead",{},[
            E("tr",{},[
                E("th",{},[T("Name")]),
                E("th",{},[T("Type")]),
                E("th",{},[T("Size")]),
            ])
        ]),
        E("tbody",{},product.ast.components.filter(x=>x.type=="Module").map(x=>x.name).map(
            name=>E("tr",{},[
                E("th",{},[T(name)]),
                E("td",{},[
                    T(product.expanded_modules[name].inputs),
                    E("span",{class:"dark"},[T("->")]),
                    T(product.expanded_modules[name].outputs.length),
                ]),
                E("td",{},[T(product.expanded_modules[name].gates.length)]),
            ])
        )),
    ])]);
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

import { Module, Test, TestProducts } from './types.js';
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
        ["h",[4,1],[
            ["h",[1,5],[
                ["c","moduleInfo"],
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
                    ["v",[3,3],[
                        ["c","errMsgArea"],
                        ["c","testResult"],
                    ]]
                ]],
            ]],
            ["c","editArea"],
        ]],
        {
            moduleInfo: ()=>{return E("div",{id:"moduleInfo"},[])},
            vmArea: ()=>{return E("div",{id:"vm"},[])},
            graph1Area: ()=>{return E("div",{id:"graph1"},[])},
            graph2Area: ()=>{return E("div",{id:"graph2"},[])},
            editArea: ()=>E("div",{id:"editor_area"},[
                E("div",{id:""},[
                    E("input",{type:"button",value:"compile"},[]).Listen("click",update),
                    E("span",{},[
                        E("input",{type:"checkbox",id:"autoCompile",checked:true},[]),
                        E("label",{for:"autoCompile"},[T("compile")]),
                    ]),
                    E("input",{type:"checkbox",id:"webSocket"},[]).Listen("change",()=>{
                        if ((document.querySelector("#webSocket") as HTMLInputElement).checked) {
                            document.querySelector("#webSocketURL").classList.remove("hide");
                            editor.setReadOnly(true);
                            initWebSocket((document.querySelector("#webSocketURL") as HTMLInputElement).value);
                        }
                        else {
                            document.querySelector("#webSocketURL").classList.add("hide");
                            editor.setReadOnly(false);
                            try {
                                socket.close();
                            } catch (e) {}
                        }
                    }),
                    E("label",{for:"webSocket"},[T("Server")]),
                    E("input",{type:"url",id:"webSocketURL"},[]).Listen("change",()=>{
                        if ((document.querySelector("#webSocket") as HTMLInputElement).checked) {
                            document.querySelector("#webSocketURL").classList.remove("hide");
                            editor.setReadOnly(true);
                            initWebSocket((document.querySelector("#webSocketURL") as HTMLInputElement).value);
                        }
                        else {
                            document.querySelector("#webSocketURL").classList.add("hide");
                            editor.setReadOnly(false);
                            try {
                                socket.close();
                            } catch (e) {}
                        }
                    }),
                ]),
                E("div",{id:"editor"},[]),
            ]),
            errMsgArea: ()=>{return E("div",{id:"errMsgArea"},[])},
            testResult: ()=>{return E("div",{id:"testResult"},[])},
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
    await initEditor();
    {
        (document.querySelector("#vmSpeed_label") as HTMLLabelElement).innerText = (document.querySelector("#vmSpeed") as HTMLInputElement).value;
        (document.querySelector("#digiAnaLastN_label") as HTMLLabelElement).innerText = (document.querySelector("#digiAnaLastN") as HTMLInputElement).value;
        if ((document.querySelector("#webSocket") as HTMLInputElement).checked) {
            document.querySelector("#webSocketURL").classList.remove("hide");
            editor.setReadOnly(true);
            initWebSocket((document.querySelector("#webSocketURL") as HTMLInputElement).value);
        }
        else {
            document.querySelector("#webSocketURL").classList.add("hide");
            editor.setReadOnly(false);
            try {
                socket.close();
            } catch (e) {}
        }
    }
    {
        mermaid.initialize({
            maxEdges: 1000
        })
    }
    { // urlパラメータ
        let param = new URLSearchParams(new URL(location.href).search);
        console.log(param);
        if (param.has("socket")) {
            (document.querySelector("#webSocketURL") as HTMLInputElement).value = param.get("socket");
            (document.querySelector("#webSocket") as HTMLInputElement).checked = true;
            document.querySelector("#webSocketURL").classList.remove("hide");
            editor.setReadOnly(true);
            initWebSocket((document.querySelector("#webSocketURL") as HTMLInputElement).value);
        }
    }
    // update();
}

run();