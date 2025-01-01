export function isIntermediateProducts(obj) {
    if (!obj || typeof obj !== 'object')
        return false;
    // 必須プロパティの存在チェック（snake_caseに対応）
    if (!('warns' in obj && 'errors' in obj && 'ast' in obj &&
        'module_type_list' in obj && 'module_dependency' in obj &&
        'module_dependency_sorted' in obj))
        return false;
    // 配列プロパティの型チェック
    if (!Array.isArray(obj.warns) || !obj.warns.every(function (w) { return typeof w === 'string'; }))
        return false;
    if (!Array.isArray(obj.errors) || !obj.errors.every(function (e) { return typeof e === 'string'; }))
        return false;
    if (!Array.isArray(obj.module_type_list) || !obj.module_type_list.every(isModuleType))
        return false;
    if (!Array.isArray(obj.module_dependency) || !obj.module_dependency.every(isNodeDepends))
        return false;
    if (!Array.isArray(obj.module_dependency_sorted) || !obj.module_dependency_sorted.every(function (m) { return typeof m === 'string'; }))
        return false;
    // astの型チェック
    if (!isFile(obj.ast))
        return false;
    return true;
}
export function isModuleType(obj) {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        isMType(obj.mtype);
}
export function isMType(obj) {
    return obj && typeof obj === 'object' &&
        typeof obj.input_count === 'number' &&
        typeof obj.output_count === 'number';
}
export function isNodeDepends(obj) {
    return obj && typeof obj === 'object' &&
        typeof obj.node === 'string' &&
        typeof obj.depends === 'string';
}
export function isFile(obj) {
    return obj && typeof obj === 'object' &&
        Array.isArray(obj.components) &&
        obj.components.every(isComponent);
}
export function isComponent(obj) {
    if (!obj || typeof obj !== 'object')
        return false;
    if (!['Using', 'Module', 'Test'].includes(obj.type))
        return false;
    switch (obj.type) {
        case 'Using': return isUsing(obj);
        case 'Module': return isModule(obj);
        case 'Test': return isTest(obj);
        default: return false;
    }
}
export function isUsing(obj) {
    return obj && typeof obj === 'object' && isMType(obj.type_sig);
}
export function isModule(obj) {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        Array.isArray(obj.inputs) &&
        obj.inputs.every(function (i) { return typeof i === 'string'; }) &&
        Array.isArray(obj.outputs) &&
        obj.outputs.every(function (o) { return typeof o === 'string'; }) &&
        Array.isArray(obj.gates) &&
        obj.gates.every(isGate);
}
export function isGate(obj) {
    return obj && typeof obj === 'object' &&
        Array.isArray(obj.outputs) &&
        obj.outputs.every(function (o) { return typeof o === 'string'; }) &&
        typeof obj.module_name === 'string' &&
        Array.isArray(obj.inputs) &&
        obj.inputs.every(function (i) { return typeof i === 'string'; });
}
export function isTest(obj) {
    return obj && typeof obj === 'object' &&
        typeof obj.name === 'string' &&
        isMType(obj.type_sig) &&
        Array.isArray(obj.patterns) &&
        obj.patterns.every(function (p) {
            return Array.isArray(p.inputs) &&
                p.inputs.every(function (i) { return typeof i === 'boolean'; }) &&
                Array.isArray(p.outputs) &&
                p.outputs.every(function (o) { return typeof o === 'boolean'; });
        });
}
