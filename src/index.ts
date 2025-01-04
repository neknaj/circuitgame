import init, { Compile, CompilerIntermediateProducts, Test } from './circuitgame.js';
import { elm as E, textelm as T } from './cdom.js';


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
        const result = CompilerIntermediateProducts(input);
        console.log(result);
        console.log(result.module_dependency_sorted[0]);
        const test_result = Test(input);
        console.log(test_result);
        for (let name of Object.keys(test_result.test_result)) {
            console.log(`test: ${name}`);
            console.table(test_result.test_result[name]);
        }
        console.log(Compile(input, result.module_dependency_sorted[0]));
    }
}

run();