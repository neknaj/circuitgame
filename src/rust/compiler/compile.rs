use std::collections::HashMap;


use super::types::*;

/// 全てのモジュールをnorのみで表す
/// modules: 依存関係によりトポロジカルソートされたモジュール名一覧
pub fn module_expansion(ast: &File,modules: &Vec<String>) -> Result<HashMap<String,CompiledModule>,Vec<String>> {
    let mut errors = Vec::new();
    let mut expanded_modules: HashMap<String,CompiledModule> = std::collections::HashMap::new(); // 全てのゲートがnorだけで構成されているmodule

    for module_name in modules.iter().rev() {
        if module_name=="nor" {
            expanded_modules.insert(module_name.clone(),CompiledModule {
                func: true,
                name: "nor".to_string(),
                inputs: 2,
                outputs: vec![0],
                gates_sequential: vec![(CompiledGateInput::Input(0),CompiledGateInput::Input(1))],
                gates_symmetry: Vec::new(),
            });
            continue;
        }
        let module = match ast.components.iter().find_map(|component| {
            if let Component::Module(ref module) = component {
                if &module.name==module_name {
                    return Some(module.clone());
                }
            }
            None
        }) {
            Some(v) => v,
            None => {errors.push(format!("Undefined module used: {}",module_name));continue;}
        };
        // 各ゲートのpointerを計算
        let mut gates_pointer = Vec::new();
        let mut gate_count = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            gates_pointer.push(gate_count);
            gate_count += expanding_gate.gates_sequential.len() as u32;
        }
        // gateのoutput名前とindexの対応表を作る
        let mut output_map = HashMap::new();
        let mut gate_index = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            let mut output_index = 0;
            for output in expanding_gate.outputs.clone() {
                let output_name = gate.outputs[output_index as usize].clone();
                output_map.insert(output_name, CompiledGateInput::NorGate(output+gates_pointer[gate_index as usize]));
                output_index+=1;
            }
            gate_index+=1;
        }
        // moduleのinputを追加
        let mut input_index = 0;
        for input in module.inputs.clone() {
            output_map.insert(input,CompiledGateInput::Input(input_index));
            input_index+=1;
        }
        // gateのinputsを解決しながら展開
        let mut expanded = Vec::new();
        gate_index = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            for egate in expanding_gate.gates_sequential.clone() {
                let input0 = match egate.0 {
                    CompiledGateInput::NorGate(n) => CompiledGateInput::NorGate(n+gates_pointer[gate_index as usize]),
                    CompiledGateInput::Input(n) => match output_map.get(&gate.inputs[n as usize]) {
                        Some(v) => v.clone(),
                        None => {errors.push(format!("Undefined gate used: {}",gate.inputs[n as usize]));continue;}
                    },
                };
                let input1 = match egate.1 {
                    CompiledGateInput::NorGate(n) => CompiledGateInput::NorGate(n+gates_pointer[gate_index as usize]),
                    CompiledGateInput::Input(n) => match output_map.get(&gate.inputs[n as usize]) {
                        Some(v) => v.clone(),
                        None => {errors.push(format!("Undefined gate used: {}",gate.inputs[n as usize]));continue;}
                    },
                };
                expanded.push((input0,input1));
            }
            gate_index+=1;
        }
        // moduleのoutputを解決
        let mut outputs = Vec::new();
        for output in module.outputs.clone() {
            let output_solved = match output_map.get(&output) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used in output: {}",output));continue;}
            };
            let output_checked = match output_solved {
                CompiledGateInput::Input(v) => v+gate_index,
                CompiledGateInput::NorGate(v) => v,
            };
            outputs.push(output_checked);
        }
        // expanded_modulesに追加
        expanded_modules.insert(module_name.clone(),CompiledModule {
            func: module.func,
            name: module_name.clone(),
            inputs: module.inputs.len() as u32,
            outputs: outputs,
            gates_sequential: expanded,
            gates_symmetry: Vec::new(),
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
    // Add func module flag
    result.push(if module.func {1} else {0});
    // Add func name
    let encoded = module.name.chars().map(|c| c as u32).collect::<Vec<u32>>();
    result.push(encoded.len() as u32);
    result.extend(encoded);
    // Serialize inputs
    result.push(module.inputs);
    // Serialize outputs
    result.push(module.outputs.len() as u32);
    result.extend(&module.outputs);
    // Serialize gates
    result.push(module.gates_sequential.len() as u32);
    result.push(module.gates_symmetry.len() as u32);
    let gates_len = (module.gates_sequential.len() + module.gates_symmetry.len()) as u32;
    for gate in &module.gates_sequential {
        result.push(match gate.0 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + gates_len,
        });
        result.push(match gate.1 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + gates_len,
        });
    }
    for gate in &module.gates_symmetry {
        result.push(match gate.0 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + gates_len,
        });
        result.push(match gate.1 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + gates_len,
        });
    }
    result
}