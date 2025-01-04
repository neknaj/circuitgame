import init, { Compile, CompilerIntermediateProducts, Test } from './circuitgame_lib.js';
import { isIntermediateProducts, isTestProducts } from './typeGuards.js';
import { IntermediateProducts, TestProducts } from './types.js';

function CompileAsTypescriptResult(code: string): IntermediateProducts {
    let result_from_rust: any = JSON.parse(CompilerIntermediateProducts(code));
    // 型チェックと変換を行う
    if (!isIntermediateProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust;
}
function TestAsTypescriptResult(input: string): TestProducts {
    let result_from_rust: any = JSON.parse(Test(input));
    if (!isTestProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust
}

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

async function run() {
    await init();
    const input = await fetchTextFile("./sample.ncg");
    {
        console.log("< Input >")
        const result = CompileAsTypescriptResult(input);
        console.log(result);
        console.log(result.module_dependency_sorted[0]);
        const test_result = TestAsTypescriptResult(input);
        console.log(test_result);
        for (let name of Object.keys(test_result.test_result)) {
            console.log(`test: ${name}`);
            console.table(test_result.test_result[name]);
        }
        console.log(Compile(input, result.module_dependency_sorted[0]));
    }
}

run();