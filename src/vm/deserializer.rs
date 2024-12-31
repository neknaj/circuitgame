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
    let gates_len = data[index] as usize;
    index += 1;

    if index + gates_len * 2 > data.len() {
        return Err("Data is too short to contain gates".to_string());
    }
    let mut gates = Vec::new();
    for _ in 0..gates_len {
        let gate: NORGate = (data[index], data[index + 1]);
        gates.push(gate);
        index += 2;
    }

    Ok(Module {
        inputs,
        outputs,
        gates,
    })
}