use std::{collections::HashMap, fmt::format, process::Output};

use super::types::*;

/// 全てのモジュールをnorのみで表す
/// modules: 依存関係によりトポロジカルソートされたモジュール名一覧
pub fn module_expansion(ast: &File,modules: &Vec<String>) -> Result<(),Vec<String>> {
    let mut errors = Vec::new();
    let mut expanded_modules: HashMap<String,CompiledModule> = std::collections::HashMap::new();

    for module_name in modules.iter().rev().skip(1) {
        println!("{:#?}",module_name);
        match ast.components.iter().find_map( |component| { // ASTの中からmodule_nameを見つける
                if let Component::Module(ref module) = component {
                    if &module.name == module_name { return Some(module.clone()); }
                }
                None
            }
        ) {
            Some(module) => {
                println!("found module {:?}",module);
                // 展開したモジュールに付ける名前は #id.0の形にする
                // idは展開する毎にインクリメント, 数値は展開するCompiledModuleのインデックス
                let mut expansion_id = 0;
                let mut tmp_gates = Vec::new();
                let mut gates_name: HashMap<String,u32> = std::collections::HashMap::new();
                let mut gates_rename: HashMap<String,String> = std::collections::HashMap::new();
                // gatesをnorだけにする
                for gate in module.gates {
                    if gate.module_name=="nor" { // norはそのまま追加するだけ
                        let output = match gate.outputs.get(0) {
                            Some(v) => v.clone(),
                            None => {errors.push(format!("nor has invalid output"));"".to_string()},
                        };
                        let input1 = match gate.inputs.get(0) {
                            Some(v) => v.clone(),
                            None => {errors.push(format!("nor has invalid input"));"".to_string()},
                        };
                        let input2 = match gate.inputs.get(1) {
                            Some(v) => v.clone(),
                            None => {errors.push(format!("nor has invalid input"));"".to_string()},
                        };
                        gates_name.insert(output, tmp_gates.len() as u32);
                        tmp_gates.push((input1,input2));
                    }
                    else { // norでなかったら展開する
                        match expanded_modules.get(&gate.module_name) {
                            Some(v) => {
                                println!("not the nor gate {:#?}",&gate.module_name);
                                println!("{:#?}",&gate);
                                let mut gate_index = 0;
                                // gateを追加
                                for gate in &v.gates {
                                    gates_name.insert(format!("#{}.{}",expansion_id,gate_index), tmp_gates.len() as u32);
                                    tmp_gates.push((format!("#{}.{}",expansion_id,gate.0),format!("#{}.{}",expansion_id,gate.1)));
                                    gate_index+=1;
                                }
                                // inputを処理
                                println!("v inputs  {:?}",&v.inputs);
                                for i in 0..v.inputs {
                                    let name = match gate.inputs.get(i as usize) {
                                        Some(v) => v.clone(),
                                        None => {errors.push(format!("gate has invalid input {}",i));"".to_string()},
                                    };
                                    gates_rename.insert(format!("#{}.{}",expansion_id,v.gates.len() as u32+i), name);
                                }
                                // outputを処理
                                println!("v outputs {:?}",&v.outputs);
                                let mut output_index = 0;
                                for i in v.outputs.clone() {
                                    println!("{}",i);
                                    println!("{:#?}",gate.outputs);
                                    let name = match gate.outputs.get(output_index as usize) {
                                        Some(v) => v.clone(),
                                        None => {errors.push(format!("gate has invalid output {}",i));"".to_string()},
                                    };
                                    gates_rename.insert(name,format!("#{}.{}",expansion_id,i));
                                    output_index+=1;
                                }
                                expansion_id+=1;
                            },
                            None => {errors.push(format!("undefined gate used"));},
                        };
                    }
                }
                // gates_nameにinputsを追加
                let mut inputs_index = 0;
                for input in &module.inputs {
                    gates_name.insert(input.clone(), tmp_gates.len() as u32 +inputs_index);
                    inputs_index+=1;
                }
                // tmp_gatesのinputの名前を解決する
                let mut gates = Vec::new();
                for gate in &tmp_gates {
                    // renameを解決
                    let gate0 = match gates_rename.get(&gate.0) {
                        Some(v) => v.clone(),
                        None => gate.0.clone(),
                    };
                    let gate1 = match gates_rename.get(&gate.1) {
                        Some(v) => v.clone(),
                        None => gate.1.clone(),
                    };
                    // indexを解決
                    let input1 = match gates_name.get(&gate0) {
                        Some(v) => v.clone(),
                        None => {errors.push(format!("gate has invalid input {} {}",&gate.0,&gate0));100},
                    };
                    let input2 = match gates_name.get(&gate1) {
                        Some(v) => v.clone(),
                        None => {errors.push(format!("gate has invalid input {} {}",&gate.1,&gate1));101},
                    };
                    gates.push((input1,input2));
                }
                // moduleのoutputの名前を解決する
                let mut outputs = Vec::new();
                for output in module.outputs {
                    // renameを解決
                    let output_ = match gates_rename.get(&output) {
                        Some(v) => v.clone(),
                        None => output.clone(),
                    };
                    // indexを解決
                    let out = match gates_name.get(&output_) {
                        Some(v) => v.clone(),
                        None => {errors.push(format!("module has invalid output"));102},
                    };
                    outputs.push(out);
                }
                println!("GatesName {:?}",gates_name);
                println!("GatesRName{:?}",gates_rename);
                println!("TMP Gates {:?}",tmp_gates);
                println!("Gates     {:?}",gates);
                println!("Outputs   {:?}",outputs);
                // expanded_modulesに保存
                expanded_modules.insert(module_name.clone(), CompiledModule {
                    inputs: module.inputs.len() as u32,
                    outputs: outputs,
                    gates
                });
                println!("{:#?}",expanded_modules);
            },
            None => { errors.push(format!("Undefined module used: {}",module_name)); },
        }
    }

    if errors.len()==0 { Ok(()) }
    else { Err(errors) }
}