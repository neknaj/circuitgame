import { Component, File, Gate, IntermediateProducts, Module, ModuleType, MType, NodeDepends, Test, TestProducts, Using } from './types';

export function isIntermediateProducts(obj: any): obj is IntermediateProducts {
    if (!obj || typeof obj !== 'object') return false;

    // 必須プロパティの存在チェック（snake_caseに対応）
    if (!('source' in obj && 'warns' in obj && 'errors' in obj && 'ast' in obj &&
        'module_type_list' in obj && 'module_dependency' in obj &&
        'module_dependency_sorted' in obj)) return false;

    // 配列プロパティの型チェック
    // if (!Array.isArray(obj.warns) || !obj.warns.every(w => typeof w === 'string')) return false;
    // if (!Array.isArray(obj.errors) || !obj.errors.every(e => typeof e === 'string')) return false;
    // if (!Array.isArray(obj.module_type_list) || !obj.module_type_list.every(isModuleType)) return false;
    // if (!Array.isArray(obj.module_dependency) || !obj.module_dependency.every(isNodeDepends)) return false;
    // if (!Array.isArray(obj.module_dependency_sorted) || !obj.module_dependency_sorted.every(m => typeof m === 'string')) return false;

    // expanded_modulesの型チェック

    // astの型チェック
    // if (!isFile(obj.ast)) return false;

    return true;
}

export function isModuleType(obj: any): obj is ModuleType {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        isMType(obj.mtype);
}

export function isMType(obj: any): obj is MType {
    return obj && typeof obj === 'object' &&
        typeof obj.input_count === 'number' &&
        typeof obj.output_count === 'number';
}

export function isNodeDepends(obj: any): obj is NodeDepends {
    return obj && typeof obj === 'object' &&
        typeof obj.node === 'string' &&
        typeof obj.depends === 'string';
}

export function isFile(obj: any): obj is File {
    return obj && typeof obj === 'object' &&
        Array.isArray(obj.components) &&
        obj.components.every(isComponent);
}

export function isComponent(obj: any): obj is Component {
    if (!obj || typeof obj !== 'object') return false;
    if (!['Using', 'Module', 'Test', 'Graphical'].includes(obj.type)) return false;

    switch (obj.type) {
        case 'Using': return isUsing(obj);
        case 'Module': return isModule(obj);
        case 'Test': return isTest(obj);
        default: return false;
    }
}

export function isUsing(obj: any): obj is Using {
    return obj && typeof obj === 'object' && isMType(obj.type_sig);
}

export function isModule(obj: any): obj is Module {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        Array.isArray(obj.inputs) &&
        obj.inputs.every(i => typeof i === 'string') &&
        Array.isArray(obj.outputs) &&
        obj.outputs.every(o => typeof o === 'string') &&
        Array.isArray(obj.gates) &&
        obj.gates.every(isGate);
}

export function isGate(obj: any): obj is Gate {
    return obj && typeof obj === 'object' &&
        Array.isArray(obj.outputs) &&
        obj.outputs.every(o => typeof o === 'string') &&
        typeof obj.module_name === 'string' &&
        Array.isArray(obj.inputs) &&
        obj.inputs.every(i => typeof i === 'string');
}

export function isTest(obj: any): obj is Test {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        isMType(obj.type_sig) &&
        Array.isArray(obj.patterns) &&
        obj.patterns.every(p =>
            Array.isArray(p.inputs) &&
            p.inputs.every(i => typeof i === 'boolean') &&
            Array.isArray(p.outputs) &&
            p.outputs.every(o => typeof o === 'boolean')
        );
}

export function isTestProducts(obj: any): obj is TestProducts {
    if (!obj || typeof obj !== 'object') return false;

    // 必須プロパティの存在チェック
    if (!('warns' in obj && 'errors' in obj && 'test_list' in obj && 'test_result' in obj)) return false;

    // 配列プロパティの型チェック
    if (!Array.isArray(obj.warns) || !obj.warns.every(w => typeof w === 'string')) return false;
    if (!Array.isArray(obj.errors) || !obj.errors.every(e => typeof e === 'string')) return false;
    if (!Array.isArray(obj.test_list) || !obj.test_list.every(t => typeof t === 'string')) return false;

    // test_resultの型チェック
    if (typeof obj.test_result !== 'object') return false;

    // test_resultの各要素（TestPattern[]）をチェック
    return Object.values(obj.test_result).every(patterns => {
        if (!Array.isArray(patterns)) return false;
        return patterns.every(pattern =>
            typeof pattern === 'object' &&
            typeof pattern.accept === 'boolean' &&
            Array.isArray(pattern.input) &&
            pattern.input.every(i => typeof i === 'boolean') &&
            Array.isArray(pattern.expect) &&
            pattern.expect.every(e => typeof e === 'boolean') &&
            Array.isArray(pattern.output) &&
            pattern.output.every(o => typeof o === 'boolean')
        );
    });
}  