use std::{collections::HashMap, sync::Arc};

use colored::Colorize;

use super::types::*;

/// 全てのモジュールをnorのみで表す
/// modules: 依存関係によりトポロジカルソートされたモジュール名一覧
pub fn module_expansion(ast: &File,modules: &Vec<String>) -> Result<HashMap<String,CompiledModule>,Vec<String>> {
    let mut errors = Vec::new();
    let mut expanded_modules: HashMap<String,CompiledModule> = std::collections::HashMap::new(); // 全てのゲートがnorだけで構成されているmodule

    for module_name in modules.iter().rev() {
        println!("\n<< module_name: {} >>\n",module_name.cyan());
        if module_name=="nor" {
            expanded_modules.insert(module_name.clone(),CompiledModule {
                inputs: 2,
                outputs: vec![0],
                gates: vec![(CompiledGateInput::Input(0),CompiledGateInput::Input(1))]
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
        println!("module: {:?}",module);
        // 各ゲートのpointerを計算
        let mut gates_pointer = Vec::new();
        let mut gate_count = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            // println!("gate {} : {:?}\n                 {:?}",&gate.module_name,gate,expanding_gate);
            gates_pointer.push(gate_count);
            gate_count += expanding_gate.gates.len() as u32;
        }
        println!("gates_pointer: {:?}",gates_pointer);
        // gateのoutput名前とindexの対応表を作る
        let mut output_map = HashMap::new();
        let mut gate_index = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            // println!("gate {} : {:?}\n                 {:?}",&gate.module_name,gate,expanding_gate);
            let mut output_index = 0;
            for output in expanding_gate.outputs.clone() {
                let output_name = gate.outputs[output_index as usize].clone();
                // println!("output_name: {} {}",output_name,output);
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
        println!("output_map: {:?}",output_map);
        // gateのinputsを解決しながら展開
        let mut expanded = Vec::new();
        gate_index = 0;
        for gate in module.gates.clone() {
            let expanding_gate = match expanded_modules.get(&gate.module_name) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used: {}",gate.module_name));continue;}
            };
            println!("gate {} : {:?}\n                 {:?}",&gate.module_name,gate,expanding_gate);
            for egate in expanding_gate.gates.clone() {
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
        println!("expanded: {:?}",expanded);
        // moduleのoutputを解決
        let mut outputs = Vec::new();
        for output in module.outputs.clone() {
            let output_solved = match output_map.get(&output) {
                Some(v) => v.clone(),
                None => {errors.push(format!("Undefined gate used in output: {}",output));continue;}
            };
            // outputにinputを直接指定することはできない
            let output_checked = match output_solved {
                CompiledGateInput::Input(_) => {errors.push(format!("Output cannot be input: {}",output));continue;}
                CompiledGateInput::NorGate(v) => v,
            };
            outputs.push(output_checked);
        }
        println!("outputs: {:?}",outputs);
        // expanded_modulesに追加
        expanded_modules.insert(module_name.clone(),CompiledModule {
            inputs: module.inputs.len() as u32,
            outputs: outputs,
            gates: expanded,
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
        result.push(match gate.0 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + module.gates.len() as u32,
        });
        result.push(match gate.1 {
            CompiledGateInput::NorGate(n) => n,
            CompiledGateInput::Input(n) => n + module.gates.len() as u32,
        });
    }
    result
}