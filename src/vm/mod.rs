mod types;
mod deserializer;

use types::*;

use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MODULE: Mutex<Module> = Mutex::new(Module {
        inputs: 0,
        outputs: Vec::new(),
        gates: Vec::new(),
    });

    static ref GATES: Mutex<Vec<bool>> = Mutex::new(Vec::new());
}

pub fn init(data: Vec<u32>) -> Result<(), String> {
    let mut module = MODULE.lock().unwrap();
    *module = deserializer::deserialize_from_vec(&data)?;

    let mut gates = GATES.lock().unwrap();
    gates.resize(module.gates.len()+module.inputs as usize, false);
    gates.fill(false);

    // println!("[INPUTS]  {:?}",module.inputs);
    // println!("[OUTPUTS] {:?}",module.outputs);
    // println!("[GATES]   {:?}",module.gates);

    Ok(())
}

pub fn set_input(index: u32,value: bool) -> Result<(),String> {
    let module = MODULE.lock().unwrap();
    let mut gates = GATES.lock().unwrap();
    match gates.get_mut(index as usize + module.gates.len()) {
        Some(v)=>{
            *v = value;
        },
        None =>{return Err(format!("Index out of bounds"));}
    };
    Ok(())
}

/// 全てのoutputを取得する
pub fn get_output() -> Result<Vec<bool>,String> {
    let module = MODULE.lock().unwrap();
    let gates = GATES.lock().unwrap();
    let mut outputs = Vec::new();
    for i in &module.outputs {
        match gates.get(*i as usize) {
            Some(v) => {
                outputs.push(v.clone());
            }
            None => {return Err(format!("Index out of bounds"));}
        }
    }
    Ok(outputs)
}

/// 全てのgatesを取得する input付き
// pub fn get_gates_all() -> Vec<bool> {
//     let gates = GATES.lock().unwrap();
//     gates.clone()
// }

/// gatesを一周更新する
pub fn next() -> Result<(), String> {
    let module = MODULE.lock().unwrap();
    let mut gates = GATES.lock().unwrap();

    // println!("{:?}",gates.clone().into_iter().map(|b| if b { "1" } else { "0" }).collect::<Vec<_>>().join(""));

    let mut gate_index = 0;
    for gate in &module.gates {
        let input1 = *gates.get(gate.0 as usize).ok_or("gates access error")?;
        let input2 = *gates.get(gate.1 as usize).ok_or("gates access error")?;
        let output = gates.get_mut(gate_index).ok_or("gates access error")?;
        *output = !(input1||input2); // input1,input2のNORを計算する
        // println!("{} = {} NOR {} // gate {}",!(input1||input2),input1,input2,gate_index);
        gate_index+=1;
    }

    Ok(())
}