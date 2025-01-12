use std::result;

use super::super::test;
use super::super::vm;
use super::super::compiler;
use compiler::types::*;

pub fn document(product: IntermediateProducts) -> Result<String,String> {
    let mut modules = Vec::new();
    for component in product.ast.components {
        match component {
            Component::Module(module) => {
                modules.push(module);
            },
            _ => {},
        }
    }
    let table_body = modules.iter().map(|module| {
        match product.expanded_modules.get(&module.name) {
            Some(emod) => format!("| {} | {} -> {} | {} |",module.name,module.inputs.len(),module.outputs.len(),emod.gates_sequential.len()),
            None => format!("| error |")
        }
    }).collect::<Vec<_>>().join("\n");
    let table = format!("| name | type | size |\n| -- | -- | -- |\n{}",table_body);
    Ok(format!("{}",table))
}