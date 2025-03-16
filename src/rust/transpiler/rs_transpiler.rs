use crate::vm::types::*;

/// Transpiles Neknaj Circuit Game modules into Rust code.
pub fn transpile(modules: Vec<Module>) -> Result<String, String> {
    let mut transpiled_modules = Vec::new();
    for module in &modules {
        if module.gates_symmetry.len() > 0 {
            return Err(format!(
                "Rust transpiler does not support symmetry module: {}",
                module.name
            ));
        }
        // Generate associated function for functional module, to be placed inside the impl block.
        let func_module = if module.func {
            let input_params = (0..module.inputs as usize)
                .map(|i| format!("b{}: bool", i + module.gates_sequential.len()))
                .collect::<Vec<String>>()
                .join(", ");
            let func_head = format!(
                "        pub fn func({}) -> [bool; {}] {{",
                input_params,
                module.outputs.len()
            );
            let func_gates = module
                .gates_sequential
                .iter()
                .enumerate()
                .map(|(index, gate)| {
                    format!(
                        "            let b{} = !( b{} || b{} );",
                        index, gate.0, gate.1
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");
            let func_return = format!(
                "            [ {} ]",
                module
                    .outputs
                    .iter()
                    .map(|v| format!("b{}", v))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            format!(
                "{}\n{}\n{}\n        }}",
                func_head, func_gates, func_return
            )
        } else {
            "".to_string()
        };

        // Create the struct definition.
        let struct_def = format!(
            "    pub struct {} {{\n        b: Vec<bool>,\n    }}",
            module.name
        );
        // Start the impl block with constants.
        let consts = format!(
            "    impl {} {{\n        pub const INPUTS_LEN: usize = {};\n        pub const OUTPUTS_LEN: usize = {};\n        pub const GATES_SEQ_LEN: usize = {};",
            module.name,
            module.inputs,
            module.outputs.len(),
            module.gates_sequential.len()
        );
        // Generate the new, inputs, next, and outputs functions.
        let new_fn = format!(
            "        pub fn new() -> Self {{\n            Self {{ b: vec![false; Self::GATES_SEQ_LEN + Self::INPUTS_LEN] }}\n        }}",
        );
        let input_fn = format!(
            "        pub fn inputs(&mut self, i: [bool; Self::INPUTS_LEN]) {{\n{}\n        }}",
            (0..module.inputs as usize)
                .map(|index| {
                    format!(
                        "            self.b[Self::GATES_SEQ_LEN + {}] = i[{}];",
                        index, index
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
        let next_fn = format!(
            "        pub fn next(&mut self) -> &mut Self {{\n{}\n            self\n        }}",
            module
                .gates_sequential
                .iter()
                .enumerate()
                .map(|(index, gate)| {
                    format!(
                        "            self.b[{}] = !( self.b[{}] || self.b[{}] );",
                        index, gate.0, gate.1
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
        let outputs_fn = format!(
            "        pub fn outputs(&self) -> [bool; Self::OUTPUTS_LEN] {{\n            [ {} ]\n        }}",
            module
                .outputs
                .iter()
                .map(|v| format!("self.b[{}]", v))
                .collect::<Vec<String>>()
                .join(", ")
        );
        // Build the complete impl block.
        let mut impl_block = format!(
            "{}\n{}\n{}\n{}\n{}\n",
            consts, new_fn, input_fn, next_fn, outputs_fn
        );
        if module.func {
            impl_block.push_str(&format!("\n{}\n", func_module));
        }
        impl_block.push_str("    }\n");
        // Combine struct definition and impl block.
        let module_code = format!("{}\n\n{}", struct_def, impl_block);
        transpiled_modules.push(module_code);
    }
    // Wrap all module definitions inside the 'modules' module.
    Ok(format!(
        "// Generated by Neknaj Circuit Game\n\npub mod modules {{\n{}\n}}\n",
        transpiled_modules.join("\n\n")
    ))
}
