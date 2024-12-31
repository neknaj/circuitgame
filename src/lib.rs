#![cfg(feature = "web")]

mod compiler;
mod test;
mod vm;

use wasm_bindgen::prelude::*;
use std::str;

#[wasm_bindgen(js_name=CompilerIntermediateProducts)]
pub fn export_compiler_intermediate_products(input: &str) -> String {
    let result = compiler::intermediate_products(input);
    match serde_json::to_string_pretty(&result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}

#[wasm_bindgen(js_name=Test)]
pub fn export_test(input: &str) -> String {
    let result = compiler::intermediate_products(input);
    if result.errors.len()>0 {
        return format!("compiling error");
    }
    let test_result = test::test(result);
    match serde_json::to_string_pretty(&test_result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}

// #[wasm_bindgen(js_name=Compile)]
// pub fn export_compile(input: &str,module: &str) -> Result<Vec<u32>,String> {
//     compiler::compile(input,module)
// }

#[wasm_bindgen(js_name=Compile)]
pub fn export_compile(input: &str,module: &str) -> Vec<u32> {
    match compiler::compile(input,module) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    }
}