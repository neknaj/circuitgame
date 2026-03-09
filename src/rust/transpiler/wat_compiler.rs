use crate::vm::types::*;

/// Compile a list of `Module`s into a single WAT (WebAssembly Text) string.
pub fn compile(modules: Vec<Module>) -> Result<String, String> {
    // Start the WAT module with a memory declaration
    let mut wat = String::new();
    wat.push_str("(module\n");
    wat.push_str("    ;; 64KiB memory for all module states\n");
    wat.push_str("    (memory $mem 1)\n");
    wat.push_str("    (export \"memory\" (memory $mem))\n\n");

    // Track byte-offset for each module's state
    let mut current_offset = 0;
    for module in &modules {
        let state_len = (module.gates_sequential.len() + module.inputs as usize) as u32;
        let offset = current_offset;
        current_offset += state_len * 4; // each bool as 4 bytes (i32.store)

        // Function: inputs_<name>
        wat.push_str(&format!(
            "    ;; Load inputs for module `{}` at offset {}\n",
            module.name, offset
        ));
        // Open func and parameters (no trailing `)` here)
        wat.push_str(&format!("    (func $inputs_{}", module.name));
        for i in 0..module.inputs {
            wat.push_str(&format!(" (param $i{} i32)", i));
        }
        wat.push_str("\n");
        // Body: store each input into memory
        for i in 0..module.inputs {
            let addr = offset + (module.gates_sequential.len() as u32 + i) * 4;
            wat.push_str(&format!(
                "        (i32.store (i32.const {}) (local.get $i{}))\n",
                addr, i
            ));
        }
        // Close func
        wat.push_str("    )\n\n");

        // Function: next_<name>
        wat.push_str(&format!(
            "    ;; Step sequential NOR gates for `{}`\n",
            module.name
        ));
        // Open func
        wat.push_str(&format!("    (func $next_{}\n", module.name));
        for (idx, &(a, b)) in module.gates_sequential.iter().enumerate() {
            let addr = offset + idx as u32 * 4;
            let addr_a = offset + a * 4;
            let addr_b = offset + b * 4;
            wat.push_str(&format!("        ;; b{} = !(b{} || b{})\n", idx, a, b));
            wat.push_str(&format!(
                "        (i32.store\n            (i32.const {})\n            (i32.eqz (i32.or\n                (i32.load (i32.const {}))\n                (i32.load (i32.const {}))\n            ))\n        )\n",
                addr, addr_a, addr_b
            ));
        }
        // Close func
        wat.push_str("    )\n\n");

        // Function: get_outputs_<name>
        wat.push_str(&format!("    ;; Read outputs for `{}`\n", module.name));
        // Open func with results
        wat.push_str(&format!("    (func $get_outputs_{} (result", module.name));
        for _ in 0..module.outputs.len() {
            wat.push_str(" i32");
        }
        wat.push_str(")\n");
        // Body: load each output bit
        for &out_idx in &module.outputs {
            let addr = offset + out_idx as u32 * 4;
            wat.push_str(&format!("        (i32.load (i32.const {}))\n", addr));
        }
        // Close func
        wat.push_str("    )\n\n");
    }

    // Close module
    wat.push_str(")\n");
    Ok(wat)
}
