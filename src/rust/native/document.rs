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
        format!("| {} | {} -> {} | {} |",module.name,module.inputs.len(),module.outputs.len(),module.gates.len())
    }).collect::<Vec<_>>().join("\n");
    let table = format!("| name | type | size |\n| -- | -- | -- |\n{}",table_body);
    Ok(format!("{}",table))
}