import init, { CompilerIntermediateProducts as CompilerIntermediateProducts_raw, Test as Test_raw, Compile, Module, VMreset, VMset, VMgetOutput, VMgetGates, VMgetTick, VMnext, TranspileTS, TranspileTSresId } from './circuitgame_lib.js';
import { IntermediateProducts, TestProducts } from './types.js';
import { isIntermediateProducts, isTestProducts } from './typeGuards.js';

function CompilerIntermediateProducts(code: string): IntermediateProducts {
    let result_from_rust: any;
    try {
        result_from_rust = JSON.parse(CompilerIntermediateProducts_raw(code))
    }
    catch (e) {
        console.error(e);
        console.log(CompilerIntermediateProducts_raw(code));
    }
    console.log(result_from_rust.ast.components)
    // 型チェックと変換を行う
    if (!isIntermediateProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust;
}
function Test(input: string): TestProducts {
    const res = Test_raw(input);
    if (res[0]!="{") { return {warns:[],errors:[],test_list:[],test_result:{}}; } // コンパイルエラーは空の結果を返す
    let result_from_rust: any = JSON.parse(res);
    if (!isTestProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません'); // 型がおかしいのはthrow
    }
    return result_from_rust
}

const VM = {
    init: Module,
    reset: VMreset,
    set: VMset,
    getOutput: VMgetOutput,
    getGates: VMgetGates,
    getTick: VMgetTick,
    next: VMnext,
}

const Transpiler = {
    TS: {
        TranspileTS,
        TranspileTSresId
    }
}

export { CompilerIntermediateProducts, Test as NCG_Test, Compile, VM, Module, Transpiler};
export { IntermediateProducts, TestProducts };
export default init;