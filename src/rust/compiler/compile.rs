use std::{collections::HashMap, sync::Arc};

use super::types::*;

/// 全てのモジュールをnorのみで表す
/// modules: 依存関係によりトポロジカルソートされたモジュール名一覧
pub fn module_expansion(ast: &File,modules: &Vec<String>) -> Result<HashMap<String,CompiledModule>,Vec<String>> {
    let mut errors = Vec::new();
    let mut expanded_modules: HashMap<String,CompiledModule> = std::collections::HashMap::new(); // 全てのゲートがnorだけで構成されているmodule

    for module_name in modules.iter().rev().skip(1) {
        println!("Expansion Module {:#?}",module_name);
        let module = match ast.components.iter().find_map(|component| {
            if let Component::Module(ref module) = component {
                if &module.name == module_name { return Some((module.clone())); }
            }
            None
        }) {
            Some(module) => module,
            None => { errors.push(format!("Undefined module used: {}",module_name)); continue; },
        };
        println!("Module {:#?}",&module);
        let mut tmp_gates = Vec::new(); // inputを処理する前の状態
        let mut gates_rename = std::collections::HashMap::new(); // nameから#x.xを取得するためのmap
        let mut gates_name = std::collections::HashMap::new(); // #x.xからindexを取得するためのmap
        let mut gate_index = 0;
        // input以外を解決
        for gate in module.gates.clone() {
            let expanded_gate = if (gate.module_name!="nor") {
                match expanded_modules.get(&gate.module_name) { // nor以外の場合は展開済みのモジュールを取得
                    Some(v) => v.clone(),
                    None => {errors.push(format!("undefined gate used: {}",&gate.module_name));continue;}, // 依存関係によってトポロジカルソートされているので、nor以外は見つからないことは無い筈
                }
            } else { // norの場合
                CompiledModule { inputs: 2, outputs: vec![0], gates: vec![(1,2)] }
            };
            // println!("EGate {:#?}",&expanded_gate);
            println!("Gate {:#?}",&gate);
            let gate_pointer = tmp_gates.clone().len() as u32; // 今回のgateの先頭index
            // tmp_gatesに追加
            let mut egate_index = 0;
            for egate in &expanded_gate.gates {
                let input1 = if (egate.0<expanded_gate.gates.len() as u32) {
                        format!("#{}.{}",gate_index,egate.0)
                    }
                    else {
                        // gate.inputs[0].clone()
                        gate.inputs[egate.0 as usize - expanded_gate.gates.len()].clone()
                    };
                let input2 = if (egate.1<expanded_gate.gates.len() as u32) {
                        format!("#{}.{}",gate_index,egate.1)
                    }
                    else {
                        // gate.inputs[1].clone()
                        gate.inputs[egate.1 as usize - expanded_gate.gates.len()].clone()
                    };
                tmp_gates.push((
                    input1,
                    input2,
                ));
                gates_name.insert(format!("#{}.{}",gate_index,egate_index),gate_pointer+egate_index);
                egate_index+=1;
            }
            // outputをgates_nameに追加
            let mut output_index = 0;
            for output in expanded_gate.outputs {
                let name = match gate.outputs.get(output_index as usize) {
                    Some(v) => v.clone(),
                    None => {errors.push(format!("gate has invalid output {}",output));"".to_string()},
                };
                // println!("Name {:#?}",&name);
                gates_rename.insert(name,format!("#{}.{}",gate_index,output_index));
                output_index+=1;
            }
            gate_index+=1;
        }
        let mut input_index = 0;
        for input in &module.inputs {
            gates_rename.insert(input.clone(), format!("#i.{}",input_index));
            gates_name.insert(format!("#i.{}",input_index),tmp_gates.len() as u32+input_index);
            input_index+=1;
        }
        // println!("TmpGates {:#?}",&tmp_gates);
        // println!("GatesReName {:#?}",&gates_rename);
        // println!("GatesName {:#?}",&gates_name);
        // GatesReNameでinputを解決
        gate_index = 0; // gate_indexをリセット
        for gate in module.gates {
            let expanded_gate = if (gate.module_name!="nor") {
                match expanded_modules.get(&gate.module_name) { // nor以外の場合は展開済みのモジュールを取得
                    Some(v) => v.clone(),
                    None => {errors.push(format!("undefined gate used: {}",&gate.module_name));continue;}, // 依存関係によってトポロジカルソートされているので、nor以外は見つからないことは無い筈
                }
            } else { // norの場合
                CompiledModule { inputs: 2, outputs: vec![0], gates: vec![(1,2)] }
            };
            println!("EGate {:#?}",&expanded_gate);
            println!("Gate {:#?}",&gate);
            let mut input_index = 0;
            for input in gate.inputs {
                println!("Input {:#?}",&input);
                let input_resolved = match gates_rename.get(&input) {
                    Some(v) => v.clone(),
                    None => {errors.push(format!("undefined input used: {}",input));continue;},
                };
                let input_resolved2 = match gates_name.get(&input_resolved) {
                    Some(v) => v.clone(),
                    None => {errors.push(format!("undefined input used: {}",input_resolved));continue;},
                };
                // println!("Input {:#?} {} #{}.{}",&input,input_resolved,gate_index,tmp_gates.len() as u32+input_index);
                gates_name.insert(format!("#{}.{}",gate_index,tmp_gates.len() as u32+input_index),input_resolved2);
                gates_rename.insert(input,format!("#{}.{}",gate_index,tmp_gates.len() as u32+input_index));
                input_index+=1;
            }
            gate_index+=1;
        }
        println!("TmpGates {:#?}",&tmp_gates);
        println!("GatesReName {:#?}",&gates_rename);
        println!("GatesName {:#?}",&gates_name);
        // GatesNameでtmp_gatesを解決
        let mut expanded_gates = Vec::new();
        for gate in tmp_gates {
            let input1_ = match gates_rename.get(&gate.0) {
                Some(v) => v.clone(),
                None => gate.0.clone(),
            };
            let input2_ = match gates_rename.get(&gate.1) {
                Some(v) => v.clone(),
                None => gate.1.clone(),
            };
            let input1 = match gates_name.get(&input1_) {
                Some(v) => v.clone(),
                None => {errors.push(format!("undefined input used0: {} {}",&input1_,&gate.0));continue;},
            };
            let input2 = match gates_name.get(&input2_) {
                Some(v) => v.clone(),
                None => {errors.push(format!("undefined input used0: {} {}",&input2_,&gate.1));continue;},
            };
            // println!("Input {:?} {} {}",gate,input1,input2);
            expanded_gates.push((input1,input2));
        }
        // GatesNameでoutputを解決
        let mut outputs = Vec::new();
        for output in module.outputs {
            let output_resolved = match gates_rename.get(&output) {
                Some(v) => v.clone(),
                None => {errors.push(format!("undefined output used2: {}",output));continue;},
            };
            let output_resolved2 = match gates_name.get(&output_resolved) {
                Some(v) => v.clone(),
                None => {errors.push(format!("undefined output used3: {}",output));continue;},
            };
            outputs.push(output_resolved2);
        }
        println!("Outputs {:#?}",&outputs);
        println!("ExpandedGates {:#?}",&expanded_gates);
        expanded_modules.insert(module_name.clone(), CompiledModule {
            inputs: module.inputs.len() as u32,
            outputs: outputs,
            gates: expanded_gates,
        });
    }

    if errors.len()==0 { Ok(expanded_modules) }
    else { Err(errors) }
}


pub fn serialize_to_vec(module: CompiledModule) -> Vec<u32> {
    let mut result = Vec::new();
    // Add magic number
    result.push(0x6247434e);
    // Add data size
    result.push(32); // 32bits (u32)
    // Serialize inputs
    result.push(module.inputs);
    // Serialize outputs
    result.push(module.outputs.len() as u32);
    result.extend(&module.outputs);
    // Serialize gates
    result.push(module.gates.len() as u32);
    for gate in &module.gates {
        result.push(gate.0);
        result.push(gate.1);
    }
    result
}