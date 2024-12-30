#![cfg(feature = "web")]

use wasm_bindgen::prelude::*;
use std::str;
mod compiler;

#[wasm_bindgen(js_name=CompilerIntermediateProducts)]
pub fn export_compiler_intermediate_products(input: &str) -> String {
    let result = compiler::intermediate_products(input);
    match serde_json::to_string_pretty(&result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}