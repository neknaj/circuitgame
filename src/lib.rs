#![cfg(feature = "web")]

use wasm_bindgen::prelude::*;
use std::str;
mod compiler;

#[wasm_bindgen(js_name=Parse)]
pub fn export_parse(input: &str) -> JsValue {
    return JsValue::from(&match compiler::parser::parser(input) {
        Ok(res) => format!("{:#?}",res),
        Err(res) => res,
    });
}