#![cfg(feature = "web")]

use wasm_bindgen::prelude::*;
use std::str;
mod compiler;

#[wasm_bindgen(js_name=Compile)]
pub fn export_compile(input: &str) -> String {
    let result = compiler::compile(input);
    match serde_json::to_string_pretty(&result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}