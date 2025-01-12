use super::types::*;

pub fn deserialize_from_vec(data: &[u32]) -> Result<Module, String> {
    let mut index = 0;

    // Check magic number
    if data.len() < 2 || data[index] != 0x6247434e {
        return Err("Invalid magic number".to_string());
    }
    index += 1;

    // Check data size
    if data[index] != 32 {
        return Err("Unsupported data size".to_string());
    }
    index += 1;

    // Func Module flag
    if index >= data.len() {
        return Err("Data is too short to contain func module flag".to_string());
    }
    let func = match data[index] {
        0 => false,
        1 => true,
        _ => {return Err("Unsupported func module flag".to_string());},
    };
    index += 1;

    // Deserialize name
    if index >= data.len() {
        return Err("Data is too short to contain name length".to_string());
    }
    let name_len = data[index] as usize;
    index += 1;

    if index + name_len > data.len() {
        return Err("Data is too short to contain name".to_string());
    }
    let name = data[index..index + name_len].to_vec().into_iter().filter_map(std::char::from_u32).collect();
    index += name_len;

    // Deserialize inputs
    if index >= data.len() {
        return Err("Data is too short to contain inputs".to_string());
    }
    let inputs = data[index];
    index += 1;

    // Deserialize outputs
    if index >= data.len() {
        return Err("Data is too short to contain outputs length".to_string());
    }
    let outputs_len = data[index] as usize;
    index += 1;

    if index + outputs_len > data.len() {
        return Err("Data is too short to contain outputs".to_string());
    }
    let outputs = data[index..index + outputs_len].to_vec();
    index += outputs_len;

    // Deserialize gates
    if index >= data.len() {
        return Err("Data is too short to contain gates length".to_string());
    }
    let gates_len_sequential = data[index] as usize;
    index += 1;
    if index >= data.len() {
        return Err("Data is too short to contain gates length".to_string());
    }
    let gates_len_symmetry = data[index] as usize;
    index += 1;

    if index + gates_len_sequential * 2 > data.len() {
        return Err("Data is too short to contain gates".to_string());
    }
    let mut gates_sequential = Vec::new();
    for _ in 0..gates_len_sequential {
        let gate: NORGate = (data[index], data[index + 1]);
        gates_sequential.push(gate);
        index += 2;
    }
    if index + gates_len_symmetry * 2 > data.len() {
        return Err("Data is too short to contain gates".to_string());
    }
    let mut gates_symmetry = Vec::new();
    for _ in 0..gates_len_symmetry {
        let gate: NORGate = (data[index], data[index + 1]);
        gates_symmetry.push(gate);
        index += 2;
    }

    // init cond
    let mut cond = Vec::new();
    cond.resize(gates_sequential.len()+gates_symmetry.len()+inputs as usize, false);

    Ok(Module {
        func,
        name,
        inputs,
        outputs,
        gates_sequential: gates_sequential,
        gates_symmetry: gates_symmetry,
        cond,
        tick: 0,
    })
}