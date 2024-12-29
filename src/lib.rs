use wasm_bindgen::prelude::*;
use std::str;

mod parser;
#[wasm_bindgen(js_name=Parse)]
pub fn export_parse(input: &str) -> JsValue {
    return JsValue::from(&match parser::parse(input) {
            Ok(res) => res,
            Err(res) => res,
    });
}