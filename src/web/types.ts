export interface MType {
    input_count: number;
    output_count: number;
}

export interface Gate {
    outputs: string[];
    module_name: string;
    inputs: string[];
}

export interface Module {
    name: string;
    inputs: string[];
    outputs: string[];
    gates: Gate[];
}

export interface Using {
    type_sig: MType;
}

export interface Test {
    name: string;
    type_sig: MType;
    patterns: {
        inputs: boolean[];
        outputs: boolean[];
    }[];
}

export type Component =
    | { type: "Using" } & Using
    | { type: "Module" } & Module
    | { type: "Test" } & Test;

export interface File {
    components: Component[];
}

export interface ModuleType {
    name: string;
    mtype: MType;
}

export interface NodeDepends {
    node: string;
    depends: string;
}


// CompiledGateInput型の修正
export type CompiledGateInput = | { NorGate: number } | { Input: number };

// CompiledGate型の定義
type CompiledGate = [CompiledGateInput, CompiledGateInput];

// CompiledModule構造体の定義
export interface CompiledModule {
    inputs: number;
    outputs: number[];
    gates: CompiledGate[];
}

export interface IntermediateProducts {
    source: string;
    warns: string[];
    errors: string[];
    ast: File;
    module_type_list: ModuleType[];
    module_dependency: NodeDepends[];
    module_dependency_sorted: string[];
    expanded_modules: Map<string,CompiledModule>;
}

export type TestPattern = {
    accept: Boolean;
    input: Boolean[];
    expect: Boolean[];
    output: Boolean[];
}

export type TestPatternMap = {
    [key: string]: TestPattern[];
}

export type TestProducts = {
    warns: string[];
    errors: string[];
    test_list: string[];
    test_result: TestPatternMap
}