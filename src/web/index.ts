import init, { Compile, CompilerIntermediateProducts, Test, IntermediateProducts } from './circuitgame.js';
import { elm as E, textelm as T } from './cdom.js';
import VMinit from './vm.js';

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
    editor.getSession().on('change',update);
    editor.setValue(await fetchTextFile("./sample.ncg"));
    editor.moveCursorTo(0, 0);
    editor.setFontSize(15);
}

async function update() {
    const input = ace.edit("editor").getValue();
    const result = CompilerIntermediateProducts(input);
    console.log(result);
    console.log(result.module_dependency_sorted[0]);
    try {
        const test_result = Test(input);
        console.log(test_result);
        for (let name of Object.keys(test_result.test_result)) {
            console.log(`test: ${name}`);
            console.table(test_result.test_result[name]);
        }
    }
    catch (e) {
        console.error(e);
    }
    // VM
    VMinit(document.querySelector("#vm"),result);
}

document.addEventListener("moduleChanged", (event) => {
    console.log(event);
    if ("detail" in event) {
        let data = event.detail as { module_name: string, product: IntermediateProducts };
        upadteGraph(data.product,data.module_name);
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
%%{init: {'theme':'dark'}}%%\n
graph TD\n
${new Array(expanded.inputs).fill(0).map((v,i)=>`in${i}(in ${i})`).join("\n")}
${expanded.outputs.map((v,i)=>`out${i}(out ${i})`).join("\n")}
${expanded.gates.map((g,i)=>`nor${i}[nor]`).join("\n")}
${wires}
${output_wires}
        `;
        console.log(graphDefinition);
        const elm = document.querySelector("#graph");
        // console.log(elm);
        const res = await mermaid.render("mermaidGraph", graphDefinition);
        // console.log(res)
        elm.innerHTML = res.svg;
    }
}

async function run() {
    await init();
    initlayout(
        document.querySelector("#layoutroot"),
        ["h",[3,5],[
            ["v",[1,1],[
                ["c","vmArea"],
                ["c","graphArea"],
            ]],
            ["c","editArea"],
        ]],
        {
            vmArea: ()=>{return E("div",{id:"vm"},[])},
            graphArea: ()=>{return E("div",{id:"graph"},[])},
            editArea: ()=>{return E("div",{id:"editor"},[])},
        }
    )
    await initEditor();
    // await initGraph();
}

run();