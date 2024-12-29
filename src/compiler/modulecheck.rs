use super::types::*;

pub fn collect_modules(ast: &File) -> Vec<ModuleType> {
    let mut modules = Vec::new();

    // NOR:2->1 は常に宣言される
    modules.push(
        ModuleType {
            name: String::from("nor"),
            mtype: MType { input_count: 2, output_count: 1 },
        }
    );

    // ASTの中で定義されたモジュールを集める
    for component in &ast.components {
        match component {
            Component::Module(module)=>{
                modules.push(
                    ModuleType {
                        name: String::from(module.name.clone()),
                        mtype: MType { input_count: module.inputs.len(), output_count: module.outputs.len() },
                    }
                );
            },
            _ => {} // モジュールでなければ何もしない
        }
    }
    return modules;
}

pub fn check_module_name_duplicates(modules: &Vec<ModuleType>) -> Result<(),String> {
    let mut module_names = std::collections::HashSet::new();
    let mut errors = Vec::new();
    for module in modules {
        if !module_names.insert(&module.name) {
            errors.push(format!("Defined module name Duplicated: {}",module.name));
        }
    }
    if errors.len()==0 { Ok(()) }
    else { Err(errors.join("\n")) }
}

pub fn check_module_gates(ast: &File, modules: &Vec<ModuleType>) -> Result<(),String> {
    let mut errors: Vec<String> = Vec::new();
    // idの宣言,使用に問題がないかを確認
    for component in &ast.components {
        match component {
            Component::Module(module)=>{ // Moduleのみ処理
                // 宣言された名前の一覧
                let mut id_names = std::collections::HashSet::new();
                for input in &module.inputs {
                    if !id_names.insert(input) {
                        errors.push(format!("Defined id Duplicated: Input {}",input));
                    }
                }
                for gates in &module.gates {
                    for output in &gates.outputs {
                        if !id_names.insert(output) {
                            errors.push(format!("Defined id Duplicated: Gate-Out {}",output));
                        }
                    }
                }
                // 宣言されていない名前が使われていないかの確認
                for output in &module.outputs {
                    if !id_names.contains(output) {
                        errors.push(format!("Undefined id used: Output {}",output));
                    }
                }
                for gates in &module.gates {
                    for input in &gates.inputs {
                        if !id_names.contains(input) {
                            errors.push(format!("Undefined id used: Gate-In {}",input));
                        }
                    }
                }
            },
            _ => {} // モジュールでなければ何もしない
        }
    }
    // moduleの呼び出しに問題がないかを確認
    for component in &ast.components {
        match component {
            Component::Module(module)=>{ // Moduleのみ処理
                for gate in &module.gates {
                    match modules.iter().find(|m| m.name==gate.module_name).map(|m| &m.mtype) {
                        Some(mtype) => { // 使われているモジュールが定義されている場合
                            // moduleのinput,outputの型を確認
                            if gate.inputs.len()!=mtype.input_count||gate.outputs.len()!=mtype.output_count {
                                errors.push(format!("Used module with unmatched type: {} expected {}->{} but got {}->{}",gate.module_name,mtype.input_count,mtype.output_count,gate.inputs.len(),gate.outputs.len()));
                            }
                        },
                        None => { errors.push(format!("Undefined module used: {}",gate.module_name)); },
                    }
                }
            },
            _ => {} // モジュールでなければ何もしない
        }
    }
    if errors.len()==0 { Ok(()) }
    else { Err(errors.join("\n")) }
}