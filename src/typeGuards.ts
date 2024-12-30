import { Component, File, Gate, IntermediateProducts, Module, ModuleType, MType, NodeDepends, Test, Using } from './types';

export function isIntermediateProducts(obj: any): obj is IntermediateProducts {
    if (!obj || typeof obj !== 'object') return false;

    // 必須プロパティの存在チェック（snake_caseに対応）
    if (!('warns' in obj && 'errors' in obj && 'ast' in obj &&
        'module_type_list' in obj && 'module_dependency' in obj &&
        'module_dependency_sorted' in obj)) return false;

    // 配列プロパティの型チェック
    if (!Array.isArray(obj.warns) || !obj.warns.every(w => typeof w === 'string')) return false;
    if (!Array.isArray(obj.errors) || !obj.errors.every(e => typeof e === 'string')) return false;
    if (!Array.isArray(obj.module_type_list) || !obj.module_type_list.every(isModuleType)) return false;
    if (!Array.isArray(obj.module_dependency) || !obj.module_dependency.every(isNodeDepends)) return false;
    if (!Array.isArray(obj.module_dependency_sorted) || !obj.module_dependency_sorted.every(m => typeof m === 'string')) return false;

    // astの型チェック
    if (!isFile(obj.ast)) return false;

    // 型変換して返す
    obj.moduleTypeList = obj.module_type_list;
    obj.moduleDependency = obj.module_dependency;
    obj.moduleDependencySorted = obj.module_dependency_sorted;

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
    if (!['Using', 'Module', 'Test'].includes(obj.type)) return false;

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