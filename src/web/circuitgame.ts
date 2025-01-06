import init, { CompilerIntermediateProducts as CompilerIntermediateProducts_raw, Test as Test_raw, Compile, VMinit, VMreset, VMset, VMgetOutput, VMgetGates, VMnext } from './circuitgame_lib.js';
import { IntermediateProducts, TestProducts } from './types.js';
import { isIntermediateProducts, isTestProducts } from './typeGuards.js';

function CompilerIntermediateProducts(code: string): IntermediateProducts {
    let result_from_rust: any = JSON.parse(CompilerIntermediateProducts_raw(code));
    // 型チェックと変換を行う
    if (!isIntermediateProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust;
}
function Test(input: string): TestProducts {
    let result_from_rust: any = JSON.parse(Test_raw(input));
    if (!isTestProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust
}

const VM = {
    init: VMinit,
    reset: VMreset,
    set: VMset,
    getOutput: VMgetOutput,
    getGates: VMgetGates,
    next: VMnext,
}

export { CompilerIntermediateProducts, Test, Compile, VM};
export { IntermediateProducts, TestProducts };
export default init;